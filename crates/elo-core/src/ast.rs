use crate::lexer::Span;

/// Top-level line classification
#[derive(Debug, Clone)]
pub enum Line {
    /// Empty line (block separator)
    Empty,
    /// Comment line (// ...)
    Comment(String),
    /// Header line (# ...)
    Header(String),
    /// Expression line, optionally with a label
    Expression {
        label: Option<String>,
        expr: Expr,
        span: Span,
    },
    /// Variable assignment: name = expr
    Assignment {
        name: String,
        expr: Expr,
        span: Span,
    },
}

/// Expression AST node
#[derive(Debug, Clone)]
pub enum Expr {
    /// Numeric literal
    Number(f64),
    /// Hex literal
    HexLiteral(i64),
    /// Binary literal
    BinLiteral(i64),
    /// Octal literal
    OctLiteral(i64),
    /// Scientific notation literal
    SciLiteral(f64, String),

    /// Percentage value (e.g., 30%)
    Percentage(f64),

    /// Identifier (variable, unit, keyword)
    Ident(String),

    /// Binary operation
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    /// Function call
    FuncCall {
        name: String,
        args: Vec<Expr>,
    },

    /// Conversion expression: expr in/to/as unit
    Conversion {
        expr: Box<Expr>,
        target: String,
    },

    /// Percentage operations
    PercentOf {
        percent: Box<Expr>,
        base: Box<Expr>,
    },
    PercentOn {
        percent: Box<Expr>,
        base: Box<Expr>,
    },
    PercentOff {
        percent: Box<Expr>,
        base: Box<Expr>,
    },

    /// "X% of what is Y" => inverse percent
    PercentOfWhatIs {
        percent: Box<Expr>,
        result: Box<Expr>,
    },

    /// Session tokens
    Prev,
    Sum,
    Avg,

    /// Number with scale: 1k, 2.5M, 1 billion
    Scaled(f64, Scale),

    /// Number with unit: 5 meters, 100 kg
    WithUnit(Box<Expr>, String),

    /// Implicit multiplication: 6(3), 2pi
    ImplicitMul(Box<Expr>, Box<Expr>),

    /// Parenthesized expression
    Paren(Box<Expr>),

    /// Unit sequence: 1 meter 20 cm => accumulate in second unit
    UnitSequence(Vec<(Box<Expr>, String)>),

    /// Date keywords
    Today,
    Tomorrow,
    Yesterday,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Pos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scale {
    Thousand, // k
    Million,  // M
    Billion,  // billion
}

impl Scale {
    pub fn multiplier(self) -> f64 {
        match self {
            Scale::Thousand => 1_000.0,
            Scale::Million => 1_000_000.0,
            Scale::Billion => 1_000_000_000.0,
        }
    }
}
