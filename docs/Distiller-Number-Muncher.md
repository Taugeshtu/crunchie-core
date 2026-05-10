# Distiller Sub-Routine: The Number Muncher

## The Problem
Because the parser is "brainless", it groups adjacent characters into single strings (`Symbol::Raw`) until it hits an explicit separator (like a space) or an operator. This means `10cm3`, `1_000_356kg`, `5M`, and `0xABCDFFm` arrive at the Distiller as monolithic strings. 

The Distiller processes symbols. If a symbol is raw, the Distiller throws its hands up and says, *"Okay, Muncher, what the fuck is it??"*

The Number Muncher is a dedicated function that takes this raw, unclassified string and returns one or more fully-typed `Symbol` variants (or a `Poison` symbol on error).

## Contract
`fn munch(symbol: &str, known_units: &HashSet<String>) -> Vec<Symbol>`

## Algorithm

The Muncher operates in three distinct phases:

### Phase 1: "Maximal Munch" (The Lexical Split)
The Muncher is a strict, left-to-right character consumer. It attempts to build the longest valid numeric string possible based on a localized rule-set. The moment it encounters a character that violates the numeric rules, it stops. 
*   **The Consumed Chunk** becomes the `Number String`.
*   **The Remaining Chunk** (if any) becomes the `Suffix String`.

**Rulesets (determined by the first two characters):**
*   **Hex (`0x`)**: Consumes `0-9`, `a-f`, `A-F`, `_`.
*   **Bin (`0b`)**: Consumes `0`, `1`, `_`.
*   **Decimal / Scientific (Default)**: Consumes `0-9`, `.`, `_`, `e`, `E`, `+`, `-`.
    *   *Constraint*: `+` and `-` are only valid immediately following an `e` or `E` (e.g., `1e-5`). Any other `+` or `-` stops the muncher.

### Phase 2: Numeric Evaluation
If a `Number String` was successfully isolated, the Muncher attempts to parse it into an `f64`.
*   If the parse fails (e.g., multiple decimal points like `1.2.3`), the Muncher immediately aborts and returns `[Symbol::Poison]`. The engine will flag this with an `MalformedNumber` diagnostic.
*   If the parse succeeds, we now hold a valid `f64` value.

### Phase 3: Suffix Resolution & Expansion
Now the Muncher analyzes the `Suffix String` (if any exists) against the Numbat registry (`known_units`).

1. **No Suffix**: Return `[Quantity(val)]`. (If there was no number either, this case shouldn't be possible as the string would be empty).
2. **SI Unitless Multiplier**: If the suffix is exactly `k`, `K`, or `M`, it acts as a shorthand multiplier. Return `[Quantity(val * multiplier)]`.
3. **Pure Physical Unit**: If the suffix exists in `known_units` (e.g., `kg`), return `[Quantity(val), PhysUnit("kg")]`. *(Note: The Unroller will later resolve these via implicit multiplication).*
4. **Power Suffix Expansion (The `cm3` Rule)**: 
    *   If the suffix ends in a single digit (`2`, `3`, `4`, or `5`), split the suffix into a `prefix` ("cm") and a `power` (3).
    *   If the `prefix` exists in `known_units`, expand the syntax! Return `[Quantity(val), PhysUnit("cm"), Operator(Pow), Quantity(3)]`.
5. **The "Garbage" Fallback**: If the suffix is "garbage" (e.g., `kg123`, `daysofstatic`) and a `Number String` was already consumed, the Muncher aborts. Return `[Symbol::Poison]` and append a `MalformedSymbol` diagnostic. If there was *no* number to begin with (e.g., the original symbol was just `x`), treat the whole string as an identifier. Return `[Variable("x")]`.

## Handling the "Cute Cursed" Cases
Because the Muncher is a complete function that resolves semantics, it handles edge cases perfectly:
*   **Case: `10cm3`** -> Munch isolates `10` and `cm3`. `cm3` expands. Returns `[Quantity(10), PhysUnit("cm"), Operator(Pow), Quantity(3)]`.
*   **Case: `65kg123`** -> Munch isolates `65` and `kg123`. `kg123` is not a valid unit or multiplier. Returns `[Symbol::Poison]` (Diagnostic: `MalformedSymbol`).
*   **Case: `5M`** -> Munch isolates `5` and `M`. Returns `[Quantity(5_000_000)]`.
*   **Case: `x`** -> Munch isolates no number. Returns `[Variable("x")]`.