# Data Model: The Unified Workspace

The Crunchie engine operates via a single mutable `Workspace`. The pipeline stages (Parser, Janitor, Distiller, Vectorizer) are progressive mutations of this shared graph.

## The Core Concept: Identity vs. Meaning

The architecture is built on a strict separation between where a thing appears and what that thing actually is.

*   **Identity (ID)**: A stable `i32` that uniquely identifies a concept (e.g., the name "x" or a specific parenthetical group).
*   **Meaning (Atom)**: The semantic definition of an ID (e.g., is it a raw string, a variable, or a container?). This **evolves** as it moves through the pipeline.
*   **Occurrence (Entity)**: A specific location in a container where an ID appears, paired with its original source offset.

> **The Power of the Map**: Because Entities only store a blind ID, we can **refine an Atom in the global map and it is immediately updated everywhere it occurs**. For example, when the Distiller identifies that ID 500 is a "Variable", every line in the workspace using ID 500 is updated instantly without a recursive search.

## 1. The Core Structs

```rust
pub struct Workspace {
    /// The global ID counter for minting new IDs.
    pub next_id: i32,
    
    /// The absolute source of truth for what an ID means.
    /// Both the Parser and the Distiller mint/update entries here.
    pub atoms: HashMap<i32, Atom>,       
    
    /// A global string-deduplication index. Ensures that the string "x"
    /// always resolves to the same ID across Parser, Janitor, and Distiller.
    pub intern_map: HashMap<String, i32>,    
    
    /// The topology of the buffer. Lists of raw IDs and offsets.
    pub containers: HashMap<i32, Container>,
    
    pub comments: Vec<Span>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Container {
    /// An incredibly tight, cache-friendly array of 8-byte chunks.
    pub contents: Vec<Entity>,
    /// Tracks if this container was unclosed or fatally malformed.
    pub corrupted: bool, 
    /// The structural starting position of this container (e.g. the '(')
    pub start_pos: Position,
}

pub struct Entity { 
    /// The pointer to `Workspace.atoms`.
    pub id: i32,
    /// Provenance: Where did this entity originate in the text buffer?
    pub offset: u32, 
}
```

## 2. The Atom Enum

The `Atom` enum acts as a tagged union representing the semantic meaning of an ID.

```rust
pub enum Atom {
    // --- Stage 1: Seeded by the Parser ---
    /// Unclassified alphanumeric monoliths (e.g., "5kg", "x")
    Raw(String),
    /// Points to a key in `Workspace.containers`
    Container(i32), 
    Operator(OpCode),
    Function(String),
    Constant(String),
    
    // --- Stage 2: Minted by the Distiller ---
    /// A terminal value owned by Fend (holds units, precision, etc.)
    Value(fend_core::value::Value),
    /// A named variable binding
    Variable(String),
    /// Injected when typization fatally fails
    Poison,
    
    // --- Stage 2.5: Minted by the Vectorizer (Aspiration) ---
    /// Evaluated from a Container during the vectorization pass
    VectorRef(i32),
    /// --- Stage 3: Minted by the Unroller ---
    Instruction { op: OpCode, args: Vec<i32> },
}
```

## 3. Why This Architecture Wins
*   **No Recursive Tree Walking**: The Distiller doesn't need to recursively walk trees. It just iterates `Workspace.containers.values_mut()`, looks at the flat `contents` lists, checks the global atom map, and updates it.
*   **Zero Loss of Provenance**: When symbols split, we just copy the `offset` from the old `Entity` into the new `Entity`s.
*   **Memory Efficiency**: The `Workspace.containers` lists are just `u64` arrays (`i32` ID + `u32` offset), fitting perfectly in CPU cache lines.
