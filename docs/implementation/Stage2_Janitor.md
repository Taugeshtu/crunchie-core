# Engine Stage: Janitor

The Janitor is the first transforming pass of the engine. Its job is to scrub the "Topological Soup" provided by the parser, acting as an optimizer that simplifies the graph before semantic analysis.

## Contract

*   **Input / Output**: `&mut Workspace` (mutated in-place. Mutation: containers structure).

## The Transformations

The Janitor performs two primary transformations to normalize the topology:

1.  **Boundary Splitting (Root Only)**: Isolates independent mathematical statements (Lines) by splitting the Root's flat contents at every `\n` or `;`.
2.  **Recursive Scrubbing (All Containers)**: Simplifies and normalizes every container in the workspace using an **Unwrap-then-Normalize** pattern.

## The Pass Algorithm

The Janitor executes these transformations in a **single recursive traversal** of the workspace graph:

### 1. Root Entrance
The Janitor begins at the Root (ID `0`). It scans the flat list of entities and splits them into segments based on `\n` or `;` boundaries. 

### 2. Recursive Scrubbing
For each segment (promoted to a Line) and every container encountered during the descent, the Janitor applies the following logic:

#### Step A: Redundant Nesting Collapse ("Unwrapping")
Before processing children, the Janitor looks for "Container-only" nesting. 
*   **The Rule**: If a container only contains **a single other container**, replace our contents with the contents of that inner container and **repeat Step A**. 
*   **Result**: `(((5 + 1)))` collapses level-by-level until it becomes `(5 + 1)`. 
*   **Poisoning**: The `corrupted` flag is OR'd upward during each collapse to preserve error provenance.

#### Step B: Deep Normalization
Once the container is unwrapped, the Janitor performs a final normalization of the surviving contents:
*   **Recurse**: If an entity is a container, immediately call the **Recursive Scrubbing** logic on it.
*   **Coerce**: Inside the container, any remaining `\n` or `;` atoms are coerced into `,` (Comma) atoms.
*   **Trim**: Leading, trailing, or consecutive commas are removed (emitting `StraySequence` diagnostics).

## Example Execution
**Input**: `x = 5; (((y = (x + 3) * max(6; 7)(4))))\ny =`

1.  **Root Pass**: Identifies boundaries. Promotes three lines: `[x = 5]`, `[(((y = ...)))]`, and `[y =]`.
2.  **Descent (Line 1)**: Normalizes to `x = 5`.
3.  **Descent (Line 2)**: 
    *   **Unwrap**: Immediately collapses `(((...)))` into `(y = (x + 3) * max(6; 7)(4))`.
    *   **Normalize**: Recurses into `(x + 3)` (identity), `max(6; 7)` (coerces `;`), and `(4)` (identity).
4.  **Descent (Line 3)**: Normalizes to `y =`.

## Final State
The Workspace is now a lean, line-addressed graph. Redundant nesting is gone, all lines are normalized, and the Distiller can now proceed with semantic analysis on a "clean" topology.
