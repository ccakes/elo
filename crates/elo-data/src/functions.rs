/// Built-in function names recognized by the engine
pub static BUILTIN_FUNCTIONS: &[&str] = &[
    "sqrt",
    "cbrt",
    "root",
    "abs",
    "log",
    "ln",
    "fact",
    "factorial",
    "round",
    "ceil",
    "floor",
    "sin",
    "cos",
    "tan",
    "arcsin",
    "arccos",
    "arctan",
    "asin",
    "acos",
    "atan",
    "sinh",
    "cosh",
    "tanh",
    "fromunix",
];

pub fn is_builtin_function(name: &str) -> bool {
    BUILTIN_FUNCTIONS.contains(&name)
}
