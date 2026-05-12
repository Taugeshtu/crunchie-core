use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
pub enum DiagnosticCode {
    StrayCloser,
    UnclosedContainer,
    IllegalCharacter,
    StraySequence,
    MalformedSymbol,
    MalformedExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
    #[serde(skip)]
    pub value: Option<fend_core::value::Value>,
}

impl PartialEq for TextEdit {
    fn eq(&self, other: &Self) -> bool {
        if self.span != other.span || self.new_text != other.new_text {
            return false;
        }
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => {
                let mut ctx = fend_core::Context::new();
                let int = fend_core::interrupt::Never;
                match a.compare(b, &mut ctx, &int) {
                    Ok(Some(std::cmp::Ordering::Equal)) => true,
                    _ => false,
                }
            }
            (None, None) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EngineResult {
    pub diagnostics: Vec<Diagnostic>,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FendOp {
    Add, Sub, Mul, Div, Mod, Pow,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    To, Factorial, Of,
    ShiftLeft, ShiftRight,
    Equals, DoubleEquals, NotEquals,
    Fn, Backslash, Dot, Semicolon,
    Combination, Permutation,
    OpenParens, CloseParens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    /// Direct mapping to Fend's lexer symbols.
    Fend(FendOp),

    // --- Missing Comparisons ---
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // --- Compound Assignments ---
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,

    // --- Structural / Crunchie-specific ---
    Comma,    // Argument separator
    Sequence, // Newline or explicit line break
    Call,     // Internal instruction calling
}

impl OpCode {
    pub fn is_sequence(&self) -> bool {
        matches!(self, OpCode::Sequence | OpCode::Fend(FendOp::Semicolon))
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    /// The global ID counter for minting new atoms and containers.
    pub next_id: i32,
    
    /// The absolute source of truth for what an ID means.
    /// Both the Parser and the Distiller mint entries here.
    pub atoms: HashMap<i32, Atom>,       
    
    /// For O(1) lookups during the Parser's initial string interning.
    pub intern_map: HashMap<String, i32>,    
    
    /// The topology of the buffer. Lists of raw IDs.
    pub containers: HashMap<i32, Container>,
    
    pub comments: Vec<Span>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Workspace {
    pub fn get_or_intern_atom(&mut self, sym: &str) -> i32 {
        if let Some(&id) = self.intern_map.get(sym) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.intern_map.insert(sym.to_string(), id);
        self.atoms.insert(id, Atom::Raw(sym.to_string()));
        id
    }

    pub fn get_or_intern_atom_typed(&mut self, sym: Atom) -> i32 {
        // If it's Raw, we can use the intern_map. For other types, we might just mint new ones
        // or add them to the intern_map if they have a string representation.
        match &sym {
            Atom::Raw(s) | Atom::Variable(s) | Atom::Constant(s) | Atom::Function(s) => {
                if let Some(&id) = self.intern_map.get(s) {
                    // Update existing atom if it was Raw but we now know it's a Variable, etc.
                    self.atoms.insert(id, sym);
                    return id;
                }
                let id = self.next_id;
                self.next_id += 1;
                self.intern_map.insert(s.clone(), id);
                self.atoms.insert(id, sym);
                id
            }
            _ => {
                let id = self.next_id;
                self.next_id += 1;
                self.atoms.insert(id, sym);
                id
            }
        }
    }

    pub fn get_position(&self, offset: u32, text: &str) -> Position {
        let mut line = 0;
        let mut col = 0;
        for (i, c) in text.char_indices() {
            if i as u32 >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        Position { offset, line, col }
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            next_id: 1,
            atoms: HashMap::new(),
            intern_map: HashMap::new(),
            containers: HashMap::new(),
            comments: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    /// An incredibly tight, cache-friendly array of 8-byte chunks.
    pub contents: Vec<Entity>,
    /// Tracks if this container was unclosed or fatally malformed.
    pub corrupted: bool, 
    /// The structural starting position of this container (e.g. the '(')
    pub start_pos: Position,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            contents: Vec::new(),
            corrupted: false,
            start_pos: Position { offset: 0, line: 0, col: 0 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity { 
    /// The pointer to `Workspace.atoms`.
    pub id: i32,
    /// Provenance: Where did this entity originate in the text buffer?
    pub offset: u32, 
}

#[derive(Debug, Clone)]
pub enum Atom {
    // --- Stage 1: Seeded by the Parser ---
    /// Unclassified alphanumeric monoliths (e.g., "5kg", "x")
    Raw(String),
    /// Points to a key in `Workspace.containers`
    Container(i32), 
    Operator(OpCode),
    Function(String),
    Constant(String),

    // --- Stage 2: Minted by the Distiller (Number Muncher) ---
    /// A terminal value owned by Fend (holds units, precision, etc.)
    Value(fend_core::value::Value),
    /// A named variable binding
    Variable(String),
    /// Injected when typization fatally fails
    Poison,

    // --- Stage 3: Minted by the Vectorizer (Aspiration) ---
    /// Evaluated from Container during the vectorization pass
    VectorRef(i32),
    /// Stage 3: Minted by the Unroller
    Instruction {
        op: OpCode,
        args: Vec<i32>,
    },
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Raw(a), Self::Raw(b)) => a == b,
            (Self::Container(a), Self::Container(b)) => a == b,
            (Self::Operator(a), Self::Operator(b)) => a == b,
            (Self::Function(a), Self::Function(b)) => a == b,
            (Self::Constant(a), Self::Constant(b)) => a == b,
            (Self::Variable(a), Self::Variable(b)) => a == b,
            (Self::Poison, Self::Poison) => true,
            (Self::VectorRef(a), Self::VectorRef(b)) => a == b,
            (Self::Instruction { op: op1, args: args1 }, Self::Instruction { op: op2, args: args2 }) => {
                op1 == op2 && args1 == args2
            }
            (Self::Value(a), Self::Value(b)) => {
                let mut ctx = fend_core::Context::new();
                let int = fend_core::interrupt::Never;
                match a.compare(b, &mut ctx, &int) {
                    Ok(Some(std::cmp::Ordering::Equal)) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
