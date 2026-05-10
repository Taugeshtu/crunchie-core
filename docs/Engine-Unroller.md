# Engine Stage: Unroller

The Unroller transforms the hierarchical, parenthetical structure of the "SemanticResult" into a flat, linear "Tape" of instructions.

## Contract

*   **Input**: `SemanticResult`
*   **Output**: `Tape` (The Bytecode)

## Innards

*   **Precedence Resolution**: Implements a shunting-yard style algorithm to handle order of operations (e.g. multiplication before addition) without needing a recursive AST.
*   **Flattening**: Recursively walks nested groups and unrolls them into the final instruction sequence.
*   **Register Allocation**: Manages intermediate calculation results by assigning them virtual "Register" IDs. This allows the Executioner to be a simple, non-stack-based loop.
*   **Provenance Mapping**: Maintains parity between the linear instructions and the original source Spans for accurate error reporting.
