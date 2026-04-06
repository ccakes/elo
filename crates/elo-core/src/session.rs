use std::sync::Arc;

use crate::ast::{Expr, Line};
use crate::eval::{EvalContext, eval_line};
use crate::formatter::format_value;
use crate::parser::Parser;
use crate::rates::RateStore;
use crate::value::Value;

/// A session represents a multi-line document evaluation
pub struct Session {
    ctx: EvalContext,
    in_code_fence: bool,
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
            in_code_fence: false,
        }
    }

    pub fn with_rates(rates: Option<Arc<RateStore>>) -> Self {
        Self {
            ctx: EvalContext::with_rates(rates),
            in_code_fence: false,
        }
    }

    /// Evaluate a single line in the context of this session
    pub fn eval_line(&mut self, input: &str) -> LineResult {
        let empty = || LineResult {
            input: input.to_string(),
            value: Value::Empty,
            display: String::new(),
        };

        if input.trim().is_empty() {
            self.ctx.new_block();
            return empty();
        }

        // Code fence toggle: lines starting with ``` produce empty results
        if input.trim_start().starts_with("```") {
            self.in_code_fence = !self.in_code_fence;
            return empty();
        }

        // Inside a code fence: don't evaluate
        if self.in_code_fence {
            return empty();
        }

        // List item prefix: strip "- " or "* ", evaluate the rest.
        // If the stripped content errors (pure text), return empty instead.
        let trimmed = input.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let after_marker = trimmed[2..].trim_start();
            // Empty list marker (just "- " or "* ")
            if after_marker.is_empty() {
                return empty();
            }
            // Non-digit content: try to evaluate, fall back to empty on error.
            // Digits fall through so "- 5" still parses as negative 5.
            if after_marker.starts_with(|c: char| !c.is_ascii_digit()) {
                let mut parser = Parser::new(after_marker);
                let line = parser.parse_line();
                // Only swallow errors for bare identifiers (plain text like "groceries").
                // Expression-like structures (function calls, operators, etc.) should
                // propagate their errors so the user gets useful feedback.
                let is_bare_text = matches!(
                    &line,
                    Line::Expression {
                        expr: Expr::Ident(_),
                        ..
                    }
                );
                let value = eval_line(&line, &mut self.ctx);
                if value.is_error() && is_bare_text {
                    return empty();
                }
                let display = format_value(&value);
                return LineResult {
                    input: input.to_string(),
                    value,
                    display,
                };
            }
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

    #[test]
    fn test_session_list_item_text() {
        let mut session = Session::new();
        let result = session.eval_line("- groceries");
        assert!(result.value.is_empty());
    }

    #[test]
    fn test_session_list_item_empty() {
        let mut session = Session::new();
        // "* " and "- " with nothing after should be empty, not error
        assert!(session.eval_line("* ").value.is_empty());
        assert!(session.eval_line("- ").value.is_empty());
        assert!(session.eval_line("*  ").value.is_empty());
    }

    #[test]
    fn test_session_list_item_star_text() {
        let mut session = Session::new();
        let result = session.eval_line("* TODO item");
        assert!(result.value.is_empty());
    }

    #[test]
    fn test_session_list_item_with_label_expr() {
        let mut session = Session::new();
        // List item with a labeled expression should evaluate
        let result = session.eval_line("* price: 100 + 50");
        assert_eq!(result.display, "150");
    }

    #[test]
    fn test_session_list_item_with_expr() {
        let mut session = Session::new();
        let result = session.eval_line("- sqrt(16)");
        assert_eq!(result.display, "4");
    }

    #[test]
    fn test_session_dash_number_still_works() {
        let mut session = Session::new();
        let result = session.eval_line("- 5");
        assert_eq!(result.display, "-5");
    }

    #[test]
    fn test_session_code_fence() {
        let mut session = Session::new();
        let result = session.eval_line("```");
        assert!(result.value.is_empty());

        // Inside code fence: should not evaluate
        let result = session.eval_line("my $foo = bar();");
        assert!(result.value.is_empty());

        let result = session.eval_line("```");
        assert!(result.value.is_empty());

        // After closing fence: normal evaluation resumes
        let result = session.eval_line("2 + 3");
        assert_eq!(result.display, "5");
    }

    #[test]
    fn test_session_code_fence_document() {
        let mut session = Session::new();
        let results = session.eval_document("10\n```\nsome code\n```\n20\nsum");
        assert_eq!(results[0].display, "10");
        assert!(results[1].value.is_empty()); // ```
        assert!(results[2].value.is_empty()); // some code
        assert!(results[3].value.is_empty()); // ```
        assert_eq!(results[4].display, "20");
        assert_eq!(results[5].display, "30"); // sum of 10 + 20
    }

    #[test]
    fn test_session_list_item_inline_code() {
        let mut session = Session::new();
        // "- `code fence` list item" should be empty, not an error
        let result = session.eval_line("- `code fence` list item");
        assert!(result.value.is_empty());
    }

    #[test]
    fn test_session_list_item_inline_code_after_text() {
        let mut session = Session::new();
        // "- foo `code fence`" should also be empty (text with inline code)
        let result = session.eval_line("- foo `code fence`");
        assert!(result.value.is_empty());
    }
}
