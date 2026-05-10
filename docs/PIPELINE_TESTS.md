# Engine Pipeline Test Cases

This document provides a human-readable "test suite by example" for each stage of the Crunchie Engine. It explains the pipeline's behavior by showing exactly what goes into each stage and what comes out. It serves as a design contract and a foundation for writing automated unit tests.

*Note: For brevity, metadata like `Span` offsets and raw `i32` IDs are omitted. The notation focuses strictly on semantic structure and transformation.*

---

## 1. Parser (Structural Sweep)

The parser is "brainless." It only knows about **Symbols** (interned strings) and **Containers** (flat lists of IDs). It doesn't know what a number or an operator is yet; it only knows about IDs.

*   **Case: Basic Separation**
    *   `Input`: `x = 5 kg`
    *   `Symbols`: `{"x": 1, "=": -1, "5": 2, "kg": 3}`
    *   `Output`: 
        *   `Root (0)` contains `Line (4)`
        *   `Line (4)` contains `[ID:1, ID:-1, ID:2, ID:3]`
*   **Case: No Separation (Monoliths)**
    *   `Input`: `5kg`
    *   `Symbols`: `{"5kg": 1}`
    *   `Output`: 
        *   `Root (0)` contains `Line (2)`
        *   `Line (2)` contains `[ID:1]`
*   **Case: Parenthetical Nesting**
    *   `Input`: `(1+2)`
    *   `Symbols`: `{"1": 1, "+": -2, "2": 2}`
    *   `Output`: 
        *   `Root (0)` contains `Line (3)`
        *   `Line (3)` contains `Container (4)`
        *   `Container (4)` contains `[ID:1, ID:-2, ID:2]`
*   **Case: Twin Rule (Root Level)**
    *   `Input`: `x = 1; y = 2`
    *   `Output`: 
        *   `Root (0)` contains `Line (4), Line (5)`
        *   `Line (4)` contains `[x, =, 1]`
        *   `Line (5)` contains `[y, =, 2]`
*   **Case: Twin Rule (Nested Sequence)**
    *   `Input`: `(1; 2)`
    *   `Symbols`: `{"1": 1, ";": -8, "2": 2}`
    *   `Output`: 
        *   `Container (3)` contains `[ID:1, ID:-8, ID:2]` *(Note: Semicolon is just another ID here)*
*   **Case: Bad Nesting (Unclosed)**
    *   `Input`: `(5`
    *   `Output`: `Container (3)` contains `[ID:1]`. `Container(3).corrupted = true`.
*   **Case: Bad Nesting (Stray)**
    *   `Input`: `5)`
    *   `Output`: `Line (1)` contains `[ID:1]`. *(Diagnostic: StrayCloser)*

---

## 2. Janitor (Hygiene)

The Janitor scrubs the topological soup. It flattens "Inert" containers (those with exactly 1 child), and normalizes sequence markers (turning `;` and `\n` IDs into a canonical `,` operator ID).

*   **Case: Inert Container Flattening**
    *   `Input`: `Line(1)` contains `[ID:2 (Inert Container)]`, `Container(2)` contains `[ID:1 (x)]`
    *   `Output`: `Line(1)` contains `[ID:1]`
*   **Case: Sequence Normalization**
    *   `Input`: `Container(1)` contains `[ID:1 (1), ID:-8 (;), ID:-8 (;), ID:2 (2)]`
    *   `Output`: `Container(1)` contains `[ID:1, ID:-7 (,), ID:2]` *(Diagnostic: StraySequence)*
*   **Case: Sequence Trimming**
    *   `Input`: `Container(1)` contains `[ID:-7 (,), ID:1 (x), ID:-9 (\n)]`
    *   `Output`: `Container(1)` contains `[ID:1]` *(Diagnostics: StraySequence)*
*   **Case: Healthy Empty Statement**
    *   `Input`: `Container(1)` is empty but marked as a healthy statement (e.g., `()`)
    *   `Output`: `Container(1)` is preserved.

---

## 3. Distiller (Typization)

The Distiller processes the cleaned contents of a single container at a time. It dresses strings in semantic clothes (Variable, Quantity, etc.) and uses the **Number Muncher** for lexical splitting.

*   **Case: Basic Typization**
    *   `Input`: `(sin, ())`
    *   `Output`: `[Function("sin"), Group(...)]`
*   **Case: Muncher (Split Symbols)**
    *   `Input`: `(5, kg)`
    *   `Output`: `[Quantity(5), PhysUnit("kg")]`
*   **Case: Muncher (Monolith Split)**
    *   `Input`: `(5kg)`
    *   `Output`: `[Quantity(5), PhysUnit("kg")]`
*   **Case: Muncher (SI Multiplier)**
    *   `Input`: `(5M)`
    *   `Output`: `[Quantity(5000000)]`
*   **Case: Muncher (Exponent Expansion)**
    *   `Input`: `(10cm3)`
    *   `Output`: `[Quantity(10), PhysUnit("cm"), Operator(Pow), Quantity(3)]`
*   **Case: Muncher (Quantity + Identifier)**
    *   `Input`: `(65kg123)`
    *   `Output`: `[Poison]` *(Diagnostic: MalformedSymbol)*
*   **Case: Muncher (Hexadecimal)**
    *   `Input`: `(0xFF)`
    *   `Output`: `[Quantity(255)]`
*   **Case: Muncher (Fallback Variable)**
    *   `Input`: `(kg123)`
    *   `Output`: `[Variable("kg123")]`
*   **Case: Poisoning**
    *   `Input`: `(1.2.3)`
    *   `Output`: `[Poison]` *(Diagnostic: MalformedNumber)*
*   **Case: Operators**
    *   `Input`: `(x, =, (), *, 5)`
    *   `Output`: `[Variable("x"), Operator(Assign), Group(...), Operator(Mul), Quantity(5)]`
*   **Case: Conversion Operator**
    *   `Input`: `(x, to, cm)`
    *   `Output`: `[Variable("x"), Operator(To), PhysUnit("cm")]`
*   **Case: Query Assignment**
    *   `Input`: `(x, =)`
    *   `Output`: `[Variable("x"), Operator(Assign)]`

---

## 4. Unroller (Precedence & Flattening)

The Unroller flattens the hierarchical SemanticUnits into a linear "Tape" of instructions. It uses the Shunting-Yard algorithm to handle precedence and virtual registers (`r0`, `r1`, etc.) for intermediate results.

*   **Case 1: Implicit Multiplication**
    *   `Input`: `[Quantity(3), Group([Quantity(1), Operator(+), Quantity(4)])]`
    *   `Logic`: The Unroller sees a Number bumping a Group and injects a `Mul` operator.
    *   `Output Tape`:
        *   `r0 = Add(Quantity(1), Quantity(4))`
        *   `r1 = Mul(Quantity(3), r0)`
*   **Case 2: Assignment & Precedence**
    *   `Input`: `[Variable("x"), Operator(Assign), Quantity(10), Operator(Div), Quantity(2), Operator(Add), Quantity(5)]`
    *   `Logic`: Division happens before Addition. Assignment happens last.
    *   `Output Tape`:
        *   `r0 = Div(Quantity(10), Quantity(2))`
        *   `r1 = Add(r0, Quantity(5))`
        *   `Assign(target: "x", value: r1)`
*   **Case 3: Units and Conversion**
    *   `Input`: `[Variable("y"), Operator(Assign), Group([Quantity(1), PhysUnit("m"), Operator(+), Quantity(10), PhysUnit("cm")]), Operator(To), PhysUnit("mm")]`
    *   `Logic`:
        1. `1m` and `10cm` are expanded via implicit multiplication.
        2. The group result is passed to the `To` operator.
    *   `Output Tape`:
        *   `r0 = Mul(Quantity(1), PhysUnit("m"))`
        *   `r1 = Mul(Quantity(10), PhysUnit("cm"))`
        *   `r2 = Add(r0, r1)`
        *   `r3 = To(value: r2, target_unit: "mm")`
        *   `Assign(target: "y", value: r3)`
*   **Case 4: Query Assignment**
    *   `Input`: `[Variable("x"), Operator(Assign)]`
    *   `Logic`: The Unroller sees an assignment with a missing right-hand side. It emits the instruction for the Executioner to handle as a "Query".
    *   `Output Tape`:
        *   `Query(target: "x")`

---

## 5. Executioner (Physical Solve)

TBD. (Will cover Numbat Context interactions, Poison propagation, and Query fulfilling).
