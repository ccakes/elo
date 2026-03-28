use crate::ast::*;
use crate::lexer::*;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    input: String,
}

/// Recognized word operators and keywords
fn is_add_word(s: &str) -> bool {
    matches!(s, "plus" | "and" | "with")
}

fn is_sub_word(s: &str) -> bool {
    matches!(s, "minus" | "subtract" | "without")
}

fn is_mul_word(s: &str) -> bool {
    matches!(s, "times" | "mul")
}

fn is_div_word(s: &str) -> bool {
    matches!(s, "divide")
}

fn is_conversion_word(s: &str) -> bool {
    matches!(s, "in" | "into" | "as" | "to")
}

fn is_session_token(s: &str) -> bool {
    matches!(s, "prev" | "sum" | "total" | "average" | "avg")
}

fn is_date_keyword(s: &str) -> bool {
    matches!(s, "today" | "tomorrow" | "yesterday")
}

fn is_scale_word(s: &str) -> bool {
    matches!(s, "k" | "thousand" | "M" | "million" | "billion")
}

fn word_to_scale(s: &str) -> Option<Scale> {
    match s {
        "k" | "thousand" => Some(Scale::Thousand),
        "M" | "million" => Some(Scale::Million),
        "billion" => Some(Scale::Billion),
        _ => None,
    }
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = Lexer::new(input).tokenize();
        Self {
            tokens,
            pos: 0,
            input: input.to_string(),
        }
    }

    pub fn parse_line(&mut self) -> Line {
        if self.tokens.is_empty() {
            return Line::Empty;
        }

        // Check for header
        if matches!(self.peek(), Some(Token::Hash)) {
            return Line::Header(self.input.clone());
        }

        // Check for comment
        if matches!(self.peek(), Some(Token::DoubleSlash)) {
            return Line::Comment(self.input.clone());
        }

        // Check for label: "Identifier: expr"
        if let Some(label) = self.try_parse_label() {
            let expr = self.parse_expr();
            return Line::Expression {
                label: Some(label),
                expr,
                span: Span::new(0, self.input.len()),
            };
        }

        // Check for assignment: "name = expr"
        if let Some((name, expr)) = self.try_parse_assignment() {
            return Line::Assignment {
                name,
                expr,
                span: Span::new(0, self.input.len()),
            };
        }

        // Regular expression
        let expr = self.parse_expr();
        Line::Expression {
            label: None,
            expr,
            span: Span::new(0, self.input.len()),
        }
    }

    fn try_parse_label(&mut self) -> Option<String> {
        // Pattern: Ident Colon (with remaining tokens after colon)
        if self.pos + 1 < self.tokens.len()
            && let Token::Ident(ref name) = self.tokens[self.pos].token
            && let Token::Colon = self.tokens[self.pos + 1].token
        {
            // Make sure there's something after the colon
            if self.pos + 2 < self.tokens.len() {
                let label = name.clone();
                self.pos += 2; // skip ident and colon
                return Some(label);
            }
        }
        None
    }

    fn try_parse_assignment(&mut self) -> Option<(String, Expr)> {
        // Pattern: Ident Equals Expr
        if self.pos + 1 < self.tokens.len()
            && let Token::Ident(ref name) = self.tokens[self.pos].token
            && let Token::Equals = self.tokens[self.pos + 1].token
        {
            let name = name.clone();
            self.pos += 2; // skip ident and =
            let expr = self.parse_expr();
            return Some((name, expr));
        }
        None
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_conversion()
    }

    /// Parse conversion: expr (in|into|as|to) target
    fn parse_conversion(&mut self) -> Expr {
        let expr = self.parse_percent_ops();

        if let Some(Token::Ident(ref word)) = self.peek()
            && is_conversion_word(word)
        {
            self.advance();
            if let Some(Token::Ident(ref target)) = self.peek() {
                let target = target.clone();
                self.advance();
                return Expr::Conversion {
                    expr: Box::new(expr),
                    target,
                };
            }
        }
        expr
    }

    /// Parse percentage operations: X% of Y, X% on Y, X% off Y
    fn parse_percent_ops(&mut self) -> Expr {
        let expr = self.parse_bitwise_or();

        // Check if this is a percentage followed by of/on/off
        if let Expr::Percentage(pct) = &expr {
            let pct_val = *pct;
            if let Some(Token::Ident(ref word)) = self.peek() {
                let word = word.clone();
                match word.as_str() {
                    "of" => {
                        self.advance();
                        // Check for "of what is" pattern
                        if let Some(Token::Ident(ref w)) = self.peek()
                            && w == "what"
                        {
                            self.advance();
                            if let Some(Token::Ident(ref w2)) = self.peek()
                                && w2 == "is"
                            {
                                self.advance();
                                let result = self.parse_bitwise_or();
                                return Expr::PercentOfWhatIs {
                                    percent: Box::new(Expr::Number(pct_val)),
                                    result: Box::new(result),
                                };
                            }
                        }
                        let base = self.parse_bitwise_or();
                        return Expr::PercentOf {
                            percent: Box::new(Expr::Number(pct_val)),
                            base: Box::new(base),
                        };
                    }
                    "on" => {
                        self.advance();
                        let base = self.parse_bitwise_or();
                        return Expr::PercentOn {
                            percent: Box::new(Expr::Number(pct_val)),
                            base: Box::new(base),
                        };
                    }
                    "off" => {
                        self.advance();
                        let base = self.parse_bitwise_or();
                        return Expr::PercentOff {
                            percent: Box::new(Expr::Number(pct_val)),
                            base: Box::new(base),
                        };
                    }
                    _ => {}
                }
            }
        }

        // Handle "expr - X%" as percent-off sugar and "expr + X%" as percent-on sugar
        expr
    }

    fn parse_bitwise_or(&mut self) -> Expr {
        let mut left = self.parse_bitwise_xor();
        while let Some(Token::Pipe) = self.peek() {
            self.advance();
            let right = self.parse_bitwise_xor();
            left = Expr::BinaryOp {
                op: BinOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_bitwise_xor(&mut self) -> Expr {
        let mut left = self.parse_bitwise_and();
        while let Some(Token::Ident(ref w)) = self.peek() {
            if w == "xor" {
                self.advance();
                let right = self.parse_bitwise_and();
                left = Expr::BinaryOp {
                    op: BinOp::BitXor,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        left
    }

    fn parse_bitwise_and(&mut self) -> Expr {
        let mut left = self.parse_shift();
        while let Some(Token::Ampersand) = self.peek() {
            self.advance();
            let right = self.parse_shift();
            left = Expr::BinaryOp {
                op: BinOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_shift(&mut self) -> Expr {
        let mut left = self.parse_additive();
        loop {
            match self.peek() {
                Some(Token::LShift) => {
                    self.advance();
                    let right = self.parse_additive();
                    left = Expr::BinaryOp {
                        op: BinOp::Shl,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::RShift) => {
                    self.advance();
                    let right = self.parse_additive();
                    left = Expr::BinaryOp {
                        op: BinOp::Shr,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    // Handle "expr + X%" as percent-on
                    if let Expr::Percentage(pct) = &right {
                        left = Expr::PercentOn {
                            percent: Box::new(Expr::Number(*pct)),
                            base: Box::new(left),
                        };
                    } else {
                        left = Expr::BinaryOp {
                            op: BinOp::Add,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    }
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    // Handle "expr - X%" as percent-off
                    if let Expr::Percentage(pct) = &right {
                        left = Expr::PercentOff {
                            percent: Box::new(Expr::Number(*pct)),
                            base: Box::new(left),
                        };
                    } else {
                        left = Expr::BinaryOp {
                            op: BinOp::Sub,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    }
                }
                Some(Token::Ident(ref w)) if is_add_word(w) => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    left = Expr::BinaryOp {
                        op: BinOp::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Ident(ref w)) if is_sub_word(w) => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    left = Expr::BinaryOp {
                        op: BinOp::Sub,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_mod();
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_mod();
                    left = Expr::BinaryOp {
                        op: BinOp::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_mod();
                    left = Expr::BinaryOp {
                        op: BinOp::Div,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Ident(ref w)) if is_mul_word(w) => {
                    self.advance();
                    let right = self.parse_mod();
                    left = Expr::BinaryOp {
                        op: BinOp::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Ident(ref w)) if w == "multiplied" => {
                    self.advance();
                    // expect "by"
                    if let Some(Token::Ident(ref w2)) = self.peek()
                        && w2 == "by"
                    {
                        self.advance();
                    }
                    let right = self.parse_mod();
                    left = Expr::BinaryOp {
                        op: BinOp::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Ident(ref w)) if is_div_word(w) => {
                    self.advance();
                    // optional "by"
                    if let Some(Token::Ident(ref w2)) = self.peek()
                        && w2 == "by"
                    {
                        self.advance();
                    }
                    let right = self.parse_mod();
                    left = Expr::BinaryOp {
                        op: BinOp::Div,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_mod(&mut self) -> Expr {
        let mut left = self.parse_power();
        while let Some(Token::Ident(ref w)) = self.peek() {
            if w == "mod" {
                self.advance();
                let right = self.parse_power();
                left = Expr::BinaryOp {
                    op: BinOp::Mod,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        left
    }

    fn parse_power(&mut self) -> Expr {
        let base = self.parse_unary();
        if let Some(Token::Caret) = self.peek() {
            self.advance();
            let exp = self.parse_unary();
            Expr::BinaryOp {
                op: BinOp::Pow,
                left: Box::new(base),
                right: Box::new(exp),
            }
        } else {
            base
        }
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let operand = self.parse_postfix();
                Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                }
            }
            Some(Token::Plus) => {
                self.advance();
                self.parse_postfix()
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        // Handle postfix: percentage, scale, unit attachment
        loop {
            match self.peek() {
                Some(Token::Percent) => {
                    self.advance();
                    // Convert the expr to a percentage value
                    if let Some(n) = expr_as_number(&expr) {
                        expr = Expr::Percentage(n);
                    }
                }
                Some(Token::Ident(ref w)) if is_scale_word(w) => {
                    if let Some(n) = expr_as_number(&expr) {
                        let scale = word_to_scale(w).unwrap();
                        self.advance();
                        expr = Expr::Scaled(n, scale);
                    } else {
                        break;
                    }
                }
                Some(Token::Ident(ref w)) => {
                    let word = w.clone();
                    // Check if it's a unit name that should attach to the number
                    if can_attach_unit(&word) {
                        self.advance();
                        expr = Expr::WithUnit(Box::new(expr), word.clone());

                        // Check for unit sequence: "1 meter 20 cm"
                        // If next is a number followed by a compatible unit
                        expr = self.try_parse_unit_sequence(expr, &word);
                    } else {
                        break;
                    }
                }
                Some(Token::LParen) => {
                    // Implicit multiplication: expr(...)
                    // Only if previous expr is a number or identifier
                    if matches!(expr, Expr::Number(_) | Expr::Ident(_) | Expr::Scaled(_, _)) {
                        let inner = self.parse_primary();
                        expr = Expr::ImplicitMul(Box::new(expr), Box::new(inner));
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        expr
    }

    /// Try to extend a WithUnit into a UnitSequence: "1 meter 20 cm"
    fn try_parse_unit_sequence(&mut self, first_expr: Expr, first_unit: &str) -> Expr {
        let first_dim = unit_dimension(first_unit);
        if first_dim.is_none() {
            return first_expr;
        }
        let dim = first_dim.unwrap();

        let mut parts: Vec<(Box<Expr>, String)> = Vec::new();

        // Extract the inner expr from the WithUnit
        if let Expr::WithUnit(inner, unit) = first_expr {
            parts.push((inner, unit));
        } else {
            return first_expr;
        }

        // Greedily consume: Number UnitInSameDimension
        loop {
            // Check if next is a number
            let has_number = matches!(self.peek(), Some(Token::Number(_)));
            if !has_number {
                break;
            }

            // Peek ahead: is there a unit after the number?
            if self.pos + 1 < self.tokens.len()
                && let Token::Ident(ref next_unit) = self.tokens[self.pos + 1].token
                && is_unit_like(next_unit)
                && unit_dimension(next_unit) == Some(dim)
            {
                // Consume number and unit
                let num_expr = self.parse_primary();
                if let Some(Token::Ident(ref u)) = self.peek() {
                    let u = u.clone();
                    self.advance();
                    parts.push((Box::new(num_expr), u));
                    continue;
                }
            }
            break;
        }

        if parts.len() == 1 {
            let (inner, unit) = parts.into_iter().next().unwrap();
            Expr::WithUnit(inner, unit)
        } else {
            Expr::UnitSequence(parts)
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek() {
            Some(Token::Number(n)) => {
                self.advance();
                Expr::Number(n)
            }
            Some(Token::HexNumber(n)) => {
                self.advance();
                Expr::HexLiteral(n)
            }
            Some(Token::BinNumber(n)) => {
                self.advance();
                Expr::BinLiteral(n)
            }
            Some(Token::OctNumber(n)) => {
                self.advance();
                Expr::OctLiteral(n)
            }
            Some(Token::SciNumber(val, text)) => {
                let text = text.clone();
                self.advance();
                Expr::SciLiteral(val, text)
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr();
                // Expect RParen
                if matches!(self.peek(), Some(Token::RParen)) {
                    self.advance();
                }
                Expr::Paren(Box::new(expr))
            }
            Some(Token::Ident(ref name)) => {
                let name = name.clone();

                // Session tokens
                match name.as_str() {
                    "prev" => {
                        self.advance();
                        return Expr::Prev;
                    }
                    "sum" | "total" => {
                        self.advance();
                        return Expr::Sum;
                    }
                    "avg" | "average" => {
                        self.advance();
                        return Expr::Avg;
                    }
                    "today" => {
                        self.advance();
                        return Expr::Today;
                    }
                    "tomorrow" => {
                        self.advance();
                        return Expr::Tomorrow;
                    }
                    "yesterday" => {
                        self.advance();
                        return Expr::Yesterday;
                    }
                    _ => {}
                }

                // Check if it's a function call: name(args)
                if self.pos + 1 < self.tokens.len()
                    && let Token::LParen = &self.tokens[self.pos + 1].token
                    && elo_data::functions::is_builtin_function(&name)
                {
                    return self.parse_func_call(&name);
                }

                // Currency symbol followed by number: $ 100, € 50
                if is_currency_symbol(&name) {
                    self.advance();
                    if let Some(Token::Number(n)) = self.peek() {
                        self.advance();
                        let code = symbol_to_currency_code(&name);
                        return Expr::WithUnit(Box::new(Expr::Number(n)), code.to_string());
                    }
                    return Expr::Ident(name);
                }

                self.advance();
                Expr::Ident(name)
            }
            _ => {
                // Error recovery: skip token and return an error ident
                if self.pos < self.tokens.len() {
                    self.advance();
                }
                Expr::Ident("__error__".to_string())
            }
        }
    }

    fn parse_func_call(&mut self, name: &str) -> Expr {
        let name = name.to_string();
        self.advance(); // skip function name
        self.advance(); // skip (

        let mut args = Vec::new();
        while !matches!(self.peek(), Some(Token::RParen) | None) {
            args.push(self.parse_expr());
            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some(Token::RParen)) {
            self.advance();
        }

        Expr::FuncCall { name, args }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).map(|t| t.token.clone())
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
}

fn expr_as_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Number(n) => Some(*n),
        Expr::HexLiteral(n) => Some(*n as f64),
        Expr::BinLiteral(n) => Some(*n as f64),
        Expr::OctLiteral(n) => Some(*n as f64),
        Expr::SciLiteral(n, _) => Some(*n),
        _ => None,
    }
}

/// Heuristic: is this identifier likely a unit name?
fn is_unit_like(name: &str) -> bool {
    // Known unit names and abbreviations
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "mm" | "cm"
            | "dm"
            | "meter"
            | "meters"
            | "metre"
            | "metres"
            | "km"
            | "inch"
            | "inches"
            | "ft"
            | "foot"
            | "feet"
            | "yd"
            | "yard"
            | "yards"
            | "mi"
            | "mile"
            | "miles"
            | "nmi"
            | "mg"
            | "g"
            | "gram"
            | "grams"
            | "kg"
            | "kilogram"
            | "kilograms"
            | "kilo"
            | "kilos"
            | "oz"
            | "ounce"
            | "ounces"
            | "lb"
            | "lbs"
            | "pound"
            | "pounds"
            | "st"
            | "stone"
            | "stones"
            | "ton"
            | "tons"
            | "tonne"
            | "tonnes"
            | "ms"
            | "s"
            | "sec"
            | "second"
            | "seconds"
            | "min"
            | "minute"
            | "minutes"
            | "hr"
            | "hour"
            | "hours"
            | "day"
            | "days"
            | "week"
            | "weeks"
            | "month"
            | "months"
            | "year"
            | "years"
            | "celsius"
            | "fahrenheit"
            | "kelvin"
            | "rad"
            | "radian"
            | "radians"
            | "deg"
            | "degree"
            | "degrees"
            | "bit"
            | "bits"
            | "byte"
            | "bytes"
            | "kb"
            | "mb"
            | "gb"
            | "tb"
            | "pb"
            | "kib"
            | "mib"
            | "gib"
            | "tib"
            | "px"
            | "pixel"
            | "pixels"
            | "pt"
            | "point"
            | "points"
            | "pc"
            | "pica"
            | "picas"
            | "em"
            | "rem"
            | "vw"
            | "vh"
            | "ml"
            | "cl"
            | "liter"
            | "liters"
            | "litre"
            | "litres"
            | "l"
            | "gal"
            | "gallon"
            | "gallons"
            | "qt"
            | "quart"
            | "quarts"
            | "cup"
            | "cups"
            | "floz"
            | "tbsp"
            | "tsp"
            | "mph"
            | "kmph"
            | "kph"
            | "knot"
            | "knots"
            | "hectare"
            | "hectares"
            | "ha"
            | "acre"
            | "acres"
            | "usd"
            | "eur"
            | "gbp"
            | "jpy"
            | "cny"
            | "aud"
            | "cad"
            | "chf"
            | "sek"
            | "nzd"
            | "krw"
            | "sgd"
            | "nok"
            | "mxn"
            | "inr"
            | "rub"
            | "brl"
            | "zar"
            | "hkd"
            | "twd"
            | "pln"
            | "thb"
            | "idr"
            | "czk"
            | "ils"
            | "php"
            | "try"
            | "dkk"
            | "btc"
            | "eth"
    ) || name == "°C"
        || name == "°F"
        || name == "″"
        || name == "m/s"
        || name == "km/h"
}

/// Can this identifier attach as a postfix unit to a number?
fn can_attach_unit(word: &str) -> bool {
    is_unit_like(word)
        && !is_conversion_word(word)
        && !is_add_word(word)
        && !is_sub_word(word)
        && !is_mul_word(word)
        && !is_div_word(word)
        && !is_session_token(word)
        && !is_date_keyword(word)
        && !matches!(
            word,
            "mod" | "of" | "on" | "off" | "multiplied" | "xor" | "what" | "is"
        )
}

/// Get the dimension tag for a unit name (for same-dimension checks in unit sequences).
/// Returns a small integer tag or None.
fn unit_dimension(name: &str) -> Option<u8> {
    elo_data::units::find_unit(name).map(|u| u.dimension as u8)
}

fn is_currency_symbol(s: &str) -> bool {
    matches!(
        s,
        "$" | "€" | "£" | "¥" | "₹" | "₩" | "₿" | "₺" | "₪" | "₱" | "₽" | "฿"
    )
}

fn symbol_to_currency_code(s: &str) -> &str {
    match s {
        "$" => "USD",
        "€" => "EUR",
        "£" => "GBP",
        "¥" => "JPY",
        "₹" => "INR",
        "₩" => "KRW",
        "₿" => "BTC",
        "₺" => "TRY",
        "₪" => "ILS",
        "₱" => "PHP",
        "₽" => "RUB",
        "฿" => "THB",
        _ => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Line {
        Parser::new(input).parse_line()
    }

    #[test]
    fn test_simple_addition() {
        let line = parse("2 + 3");
        if let Line::Expression { expr, .. } = line {
            if let Expr::BinaryOp { op, left, right } = expr {
                assert_eq!(op, BinOp::Add);
                assert!(matches!(*left, Expr::Number(n) if (n - 2.0).abs() < f64::EPSILON));
                assert!(matches!(*right, Expr::Number(n) if (n - 3.0).abs() < f64::EPSILON));
            } else {
                panic!("Expected BinaryOp");
            }
        } else {
            panic!("Expected Expression");
        }
    }

    #[test]
    fn test_word_operators() {
        let line = parse("10 plus 5");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Add, .. }));
        }
    }

    #[test]
    fn test_precedence() {
        let line = parse("2 + 3 * 4");
        if let Line::Expression { expr, .. } = line {
            if let Expr::BinaryOp { op, left, right } = expr {
                assert_eq!(op, BinOp::Add);
                assert!(matches!(*left, Expr::Number(n) if (n - 2.0).abs() < f64::EPSILON));
                assert!(matches!(*right, Expr::BinaryOp { op: BinOp::Mul, .. }));
            }
        }
    }

    #[test]
    fn test_assignment() {
        let line = parse("x = 10");
        assert!(matches!(line, Line::Assignment { ref name, .. } if name == "x"));
    }

    #[test]
    fn test_label() {
        let line = parse("Total: 5 + 5");
        if let Line::Expression { label, .. } = line {
            assert_eq!(label, Some("Total".to_string()));
        }
    }

    #[test]
    fn test_function_call() {
        let line = parse("sqrt(16)");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::FuncCall { ref name, .. } if name == "sqrt"));
        }
    }

    #[test]
    fn test_percentage() {
        let line = parse("30%");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::Percentage(n) if (n - 30.0).abs() < f64::EPSILON));
        }
    }

    #[test]
    fn test_conversion() {
        let line = parse("1 meter in cm");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::Conversion { .. }));
        }
    }

    #[test]
    fn test_session_tokens() {
        assert!(matches!(
            parse("prev"),
            Line::Expression {
                expr: Expr::Prev,
                ..
            }
        ));
        assert!(matches!(
            parse("sum"),
            Line::Expression {
                expr: Expr::Sum,
                ..
            }
        ));
        assert!(matches!(
            parse("avg"),
            Line::Expression {
                expr: Expr::Avg,
                ..
            }
        ));
        assert!(matches!(
            parse("total"),
            Line::Expression {
                expr: Expr::Sum,
                ..
            }
        ));
        assert!(matches!(
            parse("average"),
            Line::Expression {
                expr: Expr::Avg,
                ..
            }
        ));
    }

    #[test]
    fn test_comment() {
        assert!(matches!(parse("// a comment"), Line::Comment(_)));
    }

    #[test]
    fn test_header() {
        assert!(matches!(parse("# Budget"), Line::Header(_)));
    }

    #[test]
    fn test_percent_of() {
        let line = parse("50% of 200");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::PercentOf { .. }));
        }
    }

    #[test]
    fn test_percent_on() {
        let line = parse("10% on 100");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::PercentOn { .. }));
        }
    }

    #[test]
    fn test_percent_off() {
        let line = parse("10% off 100");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::PercentOff { .. }));
        }
    }

    #[test]
    fn test_scale_k() {
        let line = parse("1k");
        if let Line::Expression { expr, .. } = line {
            assert!(
                matches!(expr, Expr::Scaled(n, Scale::Thousand) if (n - 1.0).abs() < f64::EPSILON)
            );
        }
    }

    #[test]
    fn test_hex_literal() {
        let line = parse("0xFF");
        if let Line::Expression { expr, .. } = line {
            assert!(matches!(expr, Expr::HexLiteral(255)));
        }
    }
}
