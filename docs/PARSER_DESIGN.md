# Parser Design: The Brainless Sweep

The Crunchie parser is a high-performance, single-pass character transformer. It turns raw text into a flat, ID-addressed map of structural units.

## Rationale: Data-Oriented Structuralism
By avoiding a traditional recursive AST, we gain:
1. **Cache Locality**: Units are stored in flat arrays.
2. **Simplified Ownership**: No complex tree lifetimes; relationships are managed via Integer IDs.
3. **Robustness**: The parser only tracks stack depth and identity, leaving semantic validity to the Engine.

## The Unified Identity Space
The parser maintains a monotonic ID counter. Every structural unit (Symbol, Operator, or Container) is assigned an ID. The ID space is pre-populated prior to parsing:
- **Negative IDs**: Reserved for built-in operators (e.g., `-1` through `-1000000`).
- **High Positive IDs**: Reserved for built-in or user-supplied constants (e.g., `> 1000000`).
- **Standard Positive IDs**: Dynamically assigned to user-defined symbols and new containers during the sweep.
The parser accepts the built-in and constant maps as arguments during initialization.

## The State Machine
The parser iterates character-by-character with a stack of active Container IDs.

### 1. The "Twin Rule"
`\n` and `;` are boundary triggers. 
- If `stack.depth == 1` (Root): Close current Level-1 container, start new one.
- If `stack.depth > 1`: Act as a sequence operator (homogenized into the `,` operator unit).

### 2. Nesting
- `(`: Allocate new ID, push to current container's content, push to stack.
- `)`: Pop stack. If depth was 1, mark as stray error.

### 3. Symbol Interning
During the sweep, the parser maintains a `Map<String, ID>`. 
- If a string is seen again, it reuses the ID.
- This ensures **Symbolic Identity** (all instances of `x` share an ID) at the earliest possible stage.

## Output Format

The parser yields a highly-optimized, flat data structure designed for Engine iteration and separate UI reporting:

```rust
struct Position { offset: u32, line: u32, col: u32 }
struct Span { start: Position, end: Position }

// Just a span; text remains in the buffer.
type Comment = Span; 

struct Unit { id: i32, offset: u32 }

struct Container {
    contents: Vec<Unit>,
    // The ONLY error state the Engine cares about.
    valid: bool, 
}

enum DiagnosticCode {
    StrayCloser,
    UnclosedContainer,
    IllegalCharacter,
}

struct Diagnostic {
    code: DiagnosticCode,
    span: Span, // Exact provenance for the UI
}

struct ParserResult {
    containers: Map<i32, Container>, // Root is always key 0
    symbols: Map<String, i32>,
    comments: Vec<Comment>,
    diagnostics: Vec<Diagnostic>,
}
```

## Error Model: The Decoupling
We strictly decouple **Execution State** from **Reporting State**:
1. **The Engine** only cares about execution flow. If the buffer ends with items on the stack, those containers are marked `valid = false`. The Engine uses this to quickly "Poison" dependent expressions and move on.
2. **The UI/LSP** only cares about reporting. The parser emits `diagnostics` (e.g., `UnclosedContainer` or `StrayCloser`) with rich `Span` information (line/col) so the UI can draw squiggles without impacting Engine iteration speed.
