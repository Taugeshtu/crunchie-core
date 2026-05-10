use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub offset: u32,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub type Comment = Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: i32,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container {
    pub contents: Vec<Unit>,
    /// The ONLY error state the Engine cares about.
    pub corrupted: bool,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            contents: Vec::new(),
            corrupted: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCode {
    StrayCloser,
    UnclosedContainer,
    IllegalCharacter,
    StraySequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserResult {
    /// Root is always key 0
    pub containers: HashMap<i32, Container>,
    pub symbols: HashMap<String, i32>,
    pub comments: Vec<Comment>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineResult {
    pub diagnostics: Vec<Diagnostic>,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Assign,
    Sequence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoupledUnit {
    /// A physical quantity (value + optional unit)
    Quantity { value: f64, unit: Option<String> },
    /// A named binding, optionally with a requested unit (e.g. "x kg")
    Binding {
        name: String,
        request_unit: Option<String>,
    },
    /// A function call (e.g. "sin")
    Function(String),
    /// A mathematical operator
    Operator(OpCode),
    /// A nested group of units (e.g. from parentheses)
    Group(Vec<CoupledUnit>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoupledResult {
    pub lines: Vec<Vec<CoupledUnit>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    /// A direct physical quantity
    Literal { value: f64, unit: Option<String> },
    /// A reference to the result of a previous instruction on the Tape
    Register(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub op: OpCode,
    pub args: Vec<Operand>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tape {
    pub instructions: Vec<Instruction>,
    /// Maps Instruction Index -> (Variable Name, Requested Unit)
    pub assignments: HashMap<usize, (String, Option<String>)>,
    /// List of Instruction Indices that represent queries (e.g. "x = ")
    pub queries: Vec<usize>,
}

