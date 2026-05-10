# Data Model: The Unified Workspace

## The Problem
The Crunchie engine originally passed discrete structs (`ParserResult` -> `SemanticResult` -> `Tape`) between stages. This required recursively rebuilding container trees at every stage, leading to double-indirection, loss of provenance (offset tracking), and redundant allocations.

## The Solution: A Progressive Entity-Component Graph
We are flattening the pipeline. Instead of passing distinct results, the entire engine operates on a single mutable `Workspace`. The pipeline stages (Parser, Janitor, Distiller, Vectorizer) are just progressive mutations of this shared graph.

### 1. The Core Structs

```rust
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

pub struct Container {
    /// An incredibly tight, cache-friendly array of 8-byte chunks.
    pub contents: Vec<Entity>,
    /// Tracks if this container was unclosed or fatally malformed.
    pub corrupted: bool, 
}

pub struct Entity { 
    /// The pointer to `Workspace.symbols`.
    pub id: i32,
    /// Provenance: Where did this entity originate in the text buffer?
    pub offset: u32, 
}
```

### 2. The Symbol Enum

The `Symbol` enum acts as a tagged union representing the semantic meaning of an `Entity`. While Rust pads this enum to ~32 bytes (to accommodate the `String` in `Raw`), this is perfectly acceptable because the `Symbol` map is deduplicated and the engine spends 90% of its time iterating over the tightly packed `Vec<Entity>` arrays.

```rust
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
}
```

### 3. Pipeline Flow Example: `5kg`

1.  **Parser**: Sweeps `5kg` at offset `10`. It mints `Symbol::Raw("5kg")` at ID `100`. It pushes `Entity(id: 100, offset: 10)` into the current `Container`.
2.  **Distiller**: Iterates the `Container`. Sees ID `100` is `Raw("5kg")`. It runs the Muncher.
    *   Mints `Symbol::Quantity(5.0)` at ID `101`.
    *   Mints `Symbol::PhysUnit("kg")` at ID `102`.
    *   *Mutates* the container's contents: replaces `Entity(id: 100)` with `Entity(id: 101, offset: 10)` and `Entity(id: 102, offset: 10)`.
    *   *Notice that the exact original offset is preserved for both new units!*

### 4. Why This Architecture Wins
*   **No Recursive Tree Walking**: The Distiller doesn't need to recursively walk trees. It just iterates `Workspace.containers.values_mut()`, looks at the flat `contents` lists, checks the global symbol map, and swaps out `Entity` pointers.
*   **Zero Loss of Provenance**: When symbols split, we just copy the `offset` from the old `Entity` into the new `Entity`s.
*   **Memory Efficiency**: The `Workspace.containers` lists are just `u64` arrays (`i32` ID + `u32` offset), fitting perfectly in CPU cache lines.
