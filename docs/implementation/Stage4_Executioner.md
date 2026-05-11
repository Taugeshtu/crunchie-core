# Engine Stage: Executioner

The Executioner is the final stage that performs the actual computation and interacts with the Fend-core arithmetic engine.

## Contract

*   **Input**: `Tape`
*   **Output**: `EngineResult`

## Innards

*   **Fend Bridge**: Iterates through the Tape and converts instructions into string expressions evaluated by `fend_core::evaluate`. See [Fend Integration](./FEND_INTEGRATION.md) for details.
*   **Poison Propagation**: 
    *   Tracks the "Poison" state of every variable and virtual register.
    *   If an instruction's dependencies are poisoned, the result is poisoned.
*   **Solve Loop**: A single, linear sweep from top to bottom.
*   **Query Resolution**: Identifies "Query" assignments (e.g. `x = `) and captures the calculated value to generate auto-fill edits.
*   **Validation**: 
    *   **Constant Reassignment**: Enforces the `ReassignmentBehavior` configuration. If an assignment instruction targets a `Symbol::Constant`, the Executioner reports a `ConstantReassignment` diagnostic and poisons the operation.
    *   **Value Parity**: For existing assignments, it validates that the current buffer value matches the computed value, reporting a diagnostic if they diverge.
