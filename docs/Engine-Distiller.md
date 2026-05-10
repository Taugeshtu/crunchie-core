# Engine Stage: Distiller

The Distiller acts as the semantic bridge. It is responsible for assigning types to the raw strings (IDs) and coupling related concepts like numbers and units into single physical facts.

## Contract

*   **Input**: `ParserResult` (Cleaned)
*   **Output**: `CoupledResult`

## Innards

*   **Typization**: Analyzes every symbol in the `ParserResult` to determine its role:
    *   `Quantity`: Numeric literals (without units, yet).
    *   `Binding`: Named identifiers (variables).
    *   `Constant`: Pre-seeded constants (e.g., PI, TAU).
    *   `PhysUnit`: Valid standalone units from the Numbat registry.
    *   `Function`: Mathematical functions (sin, sqrt).
    *   `Operator`: Structural tokens (+, -, =, etc).
    *   `Poison`: A symbol of error injected when typization fatally fails.
*   **Greedy Snapping**: 
    *   **Binding + PhysUnit**: Merges adjacent tokens into `CoupledUnit::Binding("name", Some("unit"))`.
*   **Refinement**: Enriches the topological containers by translating their raw IDs into semantic `CoupledUnit`s, preserving the original groupings and lines for the Unroller.

## Algorithm

The Distiller processes the cleaned `ParserResult` independently for each container. It transforms a flat list of `Unit` IDs into a flat list of semantic `CoupledUnit`s. Because the Janitor removed empty containers and normalized boundaries, the Distiller only deals with meaningful topological groups.

### 1. Initialization
The Distiller is initialized with context containing `known_units` (extracted from the Numbat registry), `known_functions` (e.g., sin, cos), and `known_constants` (e.g., PI, TAU).

### 2. Typization & Munching
The Distiller iterates through the `Unit`s in a container. When it encounters a nested Container ID, it recursively applies this algorithm and wraps the result in a `CoupledUnit::Group`.

When it encounters a Symbol ID, it retrieves the original string and categorizes it in order:
1.  **Operator**: If the token matches a known operator string (`+`, `-`, `=`, `,`), it emits `CoupledUnit::Operator(OpCode)`.
2.  **Constant**: If it matches a name in `known_constants`, it emits `CoupledUnit::Constant(name)`.
3.  **Function**: If it matches a name in `known_functions`, it emits `CoupledUnit::Function(name)`.
4.  **The Muncher Fallback**: If the symbol is none of the above, the Distiller delegates it to the `munch` function (see `Distiller-Number-Muncher.md`). 
    *   The Muncher is responsible for splitting alphanumeric strings, expanding implicit exponents (like `cm3`), applying SI multipliers (like `5M`), and parsing valid quantities. 
    *   The Muncher returns a `Vec<CoupledUnit>` which is spliced directly into the Distiller's growing sequence.
    *   **The Poison Fallback**: If the Muncher fatally fails (e.g., trying to parse `1.2.3`), it appends an `InvalidNumber` diagnostic and emits `CoupledUnit::Poison` into the stream.

### 3. Greedy Snapping (Coupling)
The Distiller performs one final left-to-right sweep over the typized list to resolve adjacencies:
*   **Binding + PhysUnit**: If a `Binding(name, None)` is immediately followed by a `PhysUnit(u)`, they are replaced by a single `Binding(name, Some(u))`.

This step prevents the Unroller from panicking at adjacent operands and correctly packages declarations (like `x kg`). Everything else—including quantities, standalone `PhysUnit`s, operators, and functions—is passed through unchanged to let standard precedence (implicit multiplication) handle the math (e.g. `5 cm` -> `5 * cm`).
