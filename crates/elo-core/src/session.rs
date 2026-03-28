use crate::eval::{EvalContext, eval_line};
use crate::formatter::format_value;
use crate::parser::Parser;
use crate::value::Value;

/// A session represents a multi-line document evaluation
pub struct Session {
    ctx: EvalContext,
}

/// Result of evaluating a single line
#[derive(Debug, Clone)]
pub struct LineResult {
    pub input: String,
    pub value: Value,
    pub display: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            ctx: EvalContext::new(),
        }
    }

    /// Evaluate a single line in the context of this session
    pub fn eval_line(&mut self, input: &str) -> LineResult {
        if input.trim().is_empty() {
            self.ctx.new_block();
            return LineResult {
                input: input.to_string(),
                value: Value::Empty,
                display: String::new(),
            };
        }

        let mut parser = Parser::new(input);
        let line = parser.parse_line();
        let value = eval_line(&line, &mut self.ctx);
        let display = format_value(&value);

        LineResult {
            input: input.to_string(),
            value,
            display,
        }
    }

    /// Evaluate a full document (multiple lines)
    pub fn eval_document(&mut self, input: &str) -> Vec<LineResult> {
        input.lines().map(|line| self.eval_line(line)).collect()
    }

    /// Get the current evaluation context (for inspection)
    pub fn context(&self) -> &EvalContext {
        &self.ctx
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_basic() {
        let mut session = Session::new();
        let result = session.eval_line("2 + 3");
        assert_eq!(result.display, "5");
    }

    #[test]
    fn test_session_variables() {
        let mut session = Session::new();
        session.eval_line("x = 10");
        let result = session.eval_line("x + 5");
        assert_eq!(result.display, "15");
    }

    #[test]
    fn test_session_prev() {
        let mut session = Session::new();
        session.eval_line("42");
        let result = session.eval_line("prev");
        assert_eq!(result.display, "42");
    }

    #[test]
    fn test_session_sum() {
        let mut session = Session::new();
        session.eval_line("10");
        session.eval_line("20");
        session.eval_line("30");
        let result = session.eval_line("sum");
        assert_eq!(result.display, "60");
    }

    #[test]
    fn test_session_avg() {
        let mut session = Session::new();
        session.eval_line("10");
        session.eval_line("20");
        session.eval_line("30");
        let result = session.eval_line("avg");
        assert_eq!(result.display, "20");
    }

    #[test]
    fn test_session_block_separation() {
        let mut session = Session::new();
        session.eval_line("10");
        session.eval_line("20");
        session.eval_line(""); // empty line resets block
        session.eval_line("30");
        let result = session.eval_line("sum");
        assert_eq!(result.display, "30"); // only 30 in current block
    }

    #[test]
    fn test_session_document() {
        let mut session = Session::new();
        let results = session.eval_document("# Budget\nRent: 1500\nFood: 500\nsum");
        assert_eq!(results.len(), 4);
        assert!(results[0].value.is_empty()); // header
        assert_eq!(results[1].display, "1500");
        assert_eq!(results[2].display, "500");
        assert_eq!(results[3].display, "2000");
    }

    #[test]
    fn test_session_label() {
        let mut session = Session::new();
        let result = session.eval_line("Total: 5 + 5");
        assert_eq!(result.display, "10");
    }

    #[test]
    fn test_session_comment() {
        let mut session = Session::new();
        let result = session.eval_line("// this is a comment");
        assert!(result.value.is_empty());
    }

    #[test]
    fn test_session_format_conversion() {
        let mut session = Session::new();
        let result = session.eval_line("10 as hex");
        assert_eq!(result.display, "0xa");

        let result = session.eval_line("10 as binary");
        assert_eq!(result.display, "0b1010");

        let result = session.eval_line("10 as octal");
        assert_eq!(result.display, "0o12");

        let result = session.eval_line("100 in sci");
        assert_eq!(result.display, "1e2");
    }
}
