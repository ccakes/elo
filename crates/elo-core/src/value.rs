use std::fmt;

/// Rich typed value produced by the evaluator
#[derive(Debug, Clone)]
pub enum Value {
    /// Plain number
    Number(f64),
    /// Number with a unit
    WithUnit(f64, String),
    /// Currency amount
    Currency(f64, String),
    /// Percentage
    Percent(f64),
    /// Date/time as string representation
    DateTime(String),
    /// Duration
    Duration(f64, String),
    /// Boolean
    Boolean(bool),
    /// Error result
    Error(String),
    /// No result (comments, headers, empty lines)
    Empty,
}

impl Value {
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    /// Extract numeric value if possible
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::WithUnit(n, _) => Some(*n),
            Value::Currency(n, _) => Some(*n),
            Value::Percent(n) => Some(*n),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", format_number(*n)),
            Value::WithUnit(n, unit) => write!(f, "{} {}", format_number(*n), unit),
            Value::Currency(n, code) => {
                let symbol = currency_symbol(code);
                write!(f, "{} {}", symbol, format_number(*n))
            }
            Value::Percent(n) => write!(f, "{}%", format_number(*n)),
            Value::DateTime(s) => write!(f, "{}", s),
            Value::Duration(n, unit) => write!(f, "{} {}", format_number(*n), unit),
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Error(_) => write!(f, "error"),
            Value::Empty => Ok(()),
        }
    }
}

/// Format number: integer if whole, otherwise up to 2dp trimmed
fn format_number(n: f64) -> String {
    if n == n.floor() && n.abs() < 1e15 {
        let i = n as i64;
        format!("{}", i)
    } else {
        let s = format!("{:.2}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn currency_symbol(code: &str) -> &str {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        "INR" => "₹",
        "KRW" => "₩",
        "BTC" => "₿",
        "BRL" => "R$",
        "TRY" => "₺",
        "ILS" => "₪",
        "PHP" => "₱",
        "RUB" => "₽",
        "THB" => "฿",
        "PLN" => "zł",
        _ => code,
    }
}
