# Engine Stage: Unroller

The Unroller transforms the hierarchical, parenthetical structure of a `Container` into a flat, linear sequence of instructions (the **Tape**). It resolves mathematical precedence and allocates **Virtual Registers** using the Workspace's ID space.

## Contract

*   **Input / Output**: `&mut Workspace`.
*   **Mutation**: Mints new `Atom::Instruction` entries and replaces Line containers with "Tape" containers.

## Innards

*   **Instruction IDs as Registers**: Every minted `Atom::Instruction` is assigned a new ID. This ID serves as a **Virtual Register**. Downstream instructions that depend on a previous result simply use that instruction's ID as an argument.
*   **Implicit Multiplication**: In-flight detection of "Bumping Elbows" between adjacent entities (e.g., `5x`).
*   **Variable Preservation**: Assignments (e.g., `x = 5`) do **not** mutate the `Atom::Variable` into a value. Instead, they produce an `Equals` instruction. The actual value of the variable is managed by the Executioner's transient context during the solve loop.

## Algorithm

The Unroller processes each Line container in the Root. For each line, it performs a recursive RPN conversion followed by instruction minting.

### 1. The Precedence Pass (Recursive RPN)
The Unroller uses a Shunting-Yard algorithm to resolve order of operations. 

**Precedence Levels (High to Low):**
1. `Call` (Function invocation)
2. `^` (Power)
3. `*`, `/`, `Mod`
4. `+`, `-`
5. `to` (Conversion)
6. `=`, `+=`, `-=`, etc. (Assignments)
7. `,` (Comma / Argument separator)

**Bumping Elbows (Implicit Multiplication):**
Between every two entities `A` and `B`, the Unroller checks for a missing operator. If it detects one of the following pairs, it synthetically injects a `*` (Precedence 3) into the flow:
*   `Value` + `Container` (e.g., `5(1+2)`)
*   `Value` + `Variable` (e.g., `5x`)
*   `Value` + `Constant` (e.g., `5 PI`)
*   `Container` + `Container` (e.g., `(2)(3)`)
*   `Container` + `Variable` (e.g., `(2)x`)

**The Sorting Dance**:
*   **Operands** (`Value`, `Variable`, `Constant`): Push to the Output Queue.
*   **Nested Containers**: Recursively call the RPN logic. The resulting sequence is spliced into the current Output Queue.
*   **Functions**: Pushed to the Operator Stack.
*   **Operators**: 
    1. While the operator on top of the Stack has **higher or equal** precedence, pop it to the Output Queue.
    2. Push the current operator onto the Stack.

### 2. The Instruction Pass (Tape Generation)
The Unroller iterates through the flat RPN Queue and uses a **Value Stack** (IDs of operands or previous instructions) to mint the final linear bytecode.

1.  **Consume RPN**:
    *   **If Operand ID**: Push to the Value Stack.
    *   **If Operator/Function ID**:
        *   Pop the required number of IDs from the Value Stack (e.g., `left`, `right`).
        *   Mint a new `Atom::Instruction { op, args: [left, right] }`.
        *   Assign it a new ID and push that ID back onto the Value Stack.
2.  **Commit the Tape**:
    *   A new **Tape Container** is created to hold the sequence of instruction IDs in the order they were minted.
    *   The original line's `Atom::Container` pointer is updated to point to this new Tape.

### 3. Queries and Assignments
The Unroller identifies the "Intent" of a line by looking at the final instruction:
*   **Assignment**: An `Equals` instruction where the first argument is an `Atom::Variable`.
*   **Query**: An instruction or value that stands alone without being assigned to a variable. The Executioner uses the ID of the final item on the Value Stack as the "Result Register" to report back to the UI.

## Errors & Diagnostics
The Unroller identifies the following errors:
*   **`MalformedExpression`**:
    *   The Value Stack is empty when an operator expects an argument.
    *   The Value Stack has multiple items remaining at the end of a line (e.g., `5 10 + 2` leaves `5` and `(10+2)` on the stack).
    *   The "Bumping Elbows" logic encounters an illegal pair (e.g., `Variable` + `Value`).
*   **`ArgumentsMismatch`**:
    *   The number of arguments provided to a `Call` instruction does not match the arity of the `Atom::Function` (e.g., `sin(1, 2)` or `max(1)`).
