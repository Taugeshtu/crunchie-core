# Crunchie Core

> "Math is a conversation with a buffer, not a series of buttons on a virtual Casio."

Crunchie is a small DSL for everyday light arithmetic, in a text buffer.
Why? Because it's never *just* "PI times five". You'll need more than one step of calculation. You need context, comments, and variables. Crunchie turns any text buffer into a strict, physically-aware scratchpad where math happens as you think and type.

## The DSL: A Conversation with a Buffer

Crunchie's syntax is minimal, completely rejecting the need for an "Eval" or "Clear" button. The engine interprets your intent based on how you write your math:

```text
# 1. Variables and native physical units
radius = 10cm
mass = 5 kg

# 2. Assertions (Errors if not mathematically true)
10m + 50cm = 10.5m

# 3. Queries (The engine calculates and auto-fills)
area = PI * radius^2
area =               // Evaluates to: 314.159... cm^2
area to mm^2 =       // Evaluates to: 31415.9... mm^2

# 4. Strict Physics
5cm + 3     // Poisoned! Cannot mix dimensioned and unitless.
5kg + 10m   // Poisoned! Cannot add mass and length.
```

If you make a mistake, Crunchie **"Poisons"** that specific line and anything dependent on it, but leaves independent math untouched. One typo won't ruin your whole session.

## Documentation

The detailed architectural specifications can be found in the [`docs/`](./docs/) directory:

- **[Vision](./docs/Vision.md)**: The core philosophy of Crunchie.
- **[Syntax Guide](./docs/SYNTAX.md)**: How to write math in Crunchie.
- **[Workspace Data Model](./docs/implementation/Workspace-Data-Model.md)**: The unified flat graph that powers the engine.
- **[Pipeline Tests](./docs/implementation/PIPELINE_TESTS.md)**: A human-readable test suite for each pipeline stage.

## The Pipeline

Crunchie-core is a pure, portable library that uses a progressive, linear pipeline to transform text into math. Instead of walking recursive ASTs, it manipulates a flat, cache-friendly array of entities:

1. **[Parser (Stage 0)](./docs/implementation/Stage0_Parser.md)**: A "brainless" sweep turning raw text into a structural topology.
2. **[Janitor (Stage 1)](./docs/implementation/Stage1_Janitor.md)**: Scrubs the topological soup, breaks lines, normalizes sequences.
3. **[Distiller (Stage 2)](./docs/implementation/Stage2_Distiller.md)**: The semantic bridge; uses the "Number Muncher" to parse units, multipliers, and numbers.
4. **[Unroller (Stage 3)](./docs/implementation/Stage3_Unroller.md)**: Flattens the hierarchy into a linear tape of instructions using the Shunting-Yard algorithm.
5. **[Executioner (Stage 4)](./docs/implementation/Stage4_Executioner.md)**: The final pass that talks to the Numbat physics engine to solve the tape.

## Usage

Crunchie-core doesn't know about the disk or the OS. It only knows about the buffer you give it.

```rust
use crunchie_core::config::Config;
use crunchie_core::process_buffer;

let config = Config::default();
let (final_text, diagnostics) = process_buffer(input_text, &config);
```
