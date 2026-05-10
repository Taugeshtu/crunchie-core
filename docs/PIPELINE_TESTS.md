# Engine Pipeline Test Cases

This document provides a human-readable "test suite by example" for each stage of the Crunchie Engine. It explains the pipeline's behavior by showing exactly what goes into each stage and what comes out. It serves as a design contract and a foundation for writing automated unit tests.

*Note: For brevity, metadata like `Span` offsets and raw `i32` IDs are omitted. The notation focuses strictly on semantic structure and transformation.*

---

## 1. Parser (Structural Sweep)

The parser is "brainless." It breaks text into symbols based purely on whitespace and specific operator characters (`+`, `-`, `=`, `(`, etc.). It groups symbols into containers based on parentheses and handles the "Twin Rule" for newlines and semicolons.

*   **Case: Basic Separation**
    *   `Input`: `x = 5 kg`
    *   `Output`: `[Sym("x"), Op("="), Sym("5"), Sym("kg")]`
*   **Case: No Separation**
    *   `Input`: `5kg`
    *   `Output`: `[Sym("5kg")]`
*   **Case: Grouping and Operators**
    *   `Input`: `x = 5(1+2)`
    *   `Output`: `[Sym("x"), Op("="), Sym("5"), Group([Sym("1"), Op("+"), Sym("2")])]`
*   **Case: Twin Rule (Root Level)**
    *   `Input`: `x = 1; y = 2`
    *   `Output`: Two lines in Root: `Line 1: [Sym("x"), Op("="), Sym("1")]`, `Line 2: [Sym("y"), Op("="), Sym("2")]`
*   **Case: Twin Rule (Nested Sequence)**
    *   `Input`: `(1; 2\n 3)`
    *   `Output`: `Group([Sym("1"), Op(","), Sym("2"), Op(","), Sym("3")])`
*   **Case: Bad Nesting (Unclosed)**
    *   `Input`: `x = (5`
    *   `Output`: `[Sym("x"), Op("="), Group([Sym("5")]) (Valid: False)]` *(Diagnostic: UnclosedContainer)*
*   **Case: Bad Nesting (Stray)**
    *   `Input`: `x = 5)`
    *   `Output`: `[Sym("x"), Op("="), Sym("5")]` *(Diagnostic: StrayCloser)*

---

## 2. Janitor (Hygiene)

The Janitor scrubs the topological soup produced by the parser. It removes noise and normalizes structure so the Distiller doesn't have to deal with garbage.

*   **Case: Empty Disposal**
    *   `Input`: `[Sym("x"), Group([])]`
    *   `Output`: `[Sym("x")]`
*   **Case: Inert Container Flattening**
    *   `Input`: `[Group([Sym("x")])]`
    *   `Output`: `[Sym("x")]`
*   **Case: Sequence Stuttering**
    *   `Input`: `[Sym("1"), Op(","), Op(","), Sym("2")]`
    *   `Output`: `[Sym("1"), Op(","), Sym("2")]` *(Diagnostic: StraySequence)*
*   **Case: Trailing/Leading Sequences**
    *   `Input`: `Group([Op(","), Sym("5"), Op(",")])`
    *   `Output`: `Group([Sym("5")])` *(Diagnostics: StraySequence)*
*   **Case: Valid Empty Statement**
    *   `Input`: `[Sym("foo"), Group([])]` *(Wait, Janitor discards this. What if we want a function call with no args `foo()`? Actually, if `()` is discarded, we lose the function call indication. We should refine Janitor rules for `()` if needed, but per docs it's currently discarded or kept depending on context. Let's stick to the current doc: "intentionally empty nested container e.g. `()` is valid".)*
    *   `Input`: `Group([])`
    *   `Output`: `Group([])` *(Retained if it represents an empty statement/arg list, overrides basic empty disposal).*

---

## 3. Distiller (Typization)

The Distiller processes the cleaned symbols and determines their roles. It checks if a symbol is a known Operator, Constant, or Function. If not, it hands the raw string to the **Number Muncher** for lexical splitting and numeric evaluation.

*   **Case: Pure Typization**
    *   `Input`: `[Sym("sin"), Group([Sym("PI")])]`
    *   `Output`: `[Function("sin"), Group([Constant("PI")])]`
*   **Case: Muncher (Basic Split)**
    *   `Input`: `[Sym("5"), Sym("kg")]`
    *   `Output`: `[Quantity(5), PhysUnit("kg")]`
*   **Case: Muncher (Monolith Split)**
    *   `Input`: `[Sym("5kg")]`
    *   `Output`: `[Quantity(5), PhysUnit("kg")]`
*   **Case: Muncher (SI Multiplier)**
    *   `Input`: `[Sym("5M")]`
    *   `Output`: `[Quantity(5000000)]`
*   **Case: Muncher (Exponent Expansion)**
    *   `Input`: `[Sym("10cm3")]`
    *   `Output`: `[Quantity(10), PhysUnit("cm"), Op(^), Quantity(3)]`
*   **Case: Muncher (Hexadecimal)**
    *   `Input`: `[Sym("0xFF")]`
    *   `Output`: `[Quantity(255)]`
*   **Case: Muncher (Fallback Identifier)**
    *   `Input`: `[Sym("kg123")]`
    *   `Output`: `[Variable("kg123")]` *(Since kg123 isn't a unit and has no numeric prefix).*
*   **Case: Muncher (Quantity + Identifier)**
    *   `Input`: `[Sym("65kg123")]`
    *   `Output`: `[Quantity(65), Variable("kg123")]`
*   **Case: Muncher (Poison / Bad Parse)**
    *   `Input`: `[Sym("1.2.3")]`
    *   `Output`: `[Poison]` *(Diagnostic: InvalidNumber)*
*   **Case: The `to` Operator**
    *   `Input`: `[Sym("x"), Sym("to"), Sym("cm")]`
    *   `Output`: `[Variable("x"), Op(To), PhysUnit("cm")]`

---

## 4. Unroller (Precedence & Flattening)

TBD. (Will cover Shunting-Yard, Virtual Registers, and strict implicit multiplication rules).

---

## 5. Executioner (Physical Solve)

TBD. (Will cover Numbat Context interactions, Poison propagation, and Query fulfilling).