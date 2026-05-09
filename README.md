# Crunchie Core

> "Math is a conversation with a buffer, not a series of buttons on a virtual Casio."

Crunchie is a number-crunching engine built for people who live in text editors and terminal emulators. It rejects the "Calculator App" metaphor in favor of a **Work Management** approach to calculation.

## Documentation

Detailed design documents can be found in the [docs/](./docs/) directory:

- [Vision](./docs/Vision.md): The core philosophy of Crunchie.
- [Syntax](./docs/SYNTAX.md): Structural specification of the language.
- [Parser Design](./docs/PARSER_DESIGN.md): The "Brainless Sweep" internals.
- [Engine Architecture](./docs/ENGINE.md): The multi-stage semantic pipeline.
- [Numbat Integration](./docs/NUMBAT_INTEGRATION.md): How we bridge to the Numbat physics engine.

## Pipeline

1. **Parser**: Single-pass "brainless" sweep converting text to a flat, ID-addressed topology.
2. **Engine**: Semantic evaluation via the [[ENGINE|Engine Pipeline]].
3. **Edits**: Application of insertions/fills back into the source buffer.

## Usage

```rust
let config = crunchie_core::config::Config::default();
let (final_text, diagnostics) = crunchie_core::process_buffer(input, &config);
```
