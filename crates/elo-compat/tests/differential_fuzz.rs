use std::process::Command;

fn numi_eval(expr: &str) -> Option<String> {
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

fn elo_eval(expr: &str) -> String {
    let mut session = elo_core::Session::new();
    session.eval_line(expr).display
}

/// Normalize for comparison: strip trailing zeros, allow ±1% tolerance
fn semantic_match(numi: &str, elo: &str) -> bool {
    if numi == elo {
        return true;
    }

    // Normalize trailing zeros and decimal places
    let nn = normalize_num_str(numi);
    let ne = normalize_num_str(elo);
    if nn == ne {
        return true;
    }

    // Try extracting numbers and comparing with tolerance
    if let (Some(a), Some(b)) = (try_parse_number(numi), try_parse_number(elo)) {
        let tol = a.abs() * 0.01 + 0.01;
        if (a - b).abs() < tol {
            return true;
        }
    }

    false
}

fn normalize_num_str(s: &str) -> String {
    let s = s.trim();
    // Split at first space to separate number from unit
    let (num_part, suffix) = if let Some(idx) = s.find(|c: char| {
        !c.is_ascii_digit() && c != '.' && c != '-' && c != '+' && c != 'e' && c != 'E'
    }) {
        (&s[..idx], s[idx..].trim())
    } else {
        (s, "")
    };

    if let Ok(n) = num_part.parse::<f64>() {
        let formatted = if n == n.floor() && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            let s = format!("{:.6}", n);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        };
        if suffix.is_empty() {
            formatted
        } else {
            format!("{} {}", formatted, suffix)
        }
    } else {
        s.to_string()
    }
}

fn try_parse_number(s: &str) -> Option<f64> {
    let s = s.trim();
    // Try direct parse
    if let Ok(n) = s.parse::<f64>() {
        return Some(n);
    }
    // Try stripping suffix
    let num_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(s.len());
    s[..num_end].parse().ok()
}

/// Generate expressions from grammar fragments and compare.
/// `strict` mode fails on any mismatch. Non-strict allows elo to succeed where numi errors.
fn fuzz_category(name: &str, expressions: &[String]) {
    fuzz_category_inner(name, expressions, false);
}

fn _fuzz_category_strict(name: &str, expressions: &[String]) {
    fuzz_category_inner(name, expressions, true);
}

fn fuzz_category_inner(name: &str, expressions: &[String], strict: bool) {
    let mut mismatches = Vec::new();
    let mut elo_extra = 0; // cases where elo succeeds but numi errors (acceptable)

    for expr in expressions {
        let numi = numi_eval(expr);
        let elo = elo_eval(expr);

        let ok = match &numi {
            Some(n) => semantic_match(n, &elo),
            None => {
                if elo.is_empty() || elo.contains("error") {
                    true
                } else if !strict {
                    // Elo produces a result where numi errors - acceptable in non-strict mode
                    elo_extra += 1;
                    true
                } else {
                    false
                }
            }
        };

        if !ok {
            mismatches.push(format!("  '{}': numi={:?}, elo='{}'", expr, numi, elo,));
        }
    }

    if !mismatches.is_empty() {
        eprintln!(
            "\n[{}] {} mismatches out of {}:",
            name,
            mismatches.len(),
            expressions.len()
        );
        for m in &mismatches {
            eprintln!("{}", m);
        }
    } else {
        let extra = if elo_extra > 0 {
            format!(" ({} elo-extra)", elo_extra)
        } else {
            String::new()
        };
        eprintln!(
            "[{}] {}/{} passed{}",
            name,
            expressions.len(),
            expressions.len(),
            extra
        );
    }

    assert!(
        mismatches.is_empty(),
        "{}: {} mismatches found (see stderr output)",
        name,
        mismatches.len()
    );
}

// === Fuzz: Operator Precedence ===

#[test]
#[ignore]
fn fuzz_operator_precedence() {
    let mut exprs = Vec::new();
    let nums = [1, 2, 3, 5, 7, 10];
    let ops = ["+", "-", "*", "/"];

    for &a in &nums {
        for &op1 in &ops {
            for &b in &nums {
                for &op2 in &ops {
                    for &c in &nums {
                        if op1 == "/" && b == 0 || op2 == "/" && c == 0 {
                            continue;
                        }
                        exprs.push(format!("{} {} {} {} {}", a, op1, b, op2, c));
                    }
                }
            }
        }
    }
    // Sample a subset to keep test time reasonable
    let subset: Vec<String> = exprs.into_iter().step_by(17).collect();
    fuzz_category("operator_precedence", &subset);
}

// === Fuzz: Unary Signs ===

#[test]
#[ignore]
fn fuzz_unary_signs() {
    let mut exprs = Vec::new();
    let nums = [0, 1, 5, 10, 42, 100];
    for &n in &nums {
        exprs.push(format!("-{}", n));
        exprs.push(format!("--{}", n)); // -(-n)
        exprs.push(format!("-{} + {}", n, n));
        exprs.push(format!("{} + -{}", n, n));
        exprs.push(format!("-{} * -{}", n, n));
        exprs.push(format!("(-{})", n));
    }
    fuzz_category("unary_signs", &exprs);
}

// === Fuzz: Scale Suffixes ===

#[test]
#[ignore]
fn fuzz_scale_suffixes() {
    let mut exprs = Vec::new();
    let nums = ["1", "2", "5", "10", "100"];
    let scales = ["k", "M", "billion"];
    for &n in &nums {
        for &s in &scales {
            if s == "billion" {
                exprs.push(format!("{} {}", n, s));
            } else {
                exprs.push(format!("{}{}", n, s));
            }
        }
    }
    // Scale + arithmetic
    exprs.push("1k + 500".to_string());
    exprs.push("2M - 1M".to_string());
    exprs.push("1k * 3".to_string());
    fuzz_category("scale_suffixes", &exprs);
}

// === Fuzz: Nested Function Calls ===

#[test]
#[ignore]
fn fuzz_nested_functions() {
    let mut exprs = Vec::new();
    let fns = ["sqrt", "abs", "round", "ceil", "floor"];
    let args = ["0", "1", "2", "4", "9", "16", "25", "100", "-1", "-5"];

    for &f in &fns {
        for &a in &args {
            exprs.push(format!("{}({})", f, a));
        }
    }
    // Nested
    exprs.push("sqrt(sqrt(16))".to_string());
    exprs.push("abs(round(3.7))".to_string());
    exprs.push("ceil(sqrt(2))".to_string());
    exprs.push("floor(sqrt(10))".to_string());
    exprs.push("abs(-sqrt(9))".to_string());

    fuzz_category("nested_functions", &exprs);
}

// === Fuzz: Unit Conversions ===

#[test]
#[ignore]
fn fuzz_unit_conversions() {
    let mut exprs = Vec::new();
    let conversions = [
        ("1", "meter", "cm"),
        ("1", "meter", "km"),
        ("1", "meter", "inches"),
        ("1", "meter", "feet"),
        ("1", "km", "miles"),
        ("1", "kg", "g"),
        ("1", "kg", "pounds"),
        ("1", "hour", "minutes"),
        ("1", "hour", "seconds"),
        ("1", "day", "hours"),
        ("1", "week", "days"),
        ("0", "celsius", "fahrenheit"),
        ("100", "celsius", "fahrenheit"),
        ("32", "fahrenheit", "celsius"),
        ("212", "fahrenheit", "celsius"),
        ("1", "liter", "ml"),
        ("1", "gallon", "liters"),
    ];

    for (n, from, to) in &conversions {
        exprs.push(format!("{} {} in {}", n, from, to));
    }
    fuzz_category("unit_conversions", &exprs);
}

// === Fuzz: Mixed Labels/Comments ===

#[test]
#[ignore]
fn fuzz_labels_comments() {
    let mut exprs = Vec::new();
    let labels = ["Total", "Subtotal", "Tax", "Result", "A"];
    let values = ["5 + 5", "100", "3 * 4", "pi"];
    for &l in &labels {
        for &v in &values {
            exprs.push(format!("{}: {}", l, v));
        }
    }
    // Comments should produce empty
    exprs.push("// hello".to_string());
    exprs.push("// 5 + 5".to_string());
    // Note: # headers excluded from fuzz comparison because numi-cli doesn't support
    // them properly (evaluates trailing numbers instead of treating as headers).
    // Our behavior is intentional per spec. See docs/compatibility-matrix.md.

    fuzz_category("labels_comments", &exprs);
}

// === Fuzz: Percentage Combos ===

#[test]
#[ignore]
fn fuzz_percentage_combos() {
    let mut exprs = Vec::new();
    let percents = [0, 5, 10, 25, 50, 100, 200];
    let bases = [0, 50, 100, 200, 1000];

    for &p in &percents {
        for &b in &bases {
            exprs.push(format!("{}% of {}", p, b));
            exprs.push(format!("{}% on {}", p, b));
            exprs.push(format!("{}% off {}", p, b));
            exprs.push(format!("{} + {}%", b, p));
            exprs.push(format!("{} - {}%", b, p));
        }
    }

    fuzz_category("percentage_combos", &exprs);
}

// === Fuzz: Unit Sequences ===

#[test]
#[ignore]
fn fuzz_unit_sequences() {
    let exprs = vec![
        "1 meter 20 cm".to_string(),
        "2 meter 50 cm".to_string(),
        "5 feet 3 inches".to_string(),
        "6 feet 0 inches".to_string(),
        "1 kg 500 g".to_string(),
        "2 kg 250 g".to_string(),
        "1 hour 30 minutes".to_string(),
        "2 hours 15 minutes".to_string(),
    ];

    fuzz_category("unit_sequences", &exprs);
}
