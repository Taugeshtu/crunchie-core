# Engine Stage: Distiller

The Distiller acts as the semantic bridge. It is a **typization** stage responsible for refining the raw strings produced by the parser into semantic **Atoms** (identifying what is a value, a variable, etc.).

*Note: In the pipeline, the Distiller runs **before** the Janitor, allowing the Janitor to structurally normalize any new containers created by Distiller expansions.*

## Contract

*   **Input / Output**: `&mut Workspace`.
*   **Mutation**: Refines `atoms` in the global map and inserts new expansion containers into `containers`.

## Algorithm: Refine-then-Scan

The Distiller operates primarily on the `atoms` map to leverage the "Refine Once, Update Everywhere" property. It only touches containers to create expansions or to recover error provenance.

### 1. The Munch (Atom Refinement)
The Distiller iterates directly over every `Atom::Raw` entry in `workspace.atoms`.
*   Each string is sent to the **Fend Muncher** (the bridge to `fend-core`).
*   The Muncher uses Fend's lexer to decompose the string into terminal values, units, and identifiers.

### 2. Resolution & Refinement
Based on the Muncher's output, the Distiller updates the `atoms` map:

*   **1:1 Replacement**: If the string resolves to a single type (e.g., `"x"` becomes `Atom::Variable("x")`), the map is updated in-place. Every **Entity** in the workspace pointing to this ID is instantly refined.
*   **1:N Expansion**: If a string explodes into multiple parts (e.g., `5kg` becomes `5` and `kg`):
    1.  The Distiller mints new IDs for the components.
    2.  It creates a new expansion **Container** holding the new IDs.
    3.  It updates the original ID's entry in the `atoms` map to `Atom::Container(new_id)`.
*   **Poisoning**: If munching fails, the atom is marked `Atom::Poison` and its ID is added to a `poisoned_ids` list.

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
