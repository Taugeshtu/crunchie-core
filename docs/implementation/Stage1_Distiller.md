# Engine Stage: Distiller

The Distiller acts as the semantic bridge. It is a **typization** stage responsible for refining the raw strings produced by the parser into semantic **Atoms** (identifying what is a value, a variable, etc.).

*Note: In the pipeline, the Distiller runs **before** the Janitor, allowing the Janitor to structurally normalize any new containers created by Distiller expansions.*

## Contract

*   **Input / Output**: `&mut Workspace`.
*   **Mutation**: Refines `atoms` in the global map and inserts new expansion containers into `containers`.

## Algorithm: Two-Pass Refinement

To avoid race conditions where a compound monolith (e.g., `5x`) is processed before its components (e.g., `x = 10`) are identified as variables, the Distiller uses a two-pass approach.

### 1. Pass 1: Terminal Refinement
The Distiller iterates over all `Atom::Raw` entries and attempts to resolve them.
*   If a string resolves to a **single part** (e.g., `"x"` -> `Atom::Variable("x")`), it is updated in the `atoms` map immediately.
*   This pass "seeds" the known identifiers set, ensuring that standalone variables and units are recognized before the next pass.

### 2. Pass 2: Expansion & Poisoning
The Distiller processes the remaining atoms using the full set of identifiers collected in Pass 1.
*   **1:1 Replacement**: Finalizes any single-part resolutions that weren't caught in Pass 1.
*   **1:N Expansion**: If a string explodes into multiple parts (e.g., `5kg` becomes `5` and `kg`):
    1.  The Distiller mints new IDs for the components.
    2.  It creates a new expansion **Container** holding the new IDs.
    3.  It updates the original ID's entry in the `atoms` map to `Atom::Container(new_id)`.
*   **Poisoning**: If munching still fails for an atom, it is marked `Atom::Poison` and its ID is added to a `poisoned_ids` list.

### 3. Provenance Recovery (The Error Scan)
To keep the "happy path" fast, the Distiller only scans containers if `poisoned_ids` is not empty.
*   It performs a flat, linear scan of all vectors in `workspace.containers.values()`.
*   Whenever it encounters an **Entity** whose ID is in the `poisoned_ids` set, it uses that entity's `offset` to emit a `MalformedSymbol` diagnostic.
*   *Note: Because we scan all containers, we report every occurrence of a poisoned atom, even if it appears in a corrupted or ghost container.*

## Why This Order Wins
By running the Distiller before the Janitor, we ensure that:
1.  **Refinement is Total**: Every raw string in the initial parser soup is typed.
2.  **Expansion Normalization**: Any `Atom::Container` created by a split (like `5kg`) is passed to the Janitor, which can then perform sequence coercion or de-stacking on the new structure if needed.
3.  **Orthogonality**: Distiller worries about *Meaning* (Atoms); Janitor worries about *Structure* (Topology).
