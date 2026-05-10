# Engine Stage: Parser

The Parser is the entry point of the Crunchie pipeline. It performs a high-performance, single-pass "Brainless Sweep" to transform raw text into a structural topology.

## Contract

*   **Input**: `&str` (The raw buffer)
*   **Output**: `ParserResult` (A map of ID-addressed Containers and Symbols)

## Innards

*   **Unified Identity Space**: Interacts with the **[[ID_SPACE]]** to intern symbols. It uses negative IDs for built-ins and high-positive IDs for constants.
*   **Symbol Interning**: Ensures every unique string (e.g., every instance of `x`) is assigned exactly one ID during the sweep, enabling identity-based comparison in later stages.
*   **Stack-Based Nesting**: Tracks the hierarchy of parentheses using a stack of container IDs.
*   **Comment Capture**: Identifies and stores `#` or `//` comments as a separate list of Spans, ensuring they don't interfere with math while remaining available for UI reporting.

## Algorithm

The Parser is a character-by-character state machine that maintains a stack where `stack[0]` is the Root and `stack[1]` is the current Line.

### 1. Initialization
The parser pre-populates its symbol map with operators, functions, and constants. It initializes the `Root` container (ID 0) and the first `Line` container (ID 1).

### 2. The Sweep
As it consumes characters, it follows these primary rules:
*   **Containment (`(`, `)`)**: On `(`, it allocates a new container ID, pushes it into the current container, and pushes it onto the stack. On `)`, it pops the stack.
*   **Line Breaking (`\n`, `;`)**: At the root level, these trigger a "Boundary Transition"—closing the current line and starting a new one. 
*   **Symbol Flushing**: Any non-structural character (alphanumeric) is accumulated into an `active_sym` buffer. This buffer is "flushed" into the current container whenever a structural character (operator, parenthesis, space) is hit.

### 3. Error Recovery (Poisoning)
If the buffer ends while the stack depth is greater than 2, the parser doesn't crash. It pops the remaining containers and marks them as `corrupted: true`. This "Poison" flag tells the rest of the engine to ignore these branches while still attempting to solve independent lines.
