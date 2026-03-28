use crate::value::Value;

/// Format a value for display, handling special format markers
pub fn format_value(value: &Value) -> String {
    match value {
        Value::WithUnit(n, unit) if unit == "__hex__" => {
            format!("0x{:x}", *n as i64)
        }
        Value::WithUnit(n, unit) if unit == "__binary__" => {
            format!("0b{:b}", *n as i64)
        }
        Value::WithUnit(n, unit) if unit == "__octal__" => {
            format!("0o{:o}", *n as i64)
        }
        Value::WithUnit(n, unit) if unit == "__sci__" => format_scientific(*n),
        _ => value.to_string(),
    }
}

/// Format a number in scientific notation
fn format_scientific(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    let exp = n.abs().log10().floor() as i32;
    let mantissa = n / 10f64.powi(exp);

    if (mantissa - mantissa.round()).abs() < 1e-10 {
        format!("{}e{}", mantissa.round() as i64, exp)
    } else {
        format!("{:.2}e{}", mantissa, exp)
    }
}

/// Format a number with locale-appropriate separators
pub fn format_number_with_locale(n: f64, _locale: &str) -> String {
    // For now, default formatting
    if n == n.floor() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.2}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_format() {
        let val = Value::WithUnit(10.0, "__hex__".to_string());
        assert_eq!(format_value(&val), "0xa");
    }

    #[test]
    fn test_binary_format() {
        let val = Value::WithUnit(10.0, "__binary__".to_string());
        assert_eq!(format_value(&val), "0b1010");
    }

    #[test]
    fn test_octal_format() {
        let val = Value::WithUnit(10.0, "__octal__".to_string());
        assert_eq!(format_value(&val), "0o12");
    }

    #[test]
    fn test_scientific_format() {
        let val = Value::WithUnit(100.0, "__sci__".to_string());
        assert_eq!(format_value(&val), "1e2");
    }
}
