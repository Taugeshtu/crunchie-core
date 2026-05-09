# Numbat for Crunchie: The Semantic Bridge

> "Crunchie provides the structure; Numbat provides the soul."

This document outlines the strategy for using Numbat's physics engine as the solver for `crunchie-core`, bypassing the Numbat language/VM while retaining its dimensional IQ.

## 1. The Core Philosophy
Crunchie is a **Data-Oriented Transformer**. It turns text into IDs. 
Numbat is a **Physically-Aware Solver**. It turns Quantities into Results.

The Bridge (Engine) maps Crunchie's `ParserResult` (Flat IDs) to Numbat's `Quantity` structs, executes math, and maps results back to Crunchie's `EngineResult`.

---

## 2. Bootstrapping: The "Unit Heist"
Numbat's knowledge of the world (what is a `meter`? what is a `fortnight`?) is stored in its `prelude.nbt`. We don't want to re-parse this on every keystroke.

**Plan:**
1.  Initialize a `numbat::Context` once.
2.  Load the prelude.
3.  **Extract the `UnitRegistry` and `DimensionRegistry`**.
4.  Cache these in our `crunchie::Engine`.

### Snippet: The Startup Heist
```rust
use numbat::{Context, resolver::FileSystemModuleImporter};

pub struct CrunchieEngine {
    // This is the "Holy Grail" of physical units
    unit_registry: numbat::unit_registry::UnitRegistry,
    // We might also need to keep track of known constants/functions
    // extracted from the Numbat context.
}

impl CrunchieEngine {
    pub fn bootstrap() -> Self {
        let mut ctx = Context::new(FileSystemModuleImporter::default());
        // Load the standard library...
        ctx.interpret("use units::si", CodeSource::Internal).unwrap();
        
        // Reach into the guts of the context to steal the registry
        // Note: We'll need to check Numbat's visibility; might need a wrapper
        // or to use the public API to list all registered units.
        let unit_registry = ctx.unit_registry().clone();
        
        Self { unit_registry }
    }
}
```

---

## 3. The Mapping: ID -> Physics
In the "Brainless Sweep", `2m` becomes:
- `ID 2`: (A numeric literal)
- `ID 501`: (The interning of the string "m")

### Snippet: Resolving a Quantity
```rust
// Inside the Engine's solve loop
fn resolve_term(&self, val_id: ID, unit_id: Option<ID>) -> numbat::Quantity {
    let scalar = self.lookup_number(val_id); // e.g., 2.0
    
    let unit = if let Some(uid) = unit_id {
        let symbol = self.lookup_symbol(uid); // "m"
        // Ask Numbat to create a unit from this string
        self.unit_registry.get_unit(symbol).unwrap_or(Unit::scalar())
    } else {
        Unit::scalar()
    };

    numbat::Quantity::new(scalar, unit)
}
```

---

## 4. Operational Math
Numbat implements `Add`, `Sub`, `Mul`, `Div` for `&Quantity`.

### Snippet: The "Brainless" Addition
```rust
fn handle_add(&self, left_id: ID, right_id: ID) -> EngineResult {
    let q1 = self.get_quantity(left_id);
    let q2 = self.get_quantity(right_id);

    // Numbat does the heavy lifting:
    // - Converts 2m + 50cm to 2.5m
    // - Throws IncompatibleUnits if 2m + 50s
    match (&q1 + &q2) {
        Ok(result) => {
            // result is a numbat::Quantity
            self.store_result(result)
        }
        Err(e) => {
            // Poison the scope!
            self.mark_error(Span::between(left_id, right_id), e.to_string())
        }
    }
}
```

---

## 5. Advanced Features: "The Future"
Because we have the Numbat Logic, we get these "for free":

### A. Implicit Conversion (The "As" Verb)
In Crunchie: `100km / 2h -> mph`
The engine sees the `->` operator, takes the Numbat result of the left side, and calls `.convert_to()` using the right side's unit.

### B. Full Simplification
Numbat can take a messy unit like `kg·m/s²` and tell you it's a `N` (Newton).
```rust
let simplified = result.full_simplify_with_registry(&self.unit_registry, |name| {
    self.unit_registry.get_unit(name)
});
```

### C. Currency (The "Work" Reality)
Numbat supports currency if an exchange rate provider is hooked up.
`50 USD + 20 EUR` -> Numbat handles the fetch and the math.

---

## 6. Implementation TODOs
- [ ] **Dependency:** Add `numbat = "1.13.0"` to `Cargo.toml`.
- [ ] **Visibility:** Check if `numbat::Context` provides enough public access to `UnitRegistry` or if we need to pull units via `ctx.variable_names()`.
- [ ] **Symbol Extraction:** On bootstrap, iterate all unit symbols in Numbat and pre-populate Crunchie's `SymbolMap`. This ensures `m` always gets a low, stable ID.
- [ ] **Error Mapping:** Map `QuantityError` to Crunchie's internal `EngineError` (Poisoning).

---

## 7. Inspiration: Why this wins
We are not building a parser for `2m + 50cm`. 
The parser already turned that into `[Term(2, "m"), Op(+), Term(50, "cm")]`.
The Engine is just a **translator** that speaks Numbat to get the answer. 

**This is the "Port and Contract" model:**
- **Crunchie Port:** Raw Text <-> Flat ID Map
- **Numbat Contract:** Physical Math Solver
- **Engine:** The glue.
