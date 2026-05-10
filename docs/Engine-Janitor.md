# Engine Stage: Janitor

The Janitor is the first transforming pass of the engine. Its job is to scrub the "Topological Soup" provided by the parser, ensuring it is mathematically sane and free of structural noise, acting as both an optimizer and a structural linter.

## Contract

*   **Input**: `ParserResult` (Raw)
*   **Output**: `ParserResult` (Cleaned)

## Algorithm

The Janitor builds a cleaned `ParserResult` via a Depth-First Traversal (DFT) of the raw topology. Instead of creating containers and deleting them later, the Janitor builds children into temporary lists and only "commits" them to the new container map if they prove to be structurally significant.

### 1. Initialization
*   Create a new `ParserResult`.
*   Copy the `symbols`, `comments`, and `diagnostics` exactly as they are from the Raw Result.
*   Determine the "High-Water Mark" ID (the highest integer ID currently used by any symbol or container in the raw result). Initialize a `next_id` counter from this mark `+ 1` to ensure new containers never collide with existing Unified Identity Space IDs.
*   Pre-allocate the Root container at ID `0`.

### 2. The Root Split (Line Breaking)
The Parser's output for the Root Container (ID `0`) is a completely flat list of tokens, sequence operators (`\n`, `;`), and nested containers. The Janitor's first task is to break this flat list into isolated Line Containers.
*   Iterate through the Root's units.
*   Accumulate units into a temporary `current_line` vector.
*   When a `\n` or `;` unit is encountered:
    *   If `current_line` is empty, ignore it (this naturally cleans up leading newlines or stutters).
    *   If `current_line` is not empty, process the `current_line` (see Step 3), allocate a new Line container ID, and append it to the new Root's contents. Clear `current_line`.
*   When the end of the Root is reached, process and promote any remaining units in `current_line`.

### 3. Depth-First Traversal & Normalization
For each unit within a line (or within a nested container), the Janitor recurses. 
*   **Symbols/Operators**: Checked for stuttering. Consecutive commas are reduced.
*   **Nested Containers**: The Janitor recurses into the container, building its `children_rebuilding` vector.
    *   Inside nested containers, `\n` and `;` do *not* break lines; instead, they are coerced into canonical `,` sequence operators.
    *   **Stuttering**: If multiple sequence operators appear consecutively (e.g., `5,,6` or `5,\n6`), keep only one `,`. If the stutter involves an actual redundant comma, append a `StraySequence` diagnostic.
    *   **Leading/Trailing**: Trim leading/trailing sequences. Append `StraySequence` if they were explicit `,` or `;`.

### 4. Promotion, Flattening, or Disposal
Once a nested container's `children_rebuilding` vector is populated:
*   **Inert Container Flattening**: If the vec contains *exactly one* Unit (e.g., `(5)` or `(x)`), it is "Inert." Do not create a container. Simply return `Some(Unit)` containing that single item's ID and original offset to the parent.
*   **Promotion (Healthy Container)**: If the container has multiple items, or if it is an intentionally empty statement (e.g., `()`), allocate a new container ID, insert it into the map (preserving poison state), and return the new Unit to the parent.

### 5. Final Assembly
The new Root container `0` is fully populated with Line containers. The cleaned `ParserResult` is returned.
