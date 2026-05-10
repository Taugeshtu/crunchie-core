# Engine Stage: Janitor

The Janitor is the first transforming pass of the engine. Its job is to scrub the "Topological Soup" provided by the parser, ensuring it is mathematically sane and free of structural noise, acting as both an optimizer and a structural linter.

## Contract

*   **Input / Output**: `&mut Workspace` (mutated in-place).

## Algorithm

The Janitor modifies the `Workspace` graph via a Depth-First Traversal (DFT) of the raw topology starting at the root container (ID `0`). Instead of creating containers and deleting them later, the Janitor builds children `Entity` items into temporary vectors and only "commits" them to the `workspace.containers` map if they prove to be structurally significant.

### 1. Resolution
Because the topological representation is flat, the Janitor must resolve each `Entity`'s `id` against `workspace.symbols` to determine its behavior (e.g., whether it is a sequence operator like `\n` or `,`, or a nested `ContainerRef`).

### 2. The Root Split (Line Breaking)
The Parser's output for the Root Container (ID `0`) is a completely flat list of `Entity` items representing raw chunks, sequence operators (`\n`, `;`), and nested containers. The Janitor's first task is to break this flat list into isolated Line Containers.
*   Iterate through the Root's `contents`.
*   Accumulate `Entity` objects into a temporary `current_line` vector.
*   When a line-breaking sequence operator (`\n` or `;`) is encountered (resolved via `workspace.symbols`):
    *   If `current_line` is empty, ignore it (this naturally cleans up leading newlines or stutters).
    *   If `current_line` is not empty, process the `current_line` (see Step 3). Then, use `workspace.next_id` to allocate a new Line container ID, add it to `workspace.containers`, and append a new `ContainerRef` `Entity` to the rebuilt root contents. Clear `current_line`.
*   When the end of the Root is reached, process and promote any remaining items in `current_line`.

### 3. Depth-First Traversal & Normalization
For each `Entity` within a line (or within a nested container), the Janitor recurses. 
*   **Symbols/Operators**: Checked for stuttering. Consecutive sequence operators (like commas) are reduced.
*   **Nested Containers**: When a `ContainerRef` is resolved, the Janitor recurses into that container in `workspace.containers`, building a new `children_rebuilding` vector.
    *   Inside nested containers, `\n` and `;` do *not* break lines; instead, they are coerced into canonical `,` sequence operators.
    *   **Stuttering**: If multiple sequence operators appear consecutively (e.g., `5,,6` or `5,\n6`), keep only one `,`. If the stutter involves an actual redundant comma, push a `StraySequence` to `workspace.diagnostics`.
    *   **Leading/Trailing**: Trim leading/trailing sequences. Push `StraySequence` if they were explicit `,` or `;`.

### 4. Promotion, Flattening, or Disposal
Once a nested container's `children_rebuilding` vector is populated:
*   **Inert Container Flattening**: If the vec contains *exactly one* `Entity` (e.g., `(5)` or `(x)`), it is "Inert." Do not promote the container. Simply return the inner `Entity` directly to the parent, completely bypassing the container structure.
*   **Promotion (Healthy Container)**: If the container has multiple items, or if it is an intentionally empty statement (e.g., `()`), keep the container mapping in `workspace.containers` (updating its contents to the rebuilt vector), and return the `Entity` (the `ContainerRef`) to the parent.

### 5. Final Assembly
The Root container's (ID `0`) `contents` are overwritten with the newly built list of Line container `Entity` references. The mutated `Workspace` is now clean and ready for the Distiller.