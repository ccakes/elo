use std::fmt;

/// Source span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    HexNumber(i64),
    BinNumber(i64),
    OctNumber(i64),
    SciNumber(f64, String), // value and original text like "1.5e3"

    // Identifiers and keywords
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    Ampersand,
    Pipe,
    LShift,
    RShift,
    Equals,

    // Delimiters
    LParen,
    RParen,
    Comma,
    Colon,

    // Note structure
    Hash,          // # for headers
    DoubleSlash,   // // for comments
    Quote(String), // "quoted text"

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::HexNumber(n) => write!(f, "0x{:x}", n),
            Token::BinNumber(n) => write!(f, "0b{:b}", n),
            Token::OctNumber(n) => write!(f, "0o{:o}", n),
            Token::SciNumber(_, s) => write!(f, "{}", s),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Caret => write!(f, "^"),
            Token::Percent => write!(f, "%"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::LShift => write!(f, "<<"),
            Token::RShift => write!(f, ">>"),
            Token::Equals => write!(f, "="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Hash => write!(f, "#"),
            Token::DoubleSlash => write!(f, "//"),
            Token::Quote(s) => write!(f, "\"{}\"", s),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        // Check for header line
        if self.pos < self.chars.len() && self.chars[self.pos] == '#' {
            tokens.push(SpannedToken {
                token: Token::Hash,
                span: Span::new(0, self.input.len()),
            });
            return tokens;
        }

        // Check for comment line
        if self.input.trim_start().starts_with("//") {
            tokens.push(SpannedToken {
                token: Token::DoubleSlash,
                span: Span::new(0, self.input.len()),
            });
            return tokens;
        }

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }

            let start = self.pos;
            let ch = self.chars[self.pos];

            let token = match ch {
                '0' if self.pos + 1 < self.chars.len() => match self.chars[self.pos + 1] {
                    'x' | 'X' => self.lex_hex(),
                    'b' | 'B' => self.lex_bin(),
                    'o' | 'O' => self.lex_oct(),
                    _ => self.lex_number(),
                },
                '0'..='9' | '.'
                    if ch == '.'
                        && self.pos + 1 < self.chars.len()
                        && self.chars[self.pos + 1].is_ascii_digit() =>
                {
                    self.lex_number()
                }
                '0'..='9' => self.lex_number(),
                '+' => {
                    self.pos += 1;
                    Token::Plus
                }
                '-' => {
                    self.pos += 1;
                    Token::Minus
                }
                '*' => {
                    self.pos += 1;
                    Token::Star
                }
                '/' => {
                    self.pos += 1;
                    Token::Slash
                }
                '^' => {
                    self.pos += 1;
                    Token::Caret
                }
                '%' => {
                    self.pos += 1;
                    Token::Percent
                }
                '&' => {
                    self.pos += 1;
                    Token::Ampersand
                }
                '|' => {
                    self.pos += 1;
                    Token::Pipe
                }
                '<' if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '<' => {
                    self.pos += 2;
                    Token::LShift
                }
                '>' if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '>' => {
                    self.pos += 2;
                    Token::RShift
                }
                '=' => {
                    self.pos += 1;
                    Token::Equals
                }
                '(' => {
                    self.pos += 1;
                    Token::LParen
                }
                ')' => {
                    self.pos += 1;
                    Token::RParen
                }
                ',' => {
                    self.pos += 1;
                    Token::Comma
                }
                ':' => {
                    self.pos += 1;
                    Token::Colon
                }
                '#' => {
                    self.pos += 1;
                    Token::Hash
                }
                '"' => self.lex_quoted_string(),
                '°' => {
                    // °C or °F
                    self.pos += 1;
                    if self.pos < self.chars.len() {
                        let next = self.chars[self.pos];
                        if next == 'C' || next == 'F' {
                            self.pos += 1;
                            Token::Ident(format!("°{}", next))
                        } else {
                            Token::Ident("°".to_string())
                        }
                    } else {
                        Token::Ident("°".to_string())
                    }
                }
                '″' => {
                    self.pos += 1;
                    Token::Ident("″".to_string())
                }
                '₹' | '€' | '£' | '¥' | '₩' | '₿' | '₺' | '₪' | '₱' | '₽' | '฿' =>
                {
                    self.pos += 1;
                    Token::Ident(ch.to_string())
                }
                '$' => {
                    self.pos += 1;
                    Token::Ident("$".to_string())
                }
                _ if ch.is_alphabetic() || ch == '_' => self.lex_identifier(),
                _ => {
                    self.pos += 1;
                    continue; // skip unknown characters
                }
            };

            tokens.push(SpannedToken {
                token,
                span: Span::new(start, self.pos),
            });
        }

        tokens
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn lex_number(&mut self) -> Token {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_e = false;

        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_digit() {
                self.pos += 1;
            } else if ch == '.' && !has_dot && !has_e {
                // Check it's not a range operator or end-of-expression dot
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].is_ascii_digit() {
                    has_dot = true;
                    self.pos += 1;
                } else {
                    break;
                }
            } else if (ch == 'e' || ch == 'E') && !has_e {
                // Scientific notation
                has_e = true;
                self.pos += 1;
                if self.pos < self.chars.len()
                    && (self.chars[self.pos] == '+' || self.chars[self.pos] == '-')
                {
                    self.pos += 1;
                }
            } else if ch == ',' {
                // Thousand separator: skip if followed by exactly 3 digits then non-digit
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].is_ascii_digit() {
                    self.pos += 1; // skip comma
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let text: String = self.chars[start..self.pos]
            .iter()
            .filter(|c| **c != ',')
            .collect();

        if has_e {
            let val: f64 = text.parse().unwrap_or(0.0);
            let orig: String = self.chars[start..self.pos].iter().collect();
            Token::SciNumber(val, orig)
        } else {
            let val: f64 = text.parse().unwrap_or(0.0);
            Token::Number(val)
        }
    }

    fn lex_hex(&mut self) -> Token {
        self.pos += 2; // skip 0x
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_hexdigit() {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let val = i64::from_str_radix(&text, 16).unwrap_or(0);
        Token::HexNumber(val)
    }

    fn lex_bin(&mut self) -> Token {
        self.pos += 2; // skip 0b
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos] == '0' || self.chars[self.pos] == '1')
        {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let val = i64::from_str_radix(&text, 2).unwrap_or(0);
        Token::BinNumber(val)
    }

    fn lex_oct(&mut self) -> Token {
        self.pos += 2; // skip 0o
        let start = self.pos;
        while self.pos < self.chars.len()
            && self.chars[self.pos] >= '0'
            && self.chars[self.pos] <= '7'
        {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let val = i64::from_str_radix(&text, 8).unwrap_or(0);
        Token::OctNumber(val)
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_alphanumeric() || ch == '_' || ch == '/' {
                // Allow '/' in identifiers for units like "m/s", "km/h"
                if ch == '/' {
                    // Only include if followed by letter (unit denominator)
                    if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].is_alphabetic() {
                        self.pos += 1;
                    } else {
                        break;
                    }
                } else {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        Token::Ident(text)
    }

    fn lex_quoted_string(&mut self) -> Token {
        self.pos += 1; // skip opening "
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if self.pos < self.chars.len() {
            self.pos += 1; // skip closing "
        }
        Token::Quote(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token> {
        Lexer::new(input)
            .tokenize()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn test_basic_number() {
        assert_eq!(tokens("42"), vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_decimal_number() {
        #[allow(clippy::approx_constant)]
        let expected = vec![Token::Number(3.14)];
        assert_eq!(tokens("3.14"), expected);
    }

    #[test]
    fn test_hex_number() {
        assert_eq!(tokens("0xFF"), vec![Token::HexNumber(255)]);
    }

    #[test]
    fn test_bin_number() {
        assert_eq!(tokens("0b1010"), vec![Token::BinNumber(10)]);
    }

    #[test]
    fn test_oct_number() {
        assert_eq!(tokens("0o17"), vec![Token::OctNumber(15)]);
    }

    #[test]
    fn test_arithmetic() {
        let toks = tokens("2 + 3 * 4");
        assert_eq!(
            toks,
            vec![
                Token::Number(2.0),
                Token::Plus,
                Token::Number(3.0),
                Token::Star,
                Token::Number(4.0),
            ]
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(tokens("meter"), vec![Token::Ident("meter".to_string())]);
    }

    #[test]
    fn test_comment() {
        let toks = tokens("// this is a comment");
        assert_eq!(toks, vec![Token::DoubleSlash]);
    }

    #[test]
    fn test_header() {
        let toks = tokens("# Budget");
        assert_eq!(toks, vec![Token::Hash]);
    }

    #[test]
    fn test_label_expr() {
        let toks = tokens("Total: 5 + 5");
        assert_eq!(
            toks,
            vec![
                Token::Ident("Total".to_string()),
                Token::Colon,
                Token::Number(5.0),
                Token::Plus,
                Token::Number(5.0),
            ]
        );
    }

    #[test]
    fn test_parentheses() {
        let toks = tokens("(2 + 3)");
        assert_eq!(
            toks,
            vec![
                Token::LParen,
                Token::Number(2.0),
                Token::Plus,
                Token::Number(3.0),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_percent() {
        let toks = tokens("30%");
        assert_eq!(toks, vec![Token::Number(30.0), Token::Percent]);
    }

    #[test]
    fn test_scientific_notation() {
        let toks = tokens("1.5e3");
        assert!(matches!(toks[0], Token::SciNumber(_, _)));
        if let Token::SciNumber(val, _) = &toks[0] {
            assert!((*val - 1500.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_bitwise() {
        let toks = tokens("5 & 3 | 1 << 2 >> 1");
        assert_eq!(
            toks,
            vec![
                Token::Number(5.0),
                Token::Ampersand,
                Token::Number(3.0),
                Token::Pipe,
                Token::Number(1.0),
                Token::LShift,
                Token::Number(2.0),
                Token::RShift,
                Token::Number(1.0),
            ]
        );
    }

    #[test]
    fn test_currency_symbol() {
        let toks = tokens("€ 100");
        assert_eq!(
            toks,
            vec![Token::Ident("€".to_string()), Token::Number(100.0),]
        );
    }

    #[test]
    fn test_number_with_commas() {
        let toks = tokens("1,000,000");
        assert_eq!(toks, vec![Token::Number(1_000_000.0)]);
    }
}
