# Engine Stage: Janitor

The Janitor is the first pass of the engine. Its job is to scrub the "Topological Soup" provided by the parser, ensuring it is mathematically sane and free of structural noise, acting as both an optimizer and a structural linter.

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

### 2. Depth-First Traversal & The "Rebuilding Vec"
The Janitor processes containers recursively, starting with the children of Root (the Lines). For a given raw container, it creates a temporary `children_rebuilding` vector.

It iterates through the raw container's units:
*   **If the unit is a Symbol/Operator**: Push it directly into the `children_rebuilding` vector.
*   **If the unit points to a nested Container**: Pause and recurse into that container. The recursion will return an `Option<Unit>`. If `Some(Unit)` is returned, push it into the `children_rebuilding` vector. If `None` is returned, ignore it.

### 3. Sequence Normalization (In-Flight or Post-Vec)
The parser treats nested `,`, `;`, and `\n` as distinct symbol units. While scanning the contents (or immediately after the temporary vec is populated), the Janitor cleans these up and coerces them into a standard `,` internal representation.
It also cleans things up:
*   **Stuttering**: If multiple sequence operators appear consecutively (e.g., `5,,6` or `5,\n6`), keep only one `,`. If the stutter involves an actual redundant comma, append a `StraySequence` diagnostic.
*   **Leading**: If the vec starts with a sequence operator (e.g., `(,5)` or `(\n5)`), trim it. If it was an explicit comma or a semicolon, append a `StraySequence` diagnostic. If it was just a newline, trim it silently.
*   **Trailing**: If the vec ends with a sequence operator (e.g., `(5,)` or `(5\n)`), trim it. If it was an explicit comma or a semicolon, append a `StraySequence` diagnostic. If it was a newline, trim it silently.

### 4. Promotion, Flattening, or Disposal
Once the `children_rebuilding` vector is fully populated and normalized, the Janitor decides its fate:

*   **Empty Line Disposal**: If this is a Line container (a direct child of Root) and the vec is completely empty, return `None`. It is discarded entirely.
*   **Inert Container Flattening**: If this is a nested container (not Root, not Line) and the vec contains *exactly one* Unit (e.g., `(5)` or `(x)`), it is "Inert." Do not create a container. Simply return `Some(Unit)` containing that single item's ID and original offset to the parent.
*   **Promotion (Valid Container)**: If the container has multiple items, or if it is an intentionally empty nested container (e.g., `()`, representing an empty statement), it is valid.
    *   Allocate a new container ID using `next_id`.
    *   Insert a new `Container` into the map using the `children_rebuilding` vec, preserving the `valid` (poison) state from the raw container.
    *   Return `Some(Unit { id: new_id, offset: original_offset })` to the parent.

### 5. Final Assembly
The Root container's own `children_rebuilding` vector is inserted into the map at ID `0`, completing the cleaned `ParserResult`.
