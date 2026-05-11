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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: Span,
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
    To,
    Call,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    /// The global ID counter for minting new symbols and containers.
    pub next_id: i32,
    
    /// The absolute source of truth for what an ID means.
    /// Both the Parser and the Distiller mint entries here.
    pub symbols: HashMap<i32, Symbol>,       
    
    /// For O(1) lookups during the Parser's initial string interning.
    pub intern_map: HashMap<String, i32>,    
    
    /// The topology of the buffer. Lists of raw IDs.
    pub containers: HashMap<i32, Container>,
    
    pub comments: Vec<Span>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Workspace {
    pub fn get_or_intern_symbol(&mut self, sym: &str) -> i32 {
        if let Some(&id) = self.intern_map.get(sym) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.intern_map.insert(sym.to_string(), id);
        self.symbols.insert(id, Symbol::Raw(sym.to_string()));
        id
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            next_id: 1,
            symbols: HashMap::new(),
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
    /// The pointer to `Workspace.symbols`.
    pub id: i32,
    /// Provenance: Where did this entity originate in the text buffer?
    pub offset: u32, 
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    // --- Stage 1: Seeded by the Parser ---
    /// Unclassified alphanumeric monoliths (e.g., "5kg", "x")
    Raw(String),
    /// Points to a key in `Workspace.containers`
    ContainerRef(i32), 
    Operator(OpCode),
    Function(String),
    Constant(String),
    
    // --- Stage 2: Minted by the Distiller (Number Muncher) ---
    /// The f64 result of a successfully parsed number
    Quantity(f64),
    /// A named variable binding
    Variable(String),
    /// A standalone physical unit recognized by Numbat
    PhysUnit(String),
    /// Injected when typization fatally fails
    Poison,
    
    // --- Stage 3: Minted by the Vectorizer (Aspiration) ---
    /// Evaluated from ContainerRef during the vectorization pass
    VectorRef(i32),
    /// Stage 3: Minted by the Unroller
    Instruction {
        op: OpCode,
        args: Vec<i32>,
    },
}