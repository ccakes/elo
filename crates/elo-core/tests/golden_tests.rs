use elo_core::{RateStore, Session};

/// Normalize a number string for comparison: strip trailing zeros, trailing dot
fn normalize_number(s: &str) -> String {
    // Try parsing as f64 and reformatting
    if let Ok(n) = s.parse::<f64>() {
        if n == n.floor() && n.abs() < 1e15 {
            return format!("{}", n as i64);
        }
        let formatted = format!("{:.6}", n);
        return formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
    }
    s.to_string()
}

/// Normalize a result string for fuzzy comparison
fn normalize_result(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() || s == "error" {
        return s.to_string();
    }

    // Split into parts (number + optional suffix)
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    if parts.len() == 2 {
        let num = normalize_number(parts[0]);
        format!("{} {}", num, parts[1])
    } else {
        // Could be just a number, or a prefixed currency like "€ 100"
        normalize_number(s)
    }
}

fn results_match(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }

    let ne = normalize_result(expected);
    let na = normalize_result(actual);
    if ne == na {
        return true;
    }

    // Try numeric comparison with tolerance
    if let (Ok(a), Ok(b)) = (
        extract_number(expected).parse::<f64>(),
        extract_number(actual).parse::<f64>(),
    ) {
        let tol = (a.abs() * 1e-2).max(0.01); // 1% tolerance
        if (a - b).abs() < tol {
            // Also check suffix matches
            let sa = extract_suffix(expected);
            let sb = extract_suffix(actual);
            return sa == sb || sa.eq_ignore_ascii_case(&sb);
        }
    }

    false
}

/// Check if a CURRENCY result matches: correct symbol/code prefix, numeric value present.
fn currency_result_matches(expected_currency: &str, actual: &str) -> bool {
    let actual = actual.trim();
    if actual.is_empty() || actual == "error" || actual.contains("requires rates") {
        return false;
    }

    // Expected format: "CURRENCY EUR" means output should be "€ <number>" or "<number> EUR"
    // Check that the output contains the expected currency symbol or code
    let symbol = currency_code_to_symbol(expected_currency);

    let has_currency = actual.starts_with(&format!("{} ", symbol))
        || actual.starts_with(&format!("{} ", expected_currency))
        || actual.ends_with(&format!(" {}", expected_currency));

    if !has_currency {
        return false;
    }

    // Verify there's a numeric value
    let num_str = actual
        .replace(symbol, "")
        .replace(expected_currency, "")
        .trim()
        .to_string();
    num_str.parse::<f64>().is_ok()
}

fn currency_code_to_symbol(code: &str) -> &str {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        "INR" => "₹",
        "KRW" => "₩",
        "BTC" => "₿",
        _ => code,
    }
}

fn extract_number(s: &str) -> String {
    let s = s.trim();
    let mut end = 0;
    let chars: Vec<char> = s.chars().collect();
    if end < chars.len() && (chars[end] == '-' || chars[end] == '+') {
        end += 1;
    }
    while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
        end += 1;
    }
    chars[..end].iter().collect()
}

fn extract_suffix(s: &str) -> String {
    let num = extract_number(s);
    s.trim()[num.len()..].trim().to_string()
}

#[test]
fn test_golden_corpus() {
    let corpus = include_str!("../../../tests/goldens/expressions.txt");

    let rates = RateStore::load();

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failures: Vec<String> = Vec::new();

    for line in corpus.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let input = parts[0].trim();
        let expected = parts[1].trim();
        let _tags = if parts.len() > 2 { parts[2].trim() } else { "" };

        // Skip time-sensitive tests
        if expected == "DATE" || expected == "DATETIME" {
            skipped += 1;
            continue;
        }

        // Handle CURRENCY tests: format-only check (rates change over time)
        if expected.starts_with("CURRENCY ") {
            if rates.is_none() {
                skipped += 1;
                continue;
            }
            let expected_currency = expected.strip_prefix("CURRENCY ").unwrap().trim();
            let mut session = Session::with_rates(rates.clone());
            let result = session.eval_line(input);
            let actual = result.display.trim().to_string();

            if currency_result_matches(expected_currency, &actual) {
                passed += 1;
            } else {
                failed += 1;
                failures.push(format!(
                    "  '{}': expected currency {}, got '{}'",
                    input, expected_currency, actual
                ));
            }
            continue;
        }

        let mut session = Session::with_rates(rates.clone());
        let result = session.eval_line(input);
        let actual = result.display.trim().to_string();

        // Empty expected means we expect empty output
        if expected.is_empty() {
            if actual.is_empty() {
                passed += 1;
            } else {
                failed += 1;
                failures.push(format!("  '{}': expected empty, got '{}'", input, actual));
            }
            continue;
        }

        // Error expected
        if expected == "error" {
            if actual == "error" || actual.contains("error") || actual.is_empty() {
                passed += 1;
            } else {
                failed += 1;
                failures.push(format!("  '{}': expected error, got '{}'", input, actual));
            }
            continue;
        }

        if results_match(expected, &actual) {
            passed += 1;
        } else {
            failed += 1;
            failures.push(format!(
                "  '{}': expected '{}', got '{}'",
                input, expected, actual
            ));
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "\n=== Golden test failures ({}/{} failed) ===",
            failed,
            passed + failed
        );
        for f in &failures {
            eprintln!("{}", f);
        }
    }

    eprintln!(
        "\nGolden tests: {} passed, {} failed, {} skipped (time-sensitive/no-rates)",
        passed, failed, skipped
    );

    assert!(failed == 0, "{} golden tests failed (see above)", failed);
}
