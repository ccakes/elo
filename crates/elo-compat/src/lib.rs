use std::process::Command;

/// Run an expression through numi-cli and return the result
pub fn numi_eval(expr: &str) -> Option<String> {
    let output = Command::new("numi-cli").arg("--").arg(expr).output().ok()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result == "error" {
            None
        } else {
            Some(result)
        }
    } else {
        None
    }
}

/// Run an expression through elo-core and return the result
pub fn elo_eval(expr: &str) -> String {
    let mut session = elo_core::Session::new();
    let result = session.eval_line(expr);
    result.display
}

/// Compare results between numi-cli and elo-core
#[derive(Debug)]
pub struct ComparisonResult {
    pub input: String,
    pub numi_output: Option<String>,
    pub elo_output: String,
    pub matches: bool,
}

pub fn compare(expr: &str) -> ComparisonResult {
    let numi = numi_eval(expr);
    let elo = elo_eval(expr);

    let matches = match &numi {
        Some(n) => semantic_match(n, &elo),
        None => elo.is_empty() || elo.contains("error"),
    };

    ComparisonResult {
        input: expr.to_string(),
        numi_output: numi,
        elo_output: elo,
        matches,
    }
}

/// Semantic comparison: extracts numeric values and unit suffixes, compares with tolerance.
/// Allows differences in trailing zeros, decimal places, whitespace, and minor time differences.
fn semantic_match(numi: &str, elo: &str) -> bool {
    if numi == elo {
        return true;
    }

    // Date/time comparison: allow up to 2 second difference
    if looks_like_datetime(numi) && looks_like_datetime(elo) {
        return datetime_close(numi, elo);
    }

    // Extract numeric part and suffix from both
    let (numi_num, numi_suffix) = split_number_suffix(numi);
    let (elo_num, elo_suffix) = split_number_suffix(elo);

    // Compare suffixes (normalize whitespace, case-insensitive for units)
    let suffix_match = numi_suffix.trim() == elo_suffix.trim()
        || numi_suffix.trim().eq_ignore_ascii_case(elo_suffix.trim());

    if !suffix_match {
        return false;
    }

    // Compare numeric parts with tolerance
    if let (Ok(a), Ok(b)) = (numi_num.parse::<f64>(), elo_num.parse::<f64>()) {
        let tolerance = (a.abs() * 1e-4).max(1e-10);
        return (a - b).abs() < tolerance;
    }

    // Fallback: normalized string comparison
    let norm_numi = numi.split_whitespace().collect::<Vec<_>>().join(" ");
    let norm_elo = elo.split_whitespace().collect::<Vec<_>>().join(" ");
    norm_numi == norm_elo
}

/// Split a result string into (numeric_part, suffix).
/// Handles formats like "100 cm", "€ 0.86", "3.14", "0xa"
fn split_number_suffix(s: &str) -> (String, String) {
    let s = s.trim();

    // Handle hex/bin/oct format
    if (s.starts_with("0x") || s.starts_with("0b") || s.starts_with("0o"))
        && let Some(val) = parse_prefixed_int(s)
    {
        return (val.to_string(), String::new());
    }

    // Handle currency prefix: "€ 0.86" -> ("0.86", "€")
    if let Some(first_char) = s.chars().next()
        && !first_char.is_ascii_digit()
        && first_char != '-'
        && first_char != '.'
    {
        let rest = s
            .trim_start_matches(|c: char| !c.is_ascii_digit() && c != '-' && c != '.')
            .trim();
        let prefix = s[..s.len() - rest.len()].trim();
        let (num, suffix) = split_trailing_suffix(rest);
        let full_suffix = if suffix.is_empty() {
            prefix.to_string()
        } else {
            format!("{} {}", prefix, suffix)
        };
        return (num, full_suffix);
    }

    split_trailing_suffix(s)
}

fn split_trailing_suffix(s: &str) -> (String, String) {
    // Find where the number ends
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    // Optional sign
    if i < chars.len() && (chars[i] == '-' || chars[i] == '+') {
        i += 1;
    }

    // Digits and decimal point
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',') {
        i += 1;
    }

    // Scientific notation
    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
        i += 1;
        if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
            i += 1;
        }
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
    }

    let num_str = chars[..i].iter().collect::<String>().replace(',', "");
    let suffix = chars[i..].iter().collect::<String>().trim().to_string();

    (num_str, suffix)
}

fn parse_prefixed_int(s: &str) -> Option<i64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        i64::from_str_radix(bin, 2).ok()
    } else if let Some(oct) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        i64::from_str_radix(oct, 8).ok()
    } else {
        None
    }
}

fn looks_like_datetime(s: &str) -> bool {
    // Matches YYYY-MM-DD or YYYY-MM-DD HH:MM:SS
    let s = s.trim();
    if s.len() >= 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        return s[0..4].parse::<u32>().is_ok();
    }
    false
}

fn datetime_close(a: &str, b: &str) -> bool {
    // For date-only (YYYY-MM-DD), must be exact match
    if !a.contains(':') && !b.contains(':') {
        return a.trim() == b.trim();
    }

    // For datetime, allow up to 2 seconds difference
    use chrono::NaiveDateTime;
    let parse = |s: &str| NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M:%S").ok();
    match (parse(a), parse(b)) {
        (Some(da), Some(db)) => {
            let diff = (da - db).num_seconds().abs();
            diff <= 2
        }
        _ => a.trim() == b.trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to assert semantic compat
    fn assert_compat(cases: &[&str]) {
        for case in cases {
            let result = compare(case);
            assert!(
                result.matches,
                "Mismatch for '{}': numi={:?}, elo={}",
                case, result.numi_output, result.elo_output
            );
        }
    }

    #[test]
    #[ignore]
    fn test_arithmetic_compat() {
        assert_compat(&[
            "2 + 2",
            "3 * 4",
            "10 - 3",
            "12 / 4",
            "2 ^ 10",
            "10 mod 3",
            "2 + 3 * 4",
            "(2 + 3) * 4",
            "10 plus 5",
            "10 minus 3",
            "3 times 4",
            "12 divide by 3",
            "10 multiplied by 5",
            "10 without 3",
            "-5 + 3",
            "0 + 0",
            "1000000 * 1000000",
        ]);
    }

    #[test]
    #[ignore]
    fn test_function_compat() {
        assert_compat(&[
            "sqrt(16)",
            "sqrt(2)",
            "cbrt(27)",
            "cbrt(8)",
            "abs(-5)",
            "abs(5)",
            "round(3.7)",
            "round(3.2)",
            "ceil(3.2)",
            "floor(3.8)",
            "sin(0)",
            "cos(0)",
            "ln(e)",
        ]);
    }

    #[test]
    #[ignore]
    fn test_scale_compat() {
        assert_compat(&["1k", "2.5M", "1 billion", "3k", "10M", "2 billion"]);
    }

    #[test]
    #[ignore]
    fn test_bitwise_compat() {
        assert_compat(&["5 & 3", "5 | 3", "5 xor 3", "1 << 3", "8 >> 2", "255 & 15"]);
    }

    #[test]
    #[ignore]
    fn test_format_conversion_compat() {
        assert_compat(&[
            "10 as hex",
            "10 as binary",
            "10 as octal",
            "255 as hex",
            "100 in sci",
        ]);
    }

    #[test]
    #[ignore]
    fn test_percentage_compat() {
        assert_compat(&[
            "50% of 200",
            "10% on 100",
            "10% off 100",
            "100 - 30%",
            "5% of what is 10",
            "100 + 10%",
        ]);
    }

    #[test]
    #[ignore]
    fn test_unit_conversion_compat() {
        assert_compat(&[
            "1 meter in cm",
            "100 cm in inches",
            "72 fahrenheit in celsius",
            "1 kg in pounds",
            "1 hour in minutes",
        ]);
    }

    #[test]
    #[ignore]
    fn test_constants_compat() {
        assert_compat(&["pi", "e"]);
    }

    #[test]
    #[ignore]
    fn test_number_format_input_compat() {
        // numi preserves hex/bin/oct format for inputs; we evaluate to decimal
        // Semantic match: 0xff == 255, 0b1010 == 10, 0o17 == 15
        assert_compat(&["0xFF", "0b1010", "0o17"]);
    }

    #[test]
    #[ignore]
    fn test_unit_sequence_compat() {
        assert_compat(&["1 meter 20 cm", "5 feet 3 inches"]);
    }

    #[test]
    #[ignore]
    fn test_cross_unit_arithmetic_compat() {
        assert_compat(&["5 meter + 200 cm", "3 kg + 500 g"]);
    }

    #[test]
    #[ignore]
    fn test_date_keyword_compat() {
        assert_compat(&["today", "tomorrow", "yesterday"]);
    }

    #[test]
    #[ignore]
    fn test_date_arithmetic_compat() {
        assert_compat(&["today + 1 week", "tomorrow + 2 days"]);
    }

    #[test]
    #[ignore]
    fn test_timezone_compat() {
        assert_compat(&[
            "time in UTC",
            "time in America/New_York",
            "time in Europe/London",
            "time in Asia/Tokyo",
        ]);
    }

    // Internal tests for semantic_match
    #[test]
    fn test_semantic_match_trailing_zeros() {
        assert!(semantic_match("4.00", "4"));
        assert!(semantic_match("100.00", "100"));
        assert!(semantic_match("2.20", "2.2"));
        assert!(semantic_match("2500000.0", "2500000"));
    }

    #[test]
    fn test_semantic_match_units() {
        assert!(semantic_match("100 cm", "100 cm"));
        assert!(semantic_match("2.20 lb", "2.2 lb"));
        assert!(semantic_match("39.37 ″", "39.37 ″"));
    }

    #[test]
    fn test_semantic_match_hex() {
        assert!(semantic_match("0xff", "255"));
        assert!(semantic_match("0b1010", "10"));
        assert!(semantic_match("0o17", "15"));
    }

    #[test]
    fn test_semantic_match_exact() {
        assert!(semantic_match("42", "42"));
        assert!(semantic_match("3.14", "3.14"));
    }
}
