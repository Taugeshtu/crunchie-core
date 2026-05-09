# Engine Stage: Janitor

The Janitor is the first pass of the engine. Its job is to scrub the "Topological Soup" provided by the parser and ensure it is mathematically sane and free of structural noise.

## Contract

*   **Input**: `ParserResult` (Raw)
*   **Output**: `ParserResult` (Cleaned)

## Innards

*   **Container Scrubbing**: Iterates through the flat container map and identifies "Inert" containers.
*   **Empty Disposal**: Discards any container that has no content.
*   **Sequence Normalization**: 
    *   Finds containers that only contain sequence operators (redundant commas, semicolons).
    *   Normalizes mixed sequence markers into a canonical internal representation.
*   **Topology Validation**: Performs a quick check for immediate structural failures (e.g. operators with no possible operands) before passing the soup to the Distiller.
