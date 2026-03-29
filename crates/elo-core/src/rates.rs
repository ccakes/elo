use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use moneta::{Currency, Unit};
use serde::{Deserialize, Serialize};

const RATES_URL: &str = "https://s.numi.app/rates";
const CRYPTO_URL: &str = "https://s.numi.app/crypto";
const CACHE_MAX_AGE: u64 = 10800; // 3 hours, matching API Cache-Control

/// Crypto currencies to register with moneta's global registry.
const CRYPTO_CURRENCIES: &[(&str, &str, u8)] = &[
    ("BTC", "Bitcoin", 8),
    ("ETH", "Ethereum", 8),
    ("DOGE", "Dogecoin", 8),
    ("XRP", "Ripple", 6),
    ("LTC", "Litecoin", 8),
    ("XMR", "Monero", 8),
    ("DASH", "Dash", 8),
];

static CRYPTO_INIT: OnceLock<()> = OnceLock::new();

fn ensure_crypto_registered() {
    CRYPTO_INIT.get_or_init(|| {
        for &(code, name, decimals) in CRYPTO_CURRENCIES {
            let _ = Currency::new(code, name, decimals);
        }
    });
}

/// Cached API response stored on disk.
#[derive(Serialize, Deserialize)]
struct RatesCache {
    etag_fiat: Option<String>,
    etag_crypto: Option<String>,
    last_modified_fiat: Option<String>,
    last_modified_crypto: Option<String>,
    fetched_at: u64,
    fiat: ApiRates,
    crypto: ApiRates,
}

#[derive(Serialize, Deserialize, Default)]
struct ApiRates {
    #[serde(default)]
    timestamp: Option<u64>,
    #[serde(default)]
    base: String,
    #[serde(default)]
    rates: HashMap<String, f64>,
}

/// Exchange rate store backed by the Numi rate API and moneta currency definitions.
pub struct RateStore {
    /// Currency code → units per 1 USD (e.g. "EUR" → 0.87)
    rates: HashMap<String, f64>,
}

impl RateStore {
    /// Load rates: try cache first, fetch if stale, fall back to stale cache if offline.
    pub fn load() -> Option<Arc<Self>> {
        ensure_crypto_registered();

        let cache_path = cache_file_path()?;
        let now = now_unix();

        // Try loading existing cache
        let cached = load_cache(&cache_path);

        if let Some(ref cache) = cached
            && cache.fetched_at + CACHE_MAX_AGE > now
        {
            return Some(Arc::new(Self::from_cache(cache)));
        }

        // Cache is stale or missing — try fetching
        match fetch_rates(&cached) {
            Some(new_cache) => {
                let _ = save_cache(&cache_path, &new_cache);
                Some(Arc::new(Self::from_cache(&new_cache)))
            }
            None => {
                // Fetch failed — use stale cache as fallback
                cached.map(|c| Arc::new(Self::from_cache(&c)))
            }
        }
    }

    fn from_cache(cache: &RatesCache) -> Self {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), 1.0);

        // Fiat rates are authoritative
        for (code, rate) in &cache.fiat.rates {
            rates.insert(code.clone(), *rate);
        }

        // Add crypto rates only for codes not already present
        for (code, rate) in &cache.crypto.rates {
            rates.entry(code.clone()).or_insert(*rate);
        }

        Self { rates }
    }

    /// Convert `amount` from one currency to another.
    ///
    /// All rates are USD-based, so: `result = amount * (to_rate / from_rate)`.
    pub fn convert(&self, amount: f64, from: &str, to: &str) -> Option<f64> {
        if from == to {
            return Some(amount);
        }
        let from_rate = self.rates.get(from)?;
        let to_rate = self.rates.get(to)?;
        Some(amount * to_rate / from_rate)
    }

    /// Check whether a rate exists for the given currency code.
    pub fn has_rate(&self, code: &str) -> bool {
        self.rates.contains_key(code)
    }

    /// Validate a currency code against both moneta's ISO registry and loaded rates.
    pub fn is_known_currency(&self, code: &str) -> bool {
        self.rates.contains_key(code) || Currency::from_symbol(code).is_some()
    }
}

// ---------------------------------------------------------------------------
// Cache I/O
// ---------------------------------------------------------------------------

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("elo"))
}

fn cache_file_path() -> Option<PathBuf> {
    cache_dir().map(|d| d.join("rates_cache.json"))
}

fn load_cache(path: &PathBuf) -> Option<RatesCache> {
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_cache(path: &PathBuf, cache: &RatesCache) -> Option<()> {
    let dir = path.parent()?;
    fs::create_dir_all(dir).ok()?;
    let json = serde_json::to_string(cache).ok()?;
    fs::write(path, json).ok()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// HTTP fetching
// ---------------------------------------------------------------------------

/// Fetch a single rates endpoint, using conditional headers from the previous cache.
/// Returns `None` if the server responds 304 Not Modified or if the request fails.
fn fetch_endpoint(
    url: &str,
    prev_etag: Option<&str>,
    prev_last_modified: Option<&str>,
) -> Option<(ApiRates, Option<String>, Option<String>)> {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .http_status_as_error(false)
        .build();
    let agent = ureq::Agent::new_with_config(config);

    let mut req = agent.get(url);
    if let Some(etag) = prev_etag {
        req = req.header("If-None-Match", etag);
    }
    if let Some(lm) = prev_last_modified {
        req = req.header("If-Modified-Since", lm);
    }

    let mut resp = req.call().ok()?;

    if resp.status() == 304 {
        return None;
    }
    if !resp.status().is_success() {
        return None;
    }

    let etag = resp
        .headers()
        .get("ETag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let last_modified = resp
        .headers()
        .get("Last-Modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = resp.body_mut().read_to_string().ok()?;
    let rates: ApiRates = serde_json::from_str(&body).ok()?;

    Some((rates, etag, last_modified))
}

fn fetch_rates(prev_cache: &Option<RatesCache>) -> Option<RatesCache> {
    let now = now_unix();

    let (prev_etag_fiat, prev_lm_fiat) = prev_cache
        .as_ref()
        .map(|c| (c.etag_fiat.as_deref(), c.last_modified_fiat.as_deref()))
        .unwrap_or((None, None));

    let (prev_etag_crypto, prev_lm_crypto) = prev_cache
        .as_ref()
        .map(|c| (c.etag_crypto.as_deref(), c.last_modified_crypto.as_deref()))
        .unwrap_or((None, None));

    // Fetch fiat rates
    let (fiat, etag_fiat, lm_fiat) = match fetch_endpoint(RATES_URL, prev_etag_fiat, prev_lm_fiat) {
        Some(result) => (result.0, result.1, result.2),
        None => {
            // 304 Not Modified or error — reuse previous data
            if let Some(prev) = prev_cache {
                (
                    ApiRates {
                        timestamp: prev.fiat.timestamp,
                        base: prev.fiat.base.clone(),
                        rates: prev.fiat.rates.clone(),
                    },
                    prev.etag_fiat.clone(),
                    prev.last_modified_fiat.clone(),
                )
            } else {
                return None;
            }
        }
    };

    // Fetch crypto rates
    let (crypto, etag_crypto, lm_crypto) =
        match fetch_endpoint(CRYPTO_URL, prev_etag_crypto, prev_lm_crypto) {
            Some(result) => (result.0, result.1, result.2),
            None => {
                if let Some(prev) = prev_cache {
                    (
                        ApiRates {
                            timestamp: prev.crypto.timestamp,
                            base: prev.crypto.base.clone(),
                            rates: prev.crypto.rates.clone(),
                        },
                        prev.etag_crypto.clone(),
                        prev.last_modified_crypto.clone(),
                    )
                } else {
                    (ApiRates::default(), None, None)
                }
            }
        };

    Some(RatesCache {
        etag_fiat,
        etag_crypto,
        last_modified_fiat: lm_fiat,
        last_modified_crypto: lm_crypto,
        fetched_at: now,
        fiat,
        crypto,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_same_currency() {
        let store = RateStore {
            rates: HashMap::from([("USD".to_string(), 1.0), ("EUR".to_string(), 0.87)]),
        };
        assert_eq!(store.convert(100.0, "USD", "USD"), Some(100.0));
    }

    #[test]
    fn test_convert_usd_to_eur() {
        let store = RateStore {
            rates: HashMap::from([("USD".to_string(), 1.0), ("EUR".to_string(), 0.87)]),
        };
        let result = store.convert(100.0, "USD", "EUR").unwrap();
        assert!((result - 87.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_eur_to_usd() {
        let store = RateStore {
            rates: HashMap::from([("USD".to_string(), 1.0), ("EUR".to_string(), 0.87)]),
        };
        let result = store.convert(87.0, "EUR", "USD").unwrap();
        assert!((result - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_cross_currency() {
        let store = RateStore {
            rates: HashMap::from([
                ("USD".to_string(), 1.0),
                ("EUR".to_string(), 0.87),
                ("GBP".to_string(), 0.75),
            ]),
        };
        let result = store.convert(100.0, "EUR", "GBP").unwrap();
        assert!((result - 86.21).abs() < 0.1);
    }

    #[test]
    fn test_convert_unknown_currency() {
        let store = RateStore {
            rates: HashMap::from([("USD".to_string(), 1.0)]),
        };
        assert_eq!(store.convert(100.0, "USD", "XYZ"), None);
    }

    #[test]
    fn test_has_rate() {
        let store = RateStore {
            rates: HashMap::from([("USD".to_string(), 1.0), ("EUR".to_string(), 0.87)]),
        };
        assert!(store.has_rate("USD"));
        assert!(store.has_rate("EUR"));
        assert!(!store.has_rate("XYZ"));
    }
}
