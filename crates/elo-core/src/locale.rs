/// Locale configuration for number formatting
#[derive(Debug, Clone)]
pub struct Locale {
    /// Decimal separator character
    pub decimal_sep: char,
    /// Thousands grouping separator (None = no grouping)
    pub thousands_sep: Option<char>,
}

impl Locale {
    /// English/US locale: 1,234.56
    pub fn en() -> Self {
        Self {
            decimal_sep: '.',
            thousands_sep: Some(','),
        }
    }

    /// German/European locale: 1.234,56
    pub fn de() -> Self {
        Self {
            decimal_sep: ',',
            thousands_sep: Some('.'),
        }
    }

    /// French locale: 1 234,56
    pub fn fr() -> Self {
        Self {
            decimal_sep: ',',
            thousands_sep: Some(' '),
        }
    }

    /// No grouping, period decimal (programming-style)
    pub fn c() -> Self {
        Self {
            decimal_sep: '.',
            thousands_sep: None,
        }
    }

    /// Parse a locale identifier (BCP-47 or common name)
    pub fn from_identifier(id: &str) -> Self {
        match id.to_lowercase().as_str() {
            "en" | "en_us" | "en-us" | "en_gb" | "en-gb" | "en_au" | "en-au" => Self::en(),
            "de" | "de_de" | "de-de" | "de_at" | "de-at" | "de_ch" | "de-ch" => Self::de(),
            "fr" | "fr_fr" | "fr-fr" | "fr_ca" | "fr-ca" => Self::fr(),
            "es" | "es_es" | "es-es" | "it" | "it_it" | "it-it" | "pt" | "pt_br" | "pt-br" => {
                Self::de()
            } // comma decimal
            "c" | "posix" => Self::c(),
            _ => Self::en(), // default
        }
    }

    /// Try to detect locale from system environment
    pub fn from_system() -> Self {
        if let Ok(lang) = std::env::var("LANG") {
            let id = lang.split('.').next().unwrap_or("en");
            return Self::from_identifier(id);
        }
        if let Ok(lang) = std::env::var("LC_NUMERIC") {
            let id = lang.split('.').next().unwrap_or("en");
            return Self::from_identifier(id);
        }
        Self::en()
    }

    /// Format a number according to this locale
    pub fn format_number(&self, n: f64) -> String {
        if n == n.floor() && n.abs() < 1e15 {
            let i = n as i64;
            let s = format!("{}", i);
            if let Some(sep) = self.thousands_sep {
                add_thousands_sep(&s, sep)
            } else {
                s
            }
        } else {
            let s = format!("{:.2}", n);
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            if self.decimal_sep != '.' {
                trimmed.replace('.', &self.decimal_sep.to_string())
            } else {
                trimmed.to_string()
            }
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::en()
    }
}

fn add_thousands_sep(s: &str, sep: char) -> String {
    let (neg, digits) = if let Some(stripped) = s.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, s)
    };

    if digits.len() <= 3 {
        return s.to_string();
    }

    let mut result = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(sep);
        }
        result.push(c);
    }
    let grouped: String = result.chars().rev().collect();
    if neg {
        format!("-{}", grouped)
    } else {
        grouped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_en_locale_integer() {
        let locale = Locale::en();
        assert_eq!(locale.format_number(1234567.0), "1,234,567");
        assert_eq!(locale.format_number(42.0), "42");
        assert_eq!(locale.format_number(-1000.0), "-1,000");
    }

    #[test]
    fn test_en_locale_decimal() {
        let locale = Locale::en();
        #[allow(clippy::approx_constant)]
        let n = 3.14;
        assert_eq!(locale.format_number(n), "3.14");
    }

    #[test]
    fn test_de_locale_decimal() {
        let locale = Locale::de();
        #[allow(clippy::approx_constant)]
        let n = 3.14;
        assert_eq!(locale.format_number(n), "3,14");
    }

    #[test]
    fn test_c_locale_no_grouping() {
        let locale = Locale::c();
        assert_eq!(locale.format_number(1234567.0), "1234567");
    }

    #[test]
    fn test_from_identifier() {
        let en = Locale::from_identifier("en_US");
        assert_eq!(en.decimal_sep, '.');

        let de = Locale::from_identifier("de_DE");
        assert_eq!(de.decimal_sep, ',');
    }
}
