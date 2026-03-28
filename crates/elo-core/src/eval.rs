use std::collections::HashMap;

use chrono::{Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

use crate::ast::*;
use crate::value::Value;

/// Evaluation context holding variables, previous results, etc.
pub struct EvalContext {
    pub variables: HashMap<String, Value>,
    /// Results from previous lines in the current block (for sum/avg)
    pub block_results: Vec<f64>,
    /// Previous line result (for prev)
    pub prev_result: Option<Value>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            block_results: Vec::new(),
            prev_result: None,
        }
    }

    /// Start a new block (on empty line)
    pub fn new_block(&mut self) {
        self.block_results.clear();
    }

    /// Record a line result
    pub fn record_result(&mut self, value: &Value) {
        if let Some(n) = value.as_number() {
            self.block_results.push(n);
        }
        self.prev_result = Some(value.clone());
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn eval_line(line: &Line, ctx: &mut EvalContext) -> Value {
    match line {
        Line::Empty => {
            ctx.new_block();
            Value::Empty
        }
        Line::Comment(_) => Value::Empty,
        Line::Header(_) => Value::Empty,
        Line::Expression { expr, .. } => {
            let value = eval_expr(expr, ctx);
            ctx.record_result(&value);
            value
        }
        Line::Assignment { name, expr, .. } => {
            let value = eval_expr(expr, ctx);
            ctx.variables.insert(name.clone(), value.clone());
            ctx.record_result(&value);
            value
        }
    }
}

pub fn eval_expr(expr: &Expr, ctx: &EvalContext) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::HexLiteral(n) => Value::Number(*n as f64),
        Expr::BinLiteral(n) => Value::Number(*n as f64),
        Expr::OctLiteral(n) => Value::Number(*n as f64),
        Expr::SciLiteral(n, _) => Value::Number(*n),
        Expr::Percentage(n) => Value::Percent(*n),

        Expr::Ident(name) => eval_ident(name, ctx),

        Expr::BinaryOp { op, left, right } => eval_binary(*op, left, right, ctx),
        Expr::UnaryOp { op, operand } => eval_unary(*op, operand, ctx),

        Expr::FuncCall { name, args } => eval_function(name, args, ctx),

        Expr::Conversion { expr, target } => eval_conversion(expr, target, ctx),

        Expr::PercentOf { percent, base } => {
            let pct = eval_expr(percent, ctx);
            let base = eval_expr(base, ctx);
            match (pct.as_number(), base.as_number()) {
                (Some(p), Some(b)) => {
                    let result = p / 100.0 * b;
                    match base {
                        Value::Currency(_, ref code) => Value::Currency(result, code.clone()),
                        Value::WithUnit(_, ref unit) => Value::WithUnit(result, unit.clone()),
                        _ => Value::Number(result),
                    }
                }
                _ => Value::Error("invalid percent-of".to_string()),
            }
        }
        Expr::PercentOn { percent, base } => {
            let pct = eval_expr(percent, ctx);
            let base_val = eval_expr(base, ctx);
            match (pct.as_number(), base_val.as_number()) {
                (Some(p), Some(b)) => {
                    let result = b * (1.0 + p / 100.0);
                    match base_val {
                        Value::Currency(_, ref code) => Value::Currency(result, code.clone()),
                        Value::WithUnit(_, ref unit) => Value::WithUnit(result, unit.clone()),
                        _ => Value::Number(result),
                    }
                }
                _ => Value::Error("invalid percent-on".to_string()),
            }
        }
        Expr::PercentOff { percent, base } => {
            let pct = eval_expr(percent, ctx);
            let base_val = eval_expr(base, ctx);
            match (pct.as_number(), base_val.as_number()) {
                (Some(p), Some(b)) => {
                    let result = b * (1.0 - p / 100.0);
                    match base_val {
                        Value::Currency(_, ref code) => Value::Currency(result, code.clone()),
                        Value::WithUnit(_, ref unit) => Value::WithUnit(result, unit.clone()),
                        _ => Value::Number(result),
                    }
                }
                _ => Value::Error("invalid percent-off".to_string()),
            }
        }
        Expr::PercentOfWhatIs { percent, result } => {
            let pct = eval_expr(percent, ctx);
            let res = eval_expr(result, ctx);
            match (pct.as_number(), res.as_number()) {
                (Some(p), Some(r)) => {
                    if p == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Number(r / (p / 100.0))
                    }
                }
                _ => Value::Error("invalid inverse percent".to_string()),
            }
        }

        Expr::Prev => {
            ctx.prev_result.clone().unwrap_or(Value::Error("no previous result".to_string()))
        }
        Expr::Sum => {
            if ctx.block_results.is_empty() {
                Value::Error("no values to sum".to_string())
            } else {
                Value::Number(ctx.block_results.iter().sum())
            }
        }
        Expr::Avg => {
            if ctx.block_results.is_empty() {
                Value::Error("no values to average".to_string())
            } else {
                let sum: f64 = ctx.block_results.iter().sum();
                Value::Number(sum / ctx.block_results.len() as f64)
            }
        }

        Expr::Scaled(n, scale) => Value::Number(*n * scale.multiplier()),

        Expr::WithUnit(expr, unit) => {
            let val = eval_expr(expr, ctx);
            match val {
                Value::Number(n) => {
                    if is_currency_unit(unit) {
                        Value::Currency(n, normalize_currency(unit))
                    } else {
                        let display = display_unit(unit);
                        Value::WithUnit(n, display)
                    }
                }
                _ => val,
            }
        }

        Expr::ImplicitMul(left, right) => {
            let l = eval_expr(left, ctx);
            let r = eval_expr(right, ctx);
            match (l.as_number(), r.as_number()) {
                (Some(a), Some(b)) => Value::Number(a * b),
                _ => Value::Error("invalid implicit multiplication".to_string()),
            }
        }

        Expr::Paren(inner) => eval_expr(inner, ctx),

        Expr::UnitSequence(parts) => eval_unit_sequence(parts, ctx),

        Expr::Today => {
            let today = Local::now().date_naive();
            Value::DateTime(today.format("%Y-%m-%d").to_string())
        }
        Expr::Tomorrow => {
            let tomorrow = Local::now().date_naive() + Duration::days(1);
            Value::DateTime(tomorrow.format("%Y-%m-%d").to_string())
        }
        Expr::Yesterday => {
            let yesterday = Local::now().date_naive() - Duration::days(1);
            Value::DateTime(yesterday.format("%Y-%m-%d").to_string())
        }
    }
}

/// Evaluate a unit sequence like [("1", "meter"), ("20", "cm")]
/// Converts all parts to the last unit and sums.
fn eval_unit_sequence(parts: &[(Box<Expr>, String)], ctx: &EvalContext) -> Value {
    if parts.is_empty() {
        return Value::Error("empty unit sequence".to_string());
    }

    let target_unit_name = &parts.last().unwrap().1;
    let target_unit = elo_data::units::find_unit(target_unit_name);
    if target_unit.is_none() {
        return Value::Error(format!("unknown unit: {}", target_unit_name));
    }
    let target = target_unit.unwrap();

    let mut total = 0.0;
    for (expr, unit_name) in parts {
        let val = eval_expr(expr, ctx);
        let n = match val.as_number() {
            Some(n) => n,
            None => return Value::Error("non-numeric in unit sequence".to_string()),
        };

        if let Some(from_unit) = elo_data::units::find_unit(unit_name) {
            if let Some(converted) = elo_data::units::convert(n, from_unit, target) {
                total += converted;
            } else {
                return Value::Error(format!("cannot convert {} to {}", unit_name, target_unit_name));
            }
        } else {
            return Value::Error(format!("unknown unit: {}", unit_name));
        }
    }

    let display = display_unit(target.id);
    Value::WithUnit(total, display)
}

fn eval_ident(name: &str, ctx: &EvalContext) -> Value {
    // Constants
    match name {
        "pi" | "PI" => return Value::Number(std::f64::consts::PI),
        "e" => return Value::Number(std::f64::consts::E),
        "now" | "time" => {
            let now = Local::now();
            return Value::DateTime(now.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        _ => {}
    }

    // Check variables
    if let Some(val) = ctx.variables.get(name) {
        return val.clone();
    }

    // Unknown identifier
    Value::Error(format!("unknown identifier: {}", name))
}

fn eval_binary(op: BinOp, left: &Expr, right: &Expr, ctx: &EvalContext) -> Value {
    let lv = eval_expr(left, ctx);
    let rv = eval_expr(right, ctx);

    // Date/time arithmetic: DateTime +/- duration
    if matches!(op, BinOp::Add | BinOp::Sub) {
        if let Some(result) = try_datetime_arithmetic(&lv, &rv, op) {
            return result;
        }
    }

    // Handle unit arithmetic: same-unit values can be added/subtracted
    match (&lv, &rv) {
        (Value::WithUnit(a, unit_a), Value::WithUnit(b, unit_b)) if unit_a == unit_b => {
            match op {
                BinOp::Add => return Value::WithUnit(a + b, unit_a.clone()),
                BinOp::Sub => return Value::WithUnit(a - b, unit_a.clone()),
                _ => {}
            }
        }
        (Value::Currency(a, code_a), Value::Currency(b, code_b)) if code_a == code_b => {
            match op {
                BinOp::Add => return Value::Currency(a + b, code_a.clone()),
                BinOp::Sub => return Value::Currency(a - b, code_a.clone()),
                _ => {}
            }
        }
        _ => {}
    }

    // Cross-unit arithmetic: convert left to right's unit if same dimension (numi behavior)
    if matches!(op, BinOp::Add | BinOp::Sub) {
        if let (Value::WithUnit(a, unit_a), Value::WithUnit(b, unit_b)) = (&lv, &rv) {
            if unit_a != unit_b {
                if let Some(converted) = try_unit_convert(*a, unit_a, unit_b) {
                    if let Value::WithUnit(a_converted, _) = converted {
                        let result = match op {
                            BinOp::Add => a_converted + b,
                            BinOp::Sub => a_converted - b,
                            _ => unreachable!(),
                        };
                        return Value::WithUnit(result, unit_b.clone());
                    }
                }
            }
        }
    }

    match (lv.as_number(), rv.as_number()) {
        (Some(a), Some(b)) => {
            let result = match op {
                BinOp::Add => a + b,
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => {
                    if b == 0.0 {
                        return Value::Error("division by zero".to_string());
                    }
                    a / b
                }
                BinOp::Pow => a.powf(b),
                BinOp::Mod => {
                    if b == 0.0 {
                        return Value::Error("modulo by zero".to_string());
                    }
                    a % b
                }
                BinOp::BitAnd => ((a as i64) & (b as i64)) as f64,
                BinOp::BitOr => ((a as i64) | (b as i64)) as f64,
                BinOp::BitXor => ((a as i64) ^ (b as i64)) as f64,
                BinOp::Shl => ((a as i64) << (b as i64)) as f64,
                BinOp::Shr => ((a as i64) >> (b as i64)) as f64,
            };

            // Preserve units from left operand for mul/div where appropriate
            match (&lv, op) {
                (Value::WithUnit(_, unit), BinOp::Mul | BinOp::Div) => {
                    Value::WithUnit(result, unit.clone())
                }
                (Value::Currency(_, code), BinOp::Mul | BinOp::Div) => {
                    Value::Currency(result, code.clone())
                }
                _ => Value::Number(result),
            }
        }
        _ => Value::Error("invalid operands".to_string()),
    }
}

/// Try date/time arithmetic: DateTime +/- number with time unit
fn try_datetime_arithmetic(lv: &Value, rv: &Value, op: BinOp) -> Option<Value> {
    let dt_str = match lv {
        Value::DateTime(s) => s,
        _ => return None,
    };

    // Right side must be a duration (WithUnit with a time unit)
    let (amount, time_unit) = match rv {
        Value::WithUnit(n, unit) => (*n, unit.as_str()),
        _ => return None,
    };

    let signed_amount = match op {
        BinOp::Add => amount,
        BinOp::Sub => -amount,
        _ => return None,
    };

    // Parse the datetime from lv
    let dt = parse_datetime_str(dt_str)?;

    let duration = time_unit_to_duration(signed_amount, time_unit)?;
    let result_dt = dt.checked_add_signed(duration)?;

    // If original was date-only (no time component visible), keep date-only format
    if !dt_str.contains(':') {
        Some(Value::DateTime(result_dt.format("%Y-%m-%d").to_string()))
    } else {
        Some(Value::DateTime(result_dt.format("%Y-%m-%d %H:%M:%S").to_string()))
    }
}

fn parse_datetime_str(s: &str) -> Option<NaiveDateTime> {
    // Try full datetime first
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(dt);
    }
    // Try date-only
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d.and_hms_opt(0, 0, 0)?);
    }
    None
}

fn time_unit_to_duration(amount: f64, unit: &str) -> Option<Duration> {
    let seconds = match unit {
        "ms" | "millisecond" | "milliseconds" => amount / 1000.0,
        "s" | "sec" | "second" | "seconds" => amount,
        "min" | "minute" | "minutes" => amount * 60.0,
        "hr" | "hour" | "hours" => amount * 3600.0,
        "day" | "days" => amount * 86400.0,
        "week" | "weeks" | "wk" => amount * 604800.0,
        "month" | "months" | "mo" => amount * 2592000.0,
        "year" | "years" | "yr" => amount * 31536000.0,
        _ => return None,
    };
    Some(Duration::seconds(seconds as i64))
}

fn eval_unary(op: UnaryOp, operand: &Expr, ctx: &EvalContext) -> Value {
    let val = eval_expr(operand, ctx);
    match val.as_number() {
        Some(n) => {
            let result = match op {
                UnaryOp::Neg => -n,
                UnaryOp::Pos => n,
            };
            match val {
                Value::WithUnit(_, ref unit) => Value::WithUnit(result, unit.clone()),
                Value::Currency(_, ref code) => Value::Currency(result, code.clone()),
                _ => Value::Number(result),
            }
        }
        None => Value::Error("invalid operand for unary".to_string()),
    }
}

fn eval_function(name: &str, args: &[Expr], ctx: &EvalContext) -> Value {
    let evaluated: Vec<Value> = args.iter().map(|a| eval_expr(a, ctx)).collect();

    let get_num = |i: usize| -> Option<f64> {
        evaluated.get(i).and_then(|v| v.as_number())
    };

    match name {
        "sqrt" => {
            get_num(0).map(|n| {
                if n < 0.0 {
                    Value::Error("sqrt of negative number".to_string())
                } else {
                    Value::Number(n.sqrt())
                }
            }).unwrap_or(Value::Error("sqrt requires 1 argument".to_string()))
        }
        "cbrt" => {
            get_num(0).map(|n| Value::Number(n.cbrt()))
                .unwrap_or(Value::Error("cbrt requires 1 argument".to_string()))
        }
        "root" => {
            match (get_num(0), get_num(1)) {
                (Some(val), Some(n)) => Value::Number(val.powf(1.0 / n)),
                _ => Value::Error("root requires 2 arguments: root(value, n)".to_string()),
            }
        }
        "abs" => {
            get_num(0).map(|n| Value::Number(n.abs()))
                .unwrap_or(Value::Error("abs requires 1 argument".to_string()))
        }
        "log" => {
            get_num(0).map(|n| Value::Number(n.log10()))
                .unwrap_or(Value::Error("log requires 1 argument".to_string()))
        }
        "ln" => {
            get_num(0).map(|n| Value::Number(n.ln()))
                .unwrap_or(Value::Error("ln requires 1 argument".to_string()))
        }
        "fact" | "factorial" => {
            // numi's fact() is the identity function, not factorial
            get_num(0).map(Value::Number)
                .unwrap_or(Value::Error("fact requires 1 argument".to_string()))
        }
        "round" => {
            get_num(0).map(|n| Value::Number(n.round()))
                .unwrap_or(Value::Error("round requires 1 argument".to_string()))
        }
        "ceil" => {
            get_num(0).map(|n| Value::Number(n.ceil()))
                .unwrap_or(Value::Error("ceil requires 1 argument".to_string()))
        }
        "floor" => {
            get_num(0).map(|n| Value::Number(n.floor()))
                .unwrap_or(Value::Error("floor requires 1 argument".to_string()))
        }
        "sin" => {
            get_num(0).map(|n| Value::Number(n.sin()))
                .unwrap_or(Value::Error("sin requires 1 argument".to_string()))
        }
        "cos" => {
            get_num(0).map(|n| Value::Number(n.cos()))
                .unwrap_or(Value::Error("cos requires 1 argument".to_string()))
        }
        "tan" => {
            get_num(0).map(|n| Value::Number(n.tan()))
                .unwrap_or(Value::Error("tan requires 1 argument".to_string()))
        }
        "arcsin" | "asin" => {
            get_num(0).map(|n| Value::Number(n.asin()))
                .unwrap_or(Value::Error("arcsin requires 1 argument".to_string()))
        }
        "arccos" | "acos" => {
            get_num(0).map(|n| Value::Number(n.acos()))
                .unwrap_or(Value::Error("arccos requires 1 argument".to_string()))
        }
        "arctan" | "atan" => {
            get_num(0).map(|n| Value::Number(n.atan()))
                .unwrap_or(Value::Error("arctan requires 1 argument".to_string()))
        }
        "sinh" => {
            get_num(0).map(|n| Value::Number(n.sinh()))
                .unwrap_or(Value::Error("sinh requires 1 argument".to_string()))
        }
        "cosh" => {
            get_num(0).map(|n| Value::Number(n.cosh()))
                .unwrap_or(Value::Error("cosh requires 1 argument".to_string()))
        }
        "tanh" => {
            get_num(0).map(|n| Value::Number(n.tanh()))
                .unwrap_or(Value::Error("tanh requires 1 argument".to_string()))
        }
        "fromunix" => {
            get_num(0).map(|n| {
                let ts = n as i64;
                match Utc.timestamp_opt(ts, 0) {
                    chrono::LocalResult::Single(dt) => {
                        let local = dt.with_timezone(&Local);
                        Value::DateTime(local.format("%Y-%m-%d %H:%M:%S").to_string())
                    }
                    _ => Value::Error("invalid unix timestamp".to_string()),
                }
            }).unwrap_or(Value::Error("fromunix requires 1 argument".to_string()))
        }
        _ => Value::Error(format!("unknown function: {}", name)),
    }
}

fn eval_conversion(expr: &Expr, target: &str, ctx: &EvalContext) -> Value {
    let val = eval_expr(expr, ctx);

    // Format conversions: hex, binary, octal, sci
    match target {
        "hex" => {
            return val.as_number()
                .map(|n| Value::WithUnit(n, "__hex__".to_string()))
                .unwrap_or(Value::Error("cannot convert to hex".to_string()));
        }
        "binary" => {
            return val.as_number()
                .map(|n| Value::WithUnit(n, "__binary__".to_string()))
                .unwrap_or(Value::Error("cannot convert to binary".to_string()));
        }
        "octal" => {
            return val.as_number()
                .map(|n| Value::WithUnit(n, "__octal__".to_string()))
                .unwrap_or(Value::Error("cannot convert to octal".to_string()));
        }
        "sci" | "scientific" => {
            return val.as_number()
                .map(|n| Value::WithUnit(n, "__sci__".to_string()))
                .unwrap_or(Value::Error("cannot convert to scientific".to_string()));
        }
        _ => {}
    }

    // Timezone conversion: "time in Tokyo", "now in UTC"
    if let Value::DateTime(ref dt_str) = val {
        if let Some(tz_result) = try_timezone_conversion(dt_str, target) {
            return tz_result;
        }
    }

    // Unit/currency conversion
    match &val {
        Value::WithUnit(n, from_unit) => {
            if let Some(result) = try_unit_convert(*n, from_unit, target) {
                return result;
            }
            Value::Error(format!("cannot convert {} to {}", from_unit, target))
        }
        Value::Currency(_n, from_code) => {
            Value::Error(format!("currency conversion {} to {} requires rates", from_code, target))
        }
        Value::Number(n) => {
            if is_currency_unit(target) {
                Value::Currency(*n, normalize_currency(target))
            } else {
                Value::WithUnit(*n, display_unit(target))
            }
        }
        _ => Value::Error("cannot convert this value".to_string()),
    }
}

/// Try to convert a datetime string to a different timezone
fn try_timezone_conversion(dt_str: &str, target: &str) -> Option<Value> {
    // Look up the target timezone via alias table, or try parsing directly as IANA
    let tz: Tz = if let Some(iana_name) = elo_data::timezones::find_timezone(target) {
        iana_name.parse().ok()?
    } else {
        target.parse().ok()?
    };

    // Parse the source datetime as local time
    let naive = parse_datetime_str(dt_str)?;
    let local_dt = Local.from_local_datetime(&naive).single()?;

    // Convert to target timezone
    let converted = local_dt.with_timezone(&tz);

    if dt_str.contains(':') {
        Some(Value::DateTime(converted.format("%Y-%m-%d %H:%M:%S").to_string()))
    } else {
        Some(Value::DateTime(converted.format("%Y-%m-%d").to_string()))
    }
}

fn try_unit_convert(value: f64, from_name: &str, to_name: &str) -> Option<Value> {
    let from_unit = elo_data::units::find_unit(from_name)?;
    let to_unit = elo_data::units::find_unit(to_name)?;
    let result = elo_data::units::convert(value, from_unit, to_unit)?;
    let display = display_unit(to_unit.id);
    Some(Value::WithUnit(result, display))
}

fn is_currency_unit(name: &str) -> bool {
    elo_data::currencies::find_currency(name).is_some()
}

fn normalize_currency(name: &str) -> String {
    elo_data::currencies::find_currency(name)
        .map(|c| c.code.to_string())
        .unwrap_or_else(|| name.to_uppercase())
}

fn display_unit(id: &str) -> String {
    match id {
        "m" => "m".to_string(),
        "in" => "″".to_string(),
        "celsius" => "°C".to_string(),
        "fahrenheit" => "°F".to_string(),
        "kelvin" => "K".to_string(),
        "l" => "L".to_string(),
        _ => {
            if let Some(unit) = elo_data::units::find_unit(id) {
                unit.names[0].to_string()
            } else {
                id.to_string()
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval(input: &str) -> Value {
        let mut parser = Parser::new(input);
        let line = parser.parse_line();
        let mut ctx = EvalContext::new();
        eval_line(&line, &mut ctx)
    }

    fn eval_num(input: &str) -> f64 {
        eval(input).as_number().expect(&format!("expected number for: {}", input))
    }

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(eval_num("2 + 3"), 5.0);
        assert_eq!(eval_num("10 - 3"), 7.0);
        assert_eq!(eval_num("3 * 4"), 12.0);
        assert_eq!(eval_num("12 / 3"), 4.0);
        assert_eq!(eval_num("2 ^ 10"), 1024.0);
        assert_eq!(eval_num("10 mod 3"), 1.0);
    }

    #[test]
    fn test_word_operators() {
        assert_eq!(eval_num("10 plus 5"), 15.0);
        assert_eq!(eval_num("10 minus 3"), 7.0);
        assert_eq!(eval_num("3 times 4"), 12.0);
        assert_eq!(eval_num("10 without 3"), 7.0);
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval_num("2 + 3 * 4"), 14.0);
        assert_eq!(eval_num("(2 + 3) * 4"), 20.0);
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval_num("-5"), -5.0);
    }

    #[test]
    fn test_constants() {
        assert!((eval_num("pi") - std::f64::consts::PI).abs() < 1e-10);
        assert!((eval_num("e") - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_functions() {
        assert!((eval_num("sqrt(16)") - 4.0).abs() < 1e-10);
        assert!((eval_num("cbrt(27)") - 3.0).abs() < 1e-10);
        assert_eq!(eval_num("abs(-5)"), 5.0);
        assert_eq!(eval_num("round(3.7)"), 4.0);
        assert_eq!(eval_num("ceil(3.2)"), 4.0);
        assert_eq!(eval_num("floor(3.8)"), 3.0);
        assert!((eval_num("sin(0)") - 0.0).abs() < 1e-10);
        assert!((eval_num("cos(0)") - 1.0).abs() < 1e-10);
        assert!((eval_num("ln(e)") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fact_identity() {
        // numi's fact() is the identity function
        assert_eq!(eval_num("fact(5)"), 5.0);
        assert_eq!(eval_num("fact(0)"), 0.0);
        assert_eq!(eval_num("fact(10)"), 10.0);
    }

    #[test]
    fn test_percentages() {
        assert!((eval_num("50% of 200") - 100.0).abs() < 1e-10);
        assert!((eval_num("10% on 100") - 110.0).abs() < 1e-10);
        assert!((eval_num("10% off 100") - 90.0).abs() < 1e-10);
        assert!((eval_num("100 - 30%") - 70.0).abs() < 1e-10);
        assert!((eval_num("100 + 10%") - 110.0).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_percent() {
        assert_eq!(eval_num("5% of what is 10"), 200.0);
    }

    #[test]
    fn test_scales() {
        assert_eq!(eval_num("1k"), 1000.0);
        assert_eq!(eval_num("2.5M"), 2_500_000.0);
        assert_eq!(eval_num("1 billion"), 1_000_000_000.0);
    }

    #[test]
    fn test_hex_literal() {
        assert_eq!(eval_num("0xFF"), 255.0);
    }

    #[test]
    fn test_bin_literal() {
        assert_eq!(eval_num("0b1010"), 10.0);
    }

    #[test]
    fn test_oct_literal() {
        assert_eq!(eval_num("0o17"), 15.0);
    }

    #[test]
    fn test_bitwise() {
        assert_eq!(eval_num("5 & 3"), 1.0);
        assert_eq!(eval_num("5 | 3"), 7.0);
        assert_eq!(eval_num("5 xor 3"), 6.0);
        assert_eq!(eval_num("1 << 3"), 8.0);
        assert_eq!(eval_num("8 >> 2"), 2.0);
    }

    #[test]
    fn test_variables() {
        let mut ctx = EvalContext::new();
        let line1 = Parser::new("x = 10").parse_line();
        eval_line(&line1, &mut ctx);

        let line2 = Parser::new("x + 5").parse_line();
        let result = eval_line(&line2, &mut ctx);
        assert_eq!(result.as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_prev() {
        let mut ctx = EvalContext::new();
        let line1 = Parser::new("42").parse_line();
        eval_line(&line1, &mut ctx);

        let line2 = Parser::new("prev").parse_line();
        let result = eval_line(&line2, &mut ctx);
        assert_eq!(result.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_sum() {
        let mut ctx = EvalContext::new();
        for input in &["10", "20", "30"] {
            let line = Parser::new(input).parse_line();
            eval_line(&line, &mut ctx);
        }
        let sum_line = Parser::new("sum").parse_line();
        let result = eval_line(&sum_line, &mut ctx);
        assert_eq!(result.as_number().unwrap(), 60.0);
    }

    #[test]
    fn test_avg() {
        let mut ctx = EvalContext::new();
        for input in &["10", "20", "30"] {
            let line = Parser::new(input).parse_line();
            eval_line(&line, &mut ctx);
        }
        let avg_line = Parser::new("avg").parse_line();
        let result = eval_line(&avg_line, &mut ctx);
        assert_eq!(result.as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_unit_conversion() {
        let result = eval("1 meter in cm");
        match result {
            Value::WithUnit(n, unit) => {
                assert!((n - 100.0).abs() < 0.01);
                assert_eq!(unit, "cm");
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    #[test]
    fn test_temperature_conversion() {
        let result = eval("72 fahrenheit in celsius");
        match result {
            Value::WithUnit(n, _unit) => {
                assert!((n - 22.22).abs() < 0.1);
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    #[test]
    fn test_time_conversion() {
        let result = eval("1 hour in minutes");
        match result {
            Value::WithUnit(n, _unit) => {
                assert!((n - 60.0).abs() < 0.01);
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    #[test]
    fn test_mass_conversion() {
        let result = eval("1 kg in pounds");
        match result {
            Value::WithUnit(n, _) => {
                assert!((n - 2.2046).abs() < 0.01);
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    #[test]
    fn test_label_evaluation() {
        let result = eval("Total: 5 + 5");
        assert_eq!(result.as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_comment_empty() {
        assert!(eval("// comment").is_empty());
        assert!(eval("# header").is_empty());
    }

    // --- M3 gap tests ---

    #[test]
    fn test_unit_sequence_meter_cm() {
        let result = eval("1 meter 20 cm");
        match result {
            Value::WithUnit(n, unit) => {
                assert!((n - 120.0).abs() < 0.01, "got {}", n);
                assert_eq!(unit, "cm");
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    #[test]
    fn test_unit_sequence_feet_inches() {
        let result = eval("5 feet 3 inches");
        match result {
            Value::WithUnit(n, unit) => {
                assert!((n - 63.0).abs() < 0.01, "got {}", n);
                assert_eq!(unit, "″");
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    #[test]
    fn test_cross_unit_add() {
        // 5 meter + 200 cm => 700 cm
        let result = eval("5 meter + 200 cm");
        match result {
            Value::WithUnit(n, unit) => {
                assert!((n - 700.0).abs() < 0.01, "got {}", n);
                assert_eq!(unit, "cm");
            }
            _ => panic!("expected WithUnit for cross-unit add, got {:?}", result),
        }
    }

    #[test]
    fn test_cross_unit_add_mass() {
        // 3 kg + 500 g => 3500 g
        let result = eval("3 kg + 500 g");
        match result {
            Value::WithUnit(n, unit) => {
                assert!((n - 3500.0).abs() < 1.0, "got {}", n);
                assert_eq!(unit, "g");
            }
            _ => panic!("expected WithUnit, got {:?}", result),
        }
    }

    // --- M4 tests ---

    #[test]
    fn test_today() {
        let result = eval("today");
        match result {
            Value::DateTime(s) => {
                assert!(s.contains('-'), "expected date format, got: {}", s);
                assert_eq!(s.len(), 10); // YYYY-MM-DD
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_tomorrow() {
        let result = eval("tomorrow");
        match result {
            Value::DateTime(s) => {
                let today = Local::now().date_naive();
                let tomorrow = today + Duration::days(1);
                assert_eq!(s, tomorrow.format("%Y-%m-%d").to_string());
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_yesterday() {
        let result = eval("yesterday");
        match result {
            Value::DateTime(s) => {
                let today = Local::now().date_naive();
                let yesterday = today - Duration::days(1);
                assert_eq!(s, yesterday.format("%Y-%m-%d").to_string());
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_now_returns_datetime() {
        let result = eval("now");
        match result {
            Value::DateTime(s) => {
                assert!(s.contains(':'), "expected time in output: {}", s);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_date_arithmetic_add_days() {
        let result = eval("today + 3 days");
        match result {
            Value::DateTime(s) => {
                let expected = (Local::now().date_naive() + Duration::days(3))
                    .format("%Y-%m-%d").to_string();
                assert_eq!(s, expected);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_date_arithmetic_add_hours() {
        let result = eval("now + 1 hour");
        match result {
            Value::DateTime(s) => {
                assert!(s.contains(':'), "expected datetime with time: {}", s);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_date_arithmetic_sub_weeks() {
        let result = eval("today - 2 weeks");
        match result {
            Value::DateTime(s) => {
                let expected = (Local::now().date_naive() - Duration::weeks(2))
                    .format("%Y-%m-%d").to_string();
                assert_eq!(s, expected);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_timezone_query() {
        // "time in UTC" should return a datetime
        let result = eval("time in UTC");
        match result {
            Value::DateTime(s) => {
                assert!(s.contains(':'), "expected datetime: {}", s);
            }
            _ => panic!("expected DateTime for timezone query, got {:?}", result),
        }
    }

    #[test]
    fn test_timezone_query_iana() {
        let result = eval("now in America/New_York");
        match result {
            Value::DateTime(s) => {
                assert!(s.contains(':'));
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_fromunix_epoch() {
        let result = eval("fromunix(0)");
        match result {
            Value::DateTime(s) => {
                // Should be 1970-01-01 in some timezone
                assert!(s.starts_with("19"), "expected 1970 date, got: {}", s);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_fromunix_billion() {
        let result = eval("fromunix(1000000000)");
        match result {
            Value::DateTime(s) => {
                // 2001-09-09 in UTC
                assert!(s.starts_with("200"), "expected 2001 date, got: {}", s);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_today_plus_week() {
        let result = eval("today + 1 week");
        match result {
            Value::DateTime(s) => {
                let expected = (Local::now().date_naive() + Duration::weeks(1))
                    .format("%Y-%m-%d").to_string();
                assert_eq!(s, expected);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }

    #[test]
    fn test_tomorrow_plus_days() {
        let result = eval("tomorrow + 2 days");
        match result {
            Value::DateTime(s) => {
                let expected = (Local::now().date_naive() + Duration::days(3))
                    .format("%Y-%m-%d").to_string();
                assert_eq!(s, expected);
            }
            _ => panic!("expected DateTime, got {:?}", result),
        }
    }
}
