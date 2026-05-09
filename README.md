# Crunchie Core

> "Math is a conversation with a buffer, not a series of buttons on a virtual Casio."

Crunchie is a number-crunching engine built for people who live in text editors and terminal emulators. It rejects the "Calculator App" metaphor in favor of a **Work Management** approach to calculation.

## Pipeline

1. **Parser**: Single-pass "brainless" sweep converting text to a flat, ID-addressed topology.
2. **Engine**: Semantic evaluation, linting, and solve-state calculation (WIP).
3. **Edits**: Application of insertions/fills back into the source buffer.

## Usage

```rust
let config = crunchie_core::config::Config::default();
let (final_text, diagnostics) = crunchie_core::process_buffer(input, &config);
```
