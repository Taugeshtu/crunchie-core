# Aspiration: Vectors (Version 2)

Vectors and memberwise mathematical operations are a planned "Version 2" feature. The goal is to provide lightweight spreadsheet-like capabilities directly within the buffer.

## Syntax Sketch

```
x = (3, 5, 7)
y = x * 2
y = // this will auto-fill
sum(y) = // this can special-case maybe?..
```

## Design Considerations & Open Questions

*   **Detection Phase:** How are vectors identified in the pipeline? Are they detected right after the Distiller? (e.g., if the output of a distiller is vector-shaped, it's considered a vector).
*   **Variable Resolution:** Do we allow variables inside vectors? 
*   **Nested Vectors / Recursion:** How do we know that a variable inside a vector is not *another* vector? We surely don't want to allow vectors inside vectors—that leads to multi-dimensional tensor madness and is a recipe for accidentally growing a Python clone! If the user needs that complexity, it would be more ergonomic for them to just use Python. The engine should likely enforce strict 1D limits.
