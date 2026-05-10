# Engine Stage: Distiller

The Distiller acts as the semantic bridge. It is purely a **typization** stage. It is responsible for looking at the raw strings (IDs) produced by the parser and dressing them in their appropriate semantic clothes (identifying what is a number, a variable, an operator, etc.).

*Crucially, the Distiller does not perform any mathematical grouping, implicit multiplication, or variable assignment. It merely identifies.*

## Contract

*   **Input**: `ParserResult` (Cleaned)
*   **Output**: `SemanticResult` (A topology of `SemanticUnit`s)

## Innards

*   **Typization**: Analyzes every symbol in the `ParserResult` to determine its exact role:
    *   `Quantity`: A numeric literal.
    *   `Variable`: A named identifier used for assignment or retrieval.
    *   `Constant`: A pre-seeded mathematical constant (e.g., PI, TAU).
    *   `PhysUnit`: A valid standalone physical unit recognized by the Numbat registry.
    *   `Function`: A mathematical function (e.g., sin, sqrt).
    *   `Operator`: Structural and mathematical tokens (+, -, =, to, etc.).
    *   `Poison`: A symbol of error injected when typization fatally fails (e.g., malformed numbers or malformed alphanumeric monoliths).
*   **Refinement**: Enriches the topological containers by translating their raw IDs into semantic `SemanticUnit`s, preserving the original groupings and lines for the Unroller.

## Algorithm

The Distiller processes the cleaned `ParserResult` independently for each container. It transforms a flat list of `Unit` IDs into a flat list of `SemanticUnit`s. Because the Janitor removed empty containers and normalized boundaries, the Distiller only deals with meaningful topological groups.

### 1. Initialization
The Distiller is initialized with a context containing `known_units` (extracted from the Numbat registry), `known_functions` (e.g., sin, cos), and `known_constants` (e.g., PI, TAU).

### 2. Typization & Munching
The Distiller iterates through the `Unit`s in a container. When it encounters a nested Container ID, it recursively applies this algorithm and wraps the result in a `SemanticUnit::Group`.

When it encounters a Symbol ID, it uses the **[[ID_SPACE]]** quadrants to categorize the unit with minimal overhead:

1.  **Operator** (ID: `-999,999` to `-1`): Maps the ID directly to an `OpCode`.
2.  **Function** (ID: `<= -1,000,000`): Emits `SemanticUnit::Function(name)`.
3.  **Constant** (ID: `>= 1,000,000`): Emits `SemanticUnit::Constant(name)`.
4.  **The Muncher Fallback** (ID: `1` to `999,999`): If the symbol is a standard dynamic ID, the Distiller retrieves its string and delegates it to the `munch` function (see `Distiller-Number-Muncher.md`). 
    *   Because the parser is "brainless", the Muncher is responsible for splitting alphanumeric strings (e.g., `5kg`), expanding implicit exponents, and resolving units.
    *   The Muncher returns a `Vec<SemanticUnit>` which is spliced directly into the stream.
    *   **The Poison Fallback**: If the Muncher fails (e.g., `1.2.3`), it appends a diagnostic and emits `SemanticUnit::Poison`.


*Note: Once this list is built, the Distiller's job is done. It passes the resulting list directly to the Unroller. Any ambiguity about how a `Quantity` interacts with a neighboring `PhysUnit` is resolved by the Unroller's implicit multiplication rules.*