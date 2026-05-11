# Fend Integration

Crunchie uses `fend-core` as its arithmetic engine. 

## The Executioner Loop

The Stage 4 Executioner iterates through the linear `Tape` of instructions. 

### Register Management
- Every instruction in the Tape targets a "virtual register."
- We maintain a mapping between these registers and `fend-core::Context` variables.
- When an instruction is executed:
    1. We serialize the operation into a Fend-compatible string (e.g. `5 + 2`).
    2. We call `fend_core::evaluate(string, &mut context)`.
    3. We store the result back into our register tracking.

### Poison Propagation
- If `fend_core::evaluate` returns an error, we mark the target register as **Poisoned**.
- Any future instructions that depend on a poisoned register are skipped and their results are also marked as poisoned.

### Queries and Edits
- For Query assignments (e.g. `x = `), the Executioner captures the result of the Fend evaluation and generates a `TextEdit` to be applied to the buffer.
