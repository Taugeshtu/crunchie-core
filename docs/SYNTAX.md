# Crunchie Syntax: Structural Specification

Crunchie uses a minimalist, "One-Pass Sweep" grammar designed for high-performance structural extraction.

## 1. The Unified Identity Space
The parser treats the buffer as a stream of **Units** identified by unique IDs.
- **Symbols**: Text literals (identifiers, numbers, units).
- **Operators**: Self-breaking characters (`+`, `-`, `*`, `/`, `=`, `^`).
- **Containers**: Groups of units, starting with the implicit **Root Container** (ID: 0).

## 2. Structural Triggers

### Containment
- **`(`**: Starts a new nested container. Increases stack depth.
- **`)`**: Closes the current container. Decreases stack depth.
  - *Malformed*: A `)` at Root (Depth 1) is a stray error. A container that never sees a `)` before EOF is marked `.valid = false`.

### The "Twin Rule" (Boundaries vs. Sequence Operators)
The characters `\n` (Newline) and `;` (Semicolon) are functional twins. Their behavior depends strictly on **Stack Depth**:

| Context | Action | Behavior |
| :--- | :--- | :--- |
| **Root (Depth 1)** | **Boundary Trigger** | Ends the current "Line" container and starts a new one. |
| **Nested (Depth 2+)** | **Sequence Operator** | Acts exactly like a comma. The parser homogenizes them and explicitly emits a `,` operator unit. |

### Separators vs Sequence Operators
- **Whitespace (Space, Tab)**: Simply ends the current Symbol/Operator and is completely discarded.
- **Sequence Operators (Comma, Nested Newline, Nested Semicolon)**: End the current Symbol/Operator AND are interned as a structural unit (the `,` operator). 

This distinction allows the Engine to detect missing sequence operators. For example, `(5, 6)` is parsed as `[5, ",", 6]`, which is valid. But `(5 6)` is parsed as `[5, 6]`, allowing the Engine to flag the missing comma.

### Unary Operators
The parser is strictly structural and does not group unary operators. For example, `-5` is parsed as two distinct units: the `-` operator and the `5` symbol. The Engine resolves unary logic based on adjacency during the semantic pass.

### Illegal Characters
The parser explicitly forbids certain characters to reserve them for future use or to prevent ambiguity. Encountering these will emit an `IllegalCharacter` diagnostic, but will not crash the parser.
Forbidden characters: `~`, `` ` ``, `@`, `[`, `]`, `{`, `}`, `\`, `|`.

## 3. Semantic Intent (Engine Layer)
The syntax is purely structural. The **Engine** determines meaning based on the contents of containers:
- `x = 10` -> Assignment (If `x` is new or reassignment is enabled).
- `10 + 5 = 15` -> Assertion.
- `10 + 5 = ` -> Query/Fill.

## 4. Reassignment & Poisoning
- **Reassignment**: Allowed by default but triggers a `Warning` diagnostic. Can be configured to error.
- **Poisoning**: If a container is marked `.valid = false` (unclosed), any symbol defined using that container is "Poisoned." Downstream expressions referencing poisoned symbols will not evaluate, but independent expressions remain valid.

## 5. Comments
- Start with `#` or `//`.
- End at the next newline.
- Stored as a separate list of Spans; ignored by the structural sweep.
