# Engine Stage: Distiller

The Distiller acts as the semantic bridge. It is purely a **typization** stage. It is responsible for looking at the raw strings produced by the parser and dressing them in their appropriate semantic clothes (identifying what is a number, a variable, an operator, etc.).

*Crucially, the Distiller does not perform any mathematical grouping, implicit multiplication, or variable assignment. It merely identifies.*

## Contract

*   **Input / Output**: `&mut Workspace` (mutated in-place).

## Innards

*   **Typization**: Analyzes every raw symbol in the `Workspace` to determine its exact role:
    *   `Quantity`: A numeric literal.
    *   `Variable`: A named identifier used for assignment or retrieval.
    *   `Constant`: A pre-seeded mathematical constant (e.g., PI, TAU).
    *   `PhysUnit`: A valid standalone physical unit recognized by the Numbat registry.
    *   `Function`: A mathematical function (e.g., sin, sqrt).
    *   `Operator`: Structural and mathematical tokens (+, -, =, to, etc.).
    *   `Poison`: A symbol of error injected when typization fatally fails (e.g., malformed numbers or malformed alphanumeric monoliths).
*   **Refinement**: Enriches the `Workspace` by replacing `Symbol::Raw` entries with their typed equivalents. When a single raw string explodes into multiple symbols (e.g., `10cm3`), it wraps them in a new `Container` and updates the symbol map to point to this `ContainerRef`.

## Algorithm

The Distiller mutates the `Workspace` in-place. Because the Janitor has already cleaned the topology, the Distiller primarily concerns itself with refining the `workspace.symbols` map and only updates `workspace.containers` when an expansion occurs.

### 1. Initialization
The Distiller is initialized with a context containing `known_units` (extracted from the Numbat registry), `known_functions` (e.g., sin, cos), and `known_constants` (e.g., PI, TAU).

### 2. Typization & The Muncher
The Distiller iterates over every entry in `workspace.symbols`. It is looking specifically for the `Symbol::Raw(String)` variants left behind by the Parser.

When it finds a `Symbol::Raw`, it delegates the string to the `munch` function (see `Distiller-Number-Muncher.md`).
*   Because the parser is "brainless", the Muncher is responsible for splitting alphanumeric strings (e.g., `5kg`), expanding implicit exponents, and resolving units.
*   The Muncher returns a `Vec<Symbol>`.

### 3. Graph Updating & Topology Splice
Depending on what the Muncher returns, the Distiller must update the graph:

*   **1:1 Replacement**: If the Muncher returns a single `Symbol` (e.g., `"5"` becomes `[Quantity(5.0)]`), the Distiller simply updates the entry in `workspace.symbols` in-place. The ID remains the same, so the topology in `workspace.containers` does not need to be updated.
*   **1:N Expansion (ContainerRef Replacement)**: If the Muncher returns multiple `Symbols` (e.g., `"10cm3"` becomes `[Quantity(10), PhysUnit("cm"), Operator(Pow), Quantity(3)]`), the Distiller avoids expensive O(N) vector splices by delegating to the existing hierarchy system:
    1.  For each `Symbol` returned by the Muncher, the Distiller asks the `Workspace` to intern it (e.g., `workspace.get_or_intern_symbol(sym)`). This ensures we reuse IDs for known symbols instead of blindly minting new ones.
    2.  It constructs a new `Container` holding these IDs as `Entity`s (preserving the original offset for each) and inserts it into `workspace.containers`.
    3.  Overwrite the original ID in `workspace.symbols` with `Symbol::ContainerRef(new_container_id)`. The Unroller natively flattens `ContainerRef`s, bypassing the need to search or shift `contents` vectors entirely.

*Note: Once all `Raw` symbols are typed, the Distiller's job is done. It passes the mutated `Workspace` directly to the Unroller. Any ambiguity about how a `Quantity` interacts with a neighboring `PhysUnit` is resolved by the Unroller's implicit multiplication rules.*