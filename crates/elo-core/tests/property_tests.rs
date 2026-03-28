use elo_core::Session;
use proptest::prelude::*;

/// Strategy to generate arbitrary expression strings
fn arb_expression() -> impl Strategy<Value = String> {
    prop_oneof![
        // Random strings
        "[a-zA-Z0-9 +\\-*/^%().,:=]{0,100}",
        // Number-heavy strings
        "[0-9.eE+\\- ]{1,50}",
        // Operator-heavy strings
        "[0-9+\\-*/^%&|<> ]{1,50}",
        // Unit expressions
        "[0-9. ]+(meter|cm|kg|g|lb|hour|minute|second|day|week|month|year|fahrenheit|celsius|kelvin)[a-z ]*",
        // Function calls
        "(sqrt|cbrt|abs|sin|cos|tan|round|ceil|floor|ln|log|fact)\\([0-9.\\-]*\\)",
        // Percentage expressions
        "[0-9.]+% (of|on|off) [0-9.]+",
        // Session tokens
        "(prev|sum|total|avg|average|now|time|today|tomorrow|yesterday)",
        // Hex/bin/oct
        "0[xXbBoO][0-9a-fA-F]+",
        // Empty and whitespace
        "[ \t]*",
        // Comments and headers
        "(#|//).*",
        // Labels
        "[A-Za-z]+: [0-9+\\-*/ ]+",
        // Assignments
        "[a-z]+ = [0-9+\\-*/ ]+",
        // Conversions
        "[0-9.]+ [a-z]+ (in|to|as) [a-z]+",
        // Deeply nested parens
        "\\({0,10}[0-9]+\\){0,10}",
    ]
}

proptest! {
    /// The engine must never panic on any input
    #[test]
    fn no_panic_on_arbitrary_input(input in arb_expression()) {
        let mut session = Session::new();
        // This must not panic
        let _result = session.eval_line(&input);
    }

    /// Evaluating the same expression twice must give the same result
    #[test]
    fn deterministic_evaluation(input in arb_expression()) {
        let mut s1 = Session::new();
        let mut s2 = Session::new();
        let r1 = s1.eval_line(&input);
        let r2 = s2.eval_line(&input);
        prop_assert_eq!(r1.display, r2.display);
    }

    /// Numbers should roundtrip: parsing a number and displaying it should be stable
    #[test]
    fn number_roundtrip(n in -1e12f64..1e12f64) {
        let input = format!("{}", n);
        let mut session = Session::new();
        let result = session.eval_line(&input);
        if let Some(got) = result.value.as_number() {
            let diff = (got - n).abs();
            let tol = n.abs() * 1e-10 + 1e-10;
            prop_assert!(diff < tol, "roundtrip failed: input={}, got={}", n, got);
        }
    }

    /// Integer arithmetic should be exact
    #[test]
    fn integer_add_exact(a in -10000i64..10000, b in -10000i64..10000) {
        let input = format!("{} + {}", a, b);
        let mut session = Session::new();
        let result = session.eval_line(&input);
        let expected = (a + b) as f64;
        prop_assert_eq!(result.value.as_number(), Some(expected),
            "failed: {} + {} expected {} got {:?}", a, b, expected, result.value.as_number());
    }

    /// Integer multiplication should be exact
    #[test]
    fn integer_mul_exact(a in -1000i64..1000, b in -1000i64..1000) {
        let input = format!("{} * {}", a, b);
        let mut session = Session::new();
        let result = session.eval_line(&input);
        let expected = (a * b) as f64;
        prop_assert_eq!(result.value.as_number(), Some(expected));
    }

    /// Negation is involutive: -(-x) == x
    #[test]
    fn double_negation(n in -1e9f64..1e9f64) {
        let input = format!("-(-{})", n);
        // Only test clean integers to avoid formatting issues
        if n == n.floor() && n.abs() < 1e12 {
            let mut session = Session::new();
            let result = session.eval_line(&input);
            if let Some(got) = result.value.as_number() {
                let diff = (got - n).abs();
                prop_assert!(diff < 1e-6, "--{} = {} not {}", n, got, n);
            }
        }
    }

    /// Division by non-zero should satisfy: (a / b) * b ≈ a
    #[test]
    fn division_inverse(a in -1000.0f64..1000.0, b in prop::num::f64::NORMAL.prop_filter("non-zero", |b| b.abs() > 0.001)) {
        let input = format!("({} / {}) * {}", a, b, b);
        let mut session = Session::new();
        let result = session.eval_line(&input);
        if let Some(got) = result.value.as_number() {
            let tol = a.abs() * 1e-6 + 1e-6;
            prop_assert!((got - a).abs() < tol,
                "({} / {}) * {} = {}, expected ≈{}", a, b, b, got, a);
        }
    }

    /// Empty and whitespace input should produce empty output without panic
    #[test]
    fn whitespace_input(spaces in "[ \t\n]{0,20}") {
        let mut session = Session::new();
        let result = session.eval_line(&spaces);
        prop_assert!(result.value.is_empty() || result.display.is_empty() || result.display == "error");
    }
}

/// Targeted fuzz: deeply nested expressions
#[test]
fn no_panic_nested_parens() {
    let mut session = Session::new();
    for depth in 0..50 {
        let open: String = "(".repeat(depth);
        let close: String = ")".repeat(depth);
        let input = format!("{}42{}", open, close);
        let _result = session.eval_line(&input);
    }
}

/// Targeted fuzz: very long expressions
#[test]
fn no_panic_long_expression() {
    let mut session = Session::new();
    let input = (0..200)
        .map(|i| format!("{}", i))
        .collect::<Vec<_>>()
        .join(" + ");
    let result = session.eval_line(&input);
    // 0+1+2+...+199 = 199*200/2 = 19900
    assert_eq!(result.value.as_number(), Some(19900.0));
}

/// Targeted fuzz: chained conversions
#[test]
fn no_panic_various_conversions() {
    let cases = [
        "1 meter in cm",
        "1 meter in km",
        "1 meter in inches",
        "1 meter in feet",
        "1 kg in g",
        "1 kg in pounds",
        "1 hour in seconds",
        "1 hour in days",
        "0 celsius in fahrenheit",
        "100 celsius in fahrenheit",
        "1 liter in ml",
        "1 mile in km",
    ];
    let mut session = Session::new();
    for case in &cases {
        let result = session.eval_line(case);
        assert!(
            !result.value.is_error(),
            "error for '{}': {}",
            case,
            result.display
        );
    }
}

/// Targeted: known tricky inputs from numi-cli behavior
#[test]
fn no_panic_tricky_inputs() {
    let cases = [
        "1 +",
        "* 5",
        "/ 0",
        "+ +",
        "- -",
        "(((",
        ")))",
        "",
        "   ",
        "0xFF & 0x0F",
        "pi * * 2",
        "sqrt(sqrt(sqrt(2)))",
        "1e999",
        "now + now",
        "today - today",
        "abc def ghi",
        "1 meter 2 meter 3 meter",
        "100% of 0",
        "0% of what is 0",
        "sum",
        "prev",
        "avg", // no previous values
    ];
    let mut session = Session::new();
    for case in &cases {
        let _result = session.eval_line(case); // must not panic
    }
}
