use std::sync::Arc;

use crate::ast::{Expr, Line};
use crate::eval::{EvalContext, eval_expr};
use crate::formatter::format_value;
use crate::parser::Parser;
use crate::rates::RateStore;
use crate::value::Value;

/// A session represents a multi-line document evaluation
pub struct Session {
    ctx: EvalContext,
    in_code_fence: bool,
}

/// How a line was interpreted.
///
/// Elo is a notepad first and a calculator second: a document is a mix of
/// prose and the occasional expression. Rather than parsing every line
/// eagerly and surfacing "unknown identifier" errors for ordinary sentences,
/// each line is classified into one of these states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// The line was recognized as a formula. The accompanying [`LineResult`]
    /// `value` holds the result, which may itself be a [`Value::Error`] for a
    /// genuinely malformed calculation (e.g. `foo(10)` or `1 / bogus`).
    Formula,
    /// The line is plain text / markdown prose (or a header, comment, code
    /// fence, etc.). No result is displayed and it never reports an error.
    Text,
}

/// Result of evaluating a single line
#[derive(Debug, Clone)]
pub struct LineResult {
    pub input: String,
    pub value: Value,
    pub display: String,
    pub kind: LineKind,
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
        let text = || LineResult {
            input: input.to_string(),
            value: Value::Empty,
            display: String::new(),
            kind: LineKind::Text,
        };

        if input.trim().is_empty() {
            self.ctx.new_block();
            return text();
        }

        // Code fence toggle: lines starting with ``` produce no result.
        if input.trim_start().starts_with("```") {
            self.in_code_fence = !self.in_code_fence;
            return text();
        }

        // Inside a code fence: never evaluate.
        if self.in_code_fence {
            return text();
        }

        // Markdown list marker ("- " / "* "): strip it and evaluate the rest,
        // so "- 2 + 2" works. A marker immediately followed by a digit (e.g.
        // "- 5") is left intact so it still parses as a negation, matching how
        // a calculator would read it. A bare marker ("- ") is just text.
        let trimmed = input.trim_start();
        let content = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let after_marker = trimmed[2..].trim_start();
            if after_marker.is_empty() {
                return text();
            }
            if after_marker.starts_with(|c: char| !c.is_ascii_digit()) {
                after_marker
            } else {
                input
            }
        } else {
            input
        };

        let mut parser = Parser::new(content);
        let line = parser.parse_line();
        let consumed_all = parser.at_end();

        // Headers, comments, and lines that lex to nothing are structural prose.
        if matches!(line, Line::Empty | Line::Comment(_) | Line::Header(_)) {
            return text();
        }

        // Evaluate without committing side effects yet — a line that turns out
        // to be prose must not pollute `prev`, the running block, or variables.
        let value = match &line {
            Line::Expression { expr, .. } | Line::Assignment { expr, .. } => {
                eval_expr(expr, &self.ctx)
            }
            // `Empty`/`Comment`/`Header` were handled above.
            _ => Value::Empty,
        };

        match classify(&line, consumed_all, &value) {
            LineKind::Text => text(),
            LineKind::Formula => {
                // Commit side effects only now that we know it's a real formula.
                if let Line::Assignment { name, .. } = &line {
                    self.ctx.variables.insert(name.clone(), value.clone());
                }
                self.ctx.record_result(&value);
                let display = format_value(&value);
                LineResult {
                    input: input.to_string(),
                    value,
                    display,
                    kind: LineKind::Formula,
                }
            }
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

/// Decide whether an evaluated line should be surfaced as a formula result or
/// treated as plain prose.
///
/// The parser is deliberately eager (see the markdown/list-item work in commits
/// `090c470` and `7a36ca6`): genuine mistakes like `foo(10)` or `1 / bogus`
/// should still surface "unknown function" / "unknown identifier" errors rather
/// than silently disappearing. So we only fall back to [`LineKind::Text`] when a
/// line *both* errors *and* doesn't look like a deliberate calculation:
///
///   * the parser left tokens unconsumed — trailing prose like "buy 3 apples"
///     or "- `code` text", where only the first word parsed; or
///   * the whole line is a single bare identifier, e.g. "groceries".
///
/// A line that evaluates cleanly is always a formula (so writing a lone variable
/// name like `total` still recalls its value), and a line that parsed fully into
/// a computational structure keeps its error.
fn classify(line: &Line, consumed_all: bool, value: &Value) -> LineKind {
    if !value.is_error() {
        return LineKind::Formula;
    }
    if !consumed_all || is_bare_text(line) {
        return LineKind::Text;
    }
    LineKind::Formula
}

/// A line whose expression is a single identifier with nothing else — plain
/// prose such as "groceries" or "TODO", optionally carrying a label.
fn is_bare_text(line: &Line) -> bool {
    matches!(
        line,
        Line::Expression {
            expr: Expr::Ident(_),
            ..
        }
    )
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

    #[test]
    fn test_session_plain_text_is_not_error() {
        let mut session = Session::new();
        // A bare word and a full sentence are prose, not failed formulas.
        for line in ["groceries", "buy some milk", "this is a note"] {
            let result = session.eval_line(line);
            assert!(!result.value.is_error(), "{line:?} should not error");
            assert!(result.value.is_empty(), "{line:?} should be empty");
            assert_eq!(result.kind, LineKind::Text, "{line:?} should be text");
        }
    }

    #[test]
    fn test_session_prose_with_number_is_text() {
        let mut session = Session::new();
        // Leading word + trailing tokens: parser stops at "I", prose remains.
        let result = session.eval_line("I have 3 cats");
        assert!(!result.value.is_error());
        assert_eq!(result.kind, LineKind::Text);
    }

    #[test]
    fn test_session_real_formula_error_is_preserved() {
        let mut session = Session::new();
        // Structured calculations that fail should still surface their error.
        for line in ["1 / bogus", "foo(10)", "10 +"] {
            let result = session.eval_line(line);
            assert!(result.value.is_error(), "{line:?} should error");
            assert_eq!(result.kind, LineKind::Formula, "{line:?} is a formula");
        }
    }

    #[test]
    fn test_session_bare_variable_recall() {
        let mut session = Session::new();
        session.eval_line("total = 100");
        // A lone known variable name recalls its value (not treated as text).
        let result = session.eval_line("total");
        assert_eq!(result.display, "100");
        assert_eq!(result.kind, LineKind::Formula);
    }

    #[test]
    fn test_session_prose_does_not_pollute_prev() {
        let mut session = Session::new();
        session.eval_line("42");
        session.eval_line("this is a note"); // prose: must not clobber prev
        let result = session.eval_line("prev");
        assert_eq!(result.display, "42");
    }

    #[test]
    fn test_session_prose_does_not_pollute_sum() {
        let mut session = Session::new();
        session.eval_line("10");
        session.eval_line("shopping list"); // prose between numbers
        session.eval_line("20");
        let result = session.eval_line("sum");
        assert_eq!(result.display, "30");
    }

    #[test]
    fn test_session_text_list_item_kind() {
        let mut session = Session::new();
        assert_eq!(session.eval_line("- groceries").kind, LineKind::Text);
        assert_eq!(session.eval_line("* TODO item").kind, LineKind::Text);
    }
}
