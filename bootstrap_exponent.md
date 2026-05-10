# Parser Patch: Exponential Number Support

**The Problem:**
Currently, the "brainless" parser is a strict single-pass scanner that treats characters like `+` and `-` universally as boundaries/operators. Because of this, if a user types `1e-5`, the parser breaks it into three distinct units:
1. `1e` (Symbol)
2. `-` (Operator)
3. `5` (Symbol)

If we push the responsibility of fixing this to the Distiller, we run into a major ambiguity: we might silently "fix" invalid spacing like `3e - 5`, assuming the user meant an exponent when they actually meant subtraction involving Euler's number ($3e - 5$).

**The Hand-off Task:**
We need to introduce a tiny, localized exception in the Parser's structural sweep (`src/parser.rs`):

When encountering a `+` or `-` operator:
1. Peek at the `active_sym` buffer.
2. If `active_sym` ends with `e` or `E` **and** the buffer consists of a valid numeric prefix (e.g., it contains digits, optionally a decimal), do **not** flush the symbol.
3. Instead, consume the `+` or `-` directly into the `active_sym`.

By handling this during the topological sweep, `1e-5kg` is emitted as a single string symbol. The Distiller's Number Muncher can then safely assume that any `+` or `-` it sees inside a single symbol was typed without spaces and is legitimately part of an exponent.
