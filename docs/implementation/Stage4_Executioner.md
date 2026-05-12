# Engine Stage: Executioner

The Executioner is the final stage that performs the actual computation and interacts with the Fend-core arithmetic engine.

## Contract

*   **Input**: `Tape` (via `Workspace`)
*   **Output**: `EngineResult` (Diagnostics and TextEdits)

## Innards

*   **Native AST Bridge**: Instead of serializing instructions back into strings for Fend to re-parse, the Executioner directly constructs and evaluates `fend_core::ast::Expr` trees for each instruction. This guarantees structural fidelity and leverages Fend's internal evaluation capabilities.
*   **Virtual Registers**: The evaluated `fend_core::value::Value` from each instruction is temporarily cached in an Executioner side-table (a map of `Instruction ID -> Value`). Subsequent instructions that depend on previous results dynamically inject these values into the AST as `Expr::Literal(val)`.
*   **Solve Loop**: A single, linear sweep from top to bottom (Line 1 to Line N).
*   **Poison Propagation**: Tracked via a side-table (`HashSet<i32>`). If an instruction references an ID that is `Poisoned`, it immediately aborts and becomes `Poisoned` itself.

## The Algorithm

### 1. Initialization
1.  Create a fresh `fend_core::Context`.
2.  Initialize a `state_map: HashMap<i32, fend_core::value::Value>` to track the computed result of each instruction.
3.  Initialize a `poison_set: HashSet<i32>` to track failed evaluation paths.

### 2. The Sweep
Iterate over all Line containers in the `Workspace` (ordered by offset):

1.  **Line Poison Check**: If the line container is `corrupted` or contains an `Atom::Poison`, add the line's instructions to `poison_set` and continue to the next line.
2.  **Tape Execution**: Iterate through the `Entity`s in the Tape.
    *   **Base Atoms (`Value`, `Variable`, `Constant`)**: 
        *   Extract their values.
        *   Values become `Expr::Literal(val)`.
        *   Variables and Constants become `Expr::Ident(Ident::new(string))`.
    *   **Instructions (`OpCode`)**:
        *   Retrieve the `Expr` representation for all arguments from the `state_map` (as `Literal`s) or from base atoms.
        *   If any argument is in `poison_set`, add this instruction ID to `poison_set` and halt execution for this line.
        *   Construct the corresponding `fend_core::ast::Expr` (e.g., `Expr::Bop(Bop::Plus, lhs, rhs)`).
        *   Execute via Fend: `fend_core::ast::evaluate(expr, ...)`.
        *   If Fend returns an Error, emit an `EvaluationError` diagnostic, add to `poison_set`, and halt the line.
        *   If successful, store the returned `Value` in `state_map` under the instruction's ID.

### 3. Intent Resolution (Queries & Assertions)
The final instruction of a Line tape determines the user's intent.

*   **Assignments (`x = expr`)**:
    *   Identified by an `Equals` instruction with a Variable on the LHS.
    *   Evaluates `Expr::Assign(Ident::new("x"), Expr)` in the Fend Context.
    *   If the variable is actually a predefined `Constant` (e.g., `PI = 3`), emit a `ConstantReassignment` diagnostic and poison.
*   **Queries (`expr = `)**:
    *   Identified by an `Equals` instruction with a missing right-hand argument.
    *   Evaluate the left side.
    *   Generate a `TextEdit` to insert the stringified result into the text buffer.
*   **Assertions (`expr1 = expr2`)**:
    *   Identified by an `Equals` instruction where the LHS is an expression (not a solitary variable).
    *   Evaluate `Expr::Equality(true, Expr1, Expr2)`.
    *   If the result is `false`, emit an `AssertionFailed` diagnostic.
