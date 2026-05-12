# Crunchie-Core API Reference

Crunchie-Core is a library for processing a mathematical DSL in a text buffer. It uses a multi-stage pipeline to parse, analyze, and evaluate expressions while maintaining context and handling partial errors ("poisoning").

## Primary Entry Point

### `process_buffer`

The most common way to use Crunchie is through the `process_buffer` function. It runs the entire pipeline on a provided string and returns the modified text (with "fills" if enabled) and a list of diagnostics.

```rust
pub fn process_buffer(text: &str, config: &Config) -> (String, Vec<model::Diagnostic>)
```

- **`text`**: The raw string content of your buffer.
- **`config`**: An instance of `Config` (see below).
- **Returns**: A tuple containing:
    - `String`: The final text. If `config.generate_fills` is true, this will include calculated results inserted into the text.
    - `Vec<Diagnostic>`: A list of any errors or warnings found during processing.

---

## Configuration

### `Config` Struct

Controls the behavior of the engine.

```rust
pub struct Config {
    pub reassignment: ReassignmentBehavior,
    pub generate_fills: bool,
    pub constants: HashMap<String, f64>,
}
```

- **`reassignment`**: How the engine handles variable reassignment (e.g., `x = 5` followed by `x = 10`). Defaults to `Warn`.
- **`generate_fills`**: If `true`, the engine will generate "fills" for queries (e.g., `area =`). Defaults to `true`.
- **`constants`**: A map of pre-seeded constants that cannot be reassigned (e.g., `PI`, `E`).

### `ReassignmentBehavior` Enum

```rust
pub enum ReassignmentBehavior {
    Allow,
    Warn,   // Default
    Error,
}
```

---

## Data Models

### `Diagnostic`

Represents an error or warning in the buffer.

```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: Span,
}
```

- **`code`**: A `DiagnosticCode` indicating the type of issue.
- **`span`**: The location in the source text.

### `Span` and `Position`

Used to locate items in the source text.

```rust
pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub offset: u32, // Byte offset
    pub line: u32,   // 0-indexed line number
    pub col: u32,    // 0-indexed column number
}
```

---

## Low-Level Pipeline (Advanced)

If you need more granular control, you can run the pipeline stages manually.

1. **`parse(text, builtins, constants)`**: Performs structural extraction. Returns a `Workspace`.
2. **`distiller(&mut workspace)`**: Assigns roles to atoms (variables, units, etc.).
3. **`janitor(&mut workspace)`**: Scrubs for mathematical sanity and breaks lines.
4. **`unroller(&mut workspace)`**: Flattens the hierarchy into a linear instruction tape.
5. **`executioner(&mut workspace, &config)`**: Evaluates the math using the Fend-core engine.

Alternatively, `evaluate(text, &mut workspace, &config)` runs stages 2 through 5 on an existing `Workspace`.

### `Workspace`

The `Workspace` struct is the "brain" of the engine, containing the interning maps, the topology of the buffer, and all collected metadata.
