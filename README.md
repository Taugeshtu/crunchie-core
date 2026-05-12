> "Math is a conversation with a buffer, not a series of buttons on a virtual Casio."

Crunchie is a small DSL for everyday light arithmetic, in a text buffer, as a library.

## Why?

Because it's never _just_ "pi times five". It's area in `cm2` and length in meters; and there's an array of these things, and they are 20% air and 80% aluminium, and you need the total mass. And sometimes you need to noodle around the parameters so the solution fits better.

Capability niche: **python/spreadsheet** > _**Crunchie**_ > **calculator app**

"Math in rust" niche:
- just a bit more "language" than [Fend](https://github.com/printfn/fend)
- a fair bit less "language" than [Numbat](https://github.com/sharkdp/numbat)

Crunchie provides:
- **Gradual capability**: from just linting symbols and parentheses, through validating equations and calculating solutions, to a completely filled & annotated buffer. Use as much or as little of the pipeline as you want, hack into it
- **Poisoning**: If Line 1 is a syntax error, Line 1 is "Poisoned," but Line 2 keeps working. Independent math stays alive.

_More details in the **[Vision doc](./docs/Vision.md)**._

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

_More details in the **[Syntax Guide](./docs/SYNTAX.md)**._

## The innards

Crunchie-core is a pure, portable library that uses a progressive, linear pipeline to transform text into math. Instead of walking recursive ASTs, it manipulates a flat, cache-friendly array of entities. It should be easy to grok what each stage of the pipeline does by looking into:
- **[Pipeline Tests](./docs/implementation/PIPELINE_TESTS.md)**: A human-readable test suite for each pipeline stage
- **[Workspace Data Model](./docs/implementation/Workspace-Data-Model.md)**: The unified flat graph that powers the engine

More in-depth docs which are driving the source:
1. **[Parser (Stage 0)](./docs/implementation/Stage0_Parser.md)**: A "brainless" sweep turning raw text into a structural topology.
2. **[Distiller (Stage 1)](./docs/implementation/Stage1_Distiller.md)**: The semantic bridge; uses the "Number Muncher" to parse units, multipliers, and numbers.
3. **[Janitor (Stage 2)](./docs/implementation/Stage2_Janitor.md)**: Scrubs the topological soup, breaks lines, normalizes sequences.
4. **[Unroller (Stage 3)](./docs/implementation/Stage3_Unroller.md)**: Flattens the hierarchy into a linear tape of instructions using the Shunting-Yard algorithm.
5. **[Executioner (Stage 4)](./docs/implementation/Stage4_Executioner.md)**: The final pass that talks to the Fend-core arithmetic engine to solve the tape.

## Usage

Crunchie-core doesn't know about the disk or the OS. It only knows about the buffer you give it. See the **[API Reference](./docs/API.md)** for more details.

```rust
use crunchie_core::config::Config;
use crunchie_core::process_buffer;

let config = Config::default();
let (final_text, diagnostics) = process_buffer(input_text, &config);
```

