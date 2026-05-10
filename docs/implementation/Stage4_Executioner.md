# Engine Stage: Executioner

The Executioner is the final stage that performs the actual computation and interacts with the Numbat physics engine.

## Contract

*   **Input**: `Tape`
*   **Output**: `EngineResult`

## Innards

*   **Numbat Bridge**: Iterates through the Tape and converts instructions into calls against the `numbat::Context`.
*   **Poison Propagation**: 
    *   Tracks the "Poison" state of every variable and register.
    *   If an instruction's dependencies are poisoned, the result is poisoned.
*   **Solve Loop**: A single, linear sweep from top to bottom.
*   **Query Resolution**: Identifies "Query" assignments (e.g. `x = `) and captures the calculated value to generate auto-fill edits.
*   **Validation**: 
    *   **Constant Reassignment**: Enforces the `ReassignmentBehavior` configuration. If an assignment instruction targets a `Symbol::Constant`, the Executioner reports a `ConstantReassignment` diagnostic and poisons the operation.
    *   **Value Parity**: For existing assignments, it validates that the current buffer value matches the computed value, reporting a diagnostic if they diverge.
