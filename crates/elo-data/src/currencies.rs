/// Currency definition
#[derive(Debug, Clone)]
pub struct CurrencyDef {
    pub code: &'static str,
    pub names: &'static [&'static str],
    pub symbol: Option<&'static str>,
}

pub static CURRENCIES: &[CurrencyDef] = &[
    CurrencyDef {
        code: "USD",
        names: &["USD", "dollar", "dollars", "us dollar", "us dollars"],
        symbol: Some("$"),
    },
    CurrencyDef {
        code: "EUR",
        names: &["EUR", "euro", "euros"],
        symbol: Some("€"),
    },
    CurrencyDef {
        code: "GBP",
        names: &["GBP", "pound sterling", "british pound", "british pounds"],
        symbol: Some("£"),
    },
    CurrencyDef {
        code: "JPY",
        names: &["JPY", "yen", "japanese yen"],
        symbol: Some("¥"),
    },
    CurrencyDef {
        code: "CNY",
        names: &["CNY", "yuan", "chinese yuan", "renminbi", "RMB"],
        symbol: Some("¥"),
    },
    CurrencyDef {
        code: "AUD",
        names: &["AUD", "australian dollar", "australian dollars"],
        symbol: Some("A$"),
    },
    CurrencyDef {
        code: "CAD",
        names: &["CAD", "canadian dollar", "canadian dollars"],
        symbol: Some("C$"),
    },
    CurrencyDef {
        code: "CHF",
        names: &["CHF", "swiss franc", "swiss francs"],
        symbol: Some("CHF"),
    },
    CurrencyDef {
        code: "SEK",
        names: &["SEK", "swedish krona", "swedish kronor"],
        symbol: Some("kr"),
    },
    CurrencyDef {
        code: "NZD",
        names: &["NZD", "new zealand dollar", "new zealand dollars"],
        symbol: Some("NZ$"),
    },
    CurrencyDef {
        code: "KRW",
        names: &["KRW", "korean won", "south korean won"],
        symbol: Some("₩"),
    },
    CurrencyDef {
        code: "SGD",
        names: &["SGD", "singapore dollar", "singapore dollars"],
        symbol: Some("S$"),
    },
    CurrencyDef {
        code: "NOK",
        names: &["NOK", "norwegian krone", "norwegian kroner"],
        symbol: Some("kr"),
    },
    CurrencyDef {
        code: "MXN",
        names: &["MXN", "mexican peso", "mexican pesos"],
        symbol: Some("MX$"),
    },
    CurrencyDef {
        code: "INR",
        names: &["INR", "indian rupee", "indian rupees", "rupee", "rupees"],
        symbol: Some("₹"),
    },
    CurrencyDef {
        code: "RUB",
        names: &["RUB", "russian ruble", "russian rubles", "ruble", "rubles"],
        symbol: Some("₽"),
    },
    CurrencyDef {
        code: "BRL",
        names: &["BRL", "brazilian real", "brazilian reais", "real", "reais"],
        symbol: Some("R$"),
    },
    CurrencyDef {
        code: "ZAR",
        names: &["ZAR", "south african rand", "rand"],
        symbol: Some("R"),
    },
    CurrencyDef {
        code: "HKD",
        names: &["HKD", "hong kong dollar", "hong kong dollars"],
        symbol: Some("HK$"),
    },
    CurrencyDef {
        code: "TWD",
        names: &["TWD", "taiwan dollar", "new taiwan dollar"],
        symbol: Some("NT$"),
    },
    CurrencyDef {
        code: "PLN",
        names: &["PLN", "polish zloty", "zloty"],
        symbol: Some("zł"),
    },
    CurrencyDef {
        code: "THB",
        names: &["THB", "thai baht", "baht"],
        symbol: Some("฿"),
    },
    CurrencyDef {
        code: "IDR",
        names: &["IDR", "indonesian rupiah", "rupiah"],
        symbol: Some("Rp"),
    },
    CurrencyDef {
        code: "CZK",
        names: &["CZK", "czech koruna", "koruna"],
        symbol: Some("Kč"),
    },
    CurrencyDef {
        code: "ILS",
        names: &["ILS", "israeli shekel", "shekel", "shekels"],
        symbol: Some("₪"),
    },
    CurrencyDef {
        code: "PHP",
        names: &["PHP", "philippine peso", "philippine pesos"],
        symbol: Some("₱"),
    },
    CurrencyDef {
        code: "TRY",
        names: &["TRY", "turkish lira", "lira"],
        symbol: Some("₺"),
    },
    CurrencyDef {
        code: "DKK",
        names: &["DKK", "danish krone", "danish kroner"],
        symbol: Some("kr"),
    },
    CurrencyDef {
        code: "BTC",
        names: &["BTC", "bitcoin", "bitcoins"],
        symbol: Some("₿"),
    },
    CurrencyDef {
        code: "ETH",
        names: &["ETH", "ether", "ethereum"],
        symbol: None,
    },
];

/// Find a currency by code, name, or symbol
pub fn find_currency(name: &str) -> Option<&'static CurrencyDef> {
    let upper = name.to_uppercase();
    let lower = name.to_lowercase();

    // Exact code match first
    for c in CURRENCIES {
        if c.code == upper {
            return Some(c);
        }
    }

    // Symbol match
    for c in CURRENCIES {
        if let Some(sym) = c.symbol
            && sym == name
        {
            return Some(c);
        }
    }

    // Name match (case-insensitive)
    for c in CURRENCIES {
        for &n in c.names {
            if n.to_lowercase() == lower {
                return Some(c);
            }
        }
    }

    None
}
