# Engine Stage: Unroller

The Unroller transforms the hierarchical, parenthetical structure of a `Container` into a flat, linear sequence of instructions. In the Unified Workspace model, it operates by reading `Entity` IDs, looking up their `Symbol` meanings, and minting new `Instruction` symbols back into the workspace.

## Contract

*   **Input**: `Workspace` (Distilled)
*   **Output**: `Workspace` (Unrolled)

## Innards

*   **Precedence Resolution**: A Shunting-Yard algorithm that handles the order of operations without a recursive AST.
*   **Implicit Multiplication**: In-flight detection of "Bumping Elbows" between adjacent semantic entities.
*   **Register Allocation**: Manages intermediate results by minting new `Instruction` symbols, allowing the Executioner to be a simple, non-stack-based loop.
*   **Provenance Preservation**: Copies the `offset` from the source entities to the generated instruction entities for accurate error reporting.

## Algorithm

The Unroller iterates through the direct children of the Root container (the Lines). For each Line, it performs a two-pass transformation. 

*Note: The Unroller assumes the Distiller has successfully converted all `Symbol::Raw` entries into typed symbols (Quantity, Variable, PhysUnit, Constant, Function, or Operator). Any remaining `Raw` strings are treated as fatal errors.*

### 1. The Precedence Pass (Shunting-Yard)
The Unroller processes a line's `Entity` list left-to-right to resolve mathematical priority. It maintains an **Operator Stack** (IDs of operators) and an **Output Queue** (IDs of operands and resolved operators).

**Precedence Levels (High to Low):**
1. `^` (Power)
2. `*`, `/`, `Implicit Mul`
3. `+`, `-`
4. `to` (Conversion)
5. `=` (Assignment/Assertion/Query)

**Bumping Elbows (Implicit Multiplication):**
Between every two units `A` and `B`, the Unroller checks for a missing operator. If it detects one of the following pairs, it synthetically injects a `*` (Precedence 2) into the Operator Stack before processing `B`:
*   `Quantity` + `PhysUnit` (e.g., `5 cm`)
*   `Quantity` + `Variable` (e.g., `5x`)
*   `Quantity` + `Constant` (e.g., `5 PI`)
*   `Quantity` + `ContainerRef` (e.g., `5(1+2)`)
*   `ContainerRef` + `ContainerRef` (e.g., `(2)(3)`)

**The Sorting Dance:**
*   **Operands** (`Quantity`, `Variable`, `Constant`, `PhysUnit`): Push ID directly to the Output Queue.
*   **Nested Containers (`ContainerRef`)**: Recursively process the inner container. The resulting RPN sequence is spliced into the current Output Queue.
*   **Operators**: 
    1. While the operator on top of the Stack has **higher or equal** precedence than the current operator, pop the Stack to the Output Queue.
    2. Push the current operator onto the Stack.
*   **Finalization**: Pop any remaining operators from the Stack to the Output Queue.

### 2. The Instruction Pass (Tape Generation)
The Unroller iterates through the flat RPN Queue and uses a **Value Stack** (IDs of operands or previous instructions) to mint the final linear bytecode.

1.  **Process RPN Queue**:
    *   **If Operand ID**: Push to the Value Stack.
    *   **If Operator ID**:
        *   Pop the required number of operand IDs (e.g., `left`, `right`).
        *   Mint a new `Symbol::Instruction { op, args: [left, right] }` in `Workspace.symbols`.
        *   Store the provenance `offset` from the operator entity.
        *   Push the `new_id` of this instruction back onto the Value Stack.
2.  **Commit the Tape**:
    *   The Unroller creates a new `Container` to hold the linear sequence.
    *   It populates this container's `contents` with the `Entity` IDs of the instructions in the order they were minted.
    *   The original line's `ContainerRef` is updated to point to this new "Tape" container.

### 3. Special Logic: The Big Cleaver (`=`)
The `=` operator acts as the absolute lowest precedence marker. 
*   **Assignment**: If the LHS of the final instruction is a single `Variable` and the operator is `=`, the Unroller marks this instruction as an `Assignment` in the `Workspace`.
*   **Queries**: If an `=` is followed by nothing (end of line), the Unroller identifies the last register ID on the stack and flags it as a `Query` for the Executioner to fill.

### 4. Errors & Diagnostics
The Unroller appends `MalformedExpression` diagnostics to the `Workspace` if:
*   An operator is missing operands (e.g., `5 + `).
*   The stack contains multiple values at the end of a line (missing operators).
*   An invalid "Bumping Elbows" pair is detected (e.g., `Variable` + `Quantity`).
