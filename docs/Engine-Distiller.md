# Engine Stage: Distiller

The Distiller acts as the semantic bridge. It is responsible for assigning types to the raw strings (IDs) and coupling related concepts like numbers and units into single physical facts.

## Contract

*   **Input**: `ParserResult` (Cleaned)
*   **Output**: `CoupledResult`

## Innards

*   **Typization**: Analyzes every symbol in the `ParserResult` to determine its role:
    *   `Number`: Numeric literals.
    *   `Variable`: Named identifiers.
    *   `PhysUnit`: Valid units from the Numbat registry.
    *   `Function`: Mathematical functions (sin, sqrt).
    *   `Operator`: Structural tokens (+, -, =, etc).
*   **Greedy Coupling**: 
    *   **Literal + Unit**: Merges `5` and `kg` into a single `CoupledUnit::Quantity(5kg)`.
    *   **Variable + Unit**: Merges `x` and `kg` into `CoupledUnit::Binding("x", Some("kg"))`.
*   **Refinement**: Produces a structure where the original ID-based topology is replaced by a sequence of high-level `CoupledUnit` enums.
