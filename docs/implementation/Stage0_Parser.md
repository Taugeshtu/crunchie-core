# Engine Stage: Parser

The Parser is the entry point of the Crunchie pipeline. It performs a high-performance, single-pass "Brainless Sweep" to transform raw text into a structural topology.

## Contract

*   **Input**: `&str` (The raw buffer)
*   **Output**: `Workspace` (A map of ID-addressed Containers and Atoms, plus diagnostics and comments)

## Innards

*   **Unified Identity Space**: Interacts with the **[[ID_SPACE]]** to intern atoms. It uses negative IDs for built-ins and high-positive IDs for constants.
*   **Atom Interning**: Uses the `intern_map` to ensure every unique string (e.g., every instance of `x`) is assigned exactly one ID during the sweep.
*   **Stack-Based Nesting**: Tracks the hierarchy of parentheses using a stack of container IDs.
*   **Comment Capture**: Identifies and stores `#` or `//` comments as a separate list of Spans, ensuring they don't interfere with math while remaining available for UI reporting.

## Algorithm

The Parser is a character-by-character state machine that maintains a stack where `stack[0]` is the Root container.

### 1. Initialization
The parser pre-populates its `atoms` and `intern_map` with built-in operators, functions, and constants. It initializes the `Root` container (ID 0).

### 2. The Sweep
As it consumes characters, it follows these primary rules:
*   **Containment (`(`, `)`)**: On `(`, it allocates a new container ID, pushes an **Entity** pointing to an `Atom::Container(id)` into the current container, and pushes the new ID onto the stack. On `)`, it pops the stack.
*   **Line Breaking (`\n`, `;`)**: These are interned as strings. The `Workspace` initialization logic maps these strings to IDs associated with `Atom::Operator` types. The Parser outputs a completely flat Root container; line breaking/splitting is delegated to the **Janitor**. 
*   **Symbol Flushing**: Any non-structural character (alphanumeric) is accumulated into an `active_sym` buffer. This buffer is "flushed" into the current container as an **Entity** whenever a structural character (operator, parenthesis, space) is hit. 
    *   **Resolution**: The parser checks the `intern_map` during the flush. If the symbol was pre-interned (e.g., `pi`, `sin`, or `+`), the Entity points to that existing ID. If not found, a new ID is minted as an `Atom::Raw`.
*   **Number Handling (Exponent Signs)**: If the parser encounters `+` or `-` immediately following an `e` or `E` in an active numeric symbol (e.g., `1.2e-10`), it suppresses the flush and continues accumulating the symbol.

### 3. Error Recovery (Poisoning)
If the buffer ends while the stack depth is greater than 1 (meaning the Root is not the only thing on the stack), the parser doesn't crash. It pops all remaining containers and marks them as `corrupted: true`. This flag tells downstream stages (like the Executioner) to ignore these branches while still attempting to solve independent lines.
