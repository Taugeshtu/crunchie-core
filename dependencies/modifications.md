# Fend Core Modifications

This directory contains a vendored and modified version of `fend-core` (version 1.5.8).

## Summary of Changes

1. **Selective Vendoring**: Only the `fend-core` subdirectory was extracted from the original [fend](https://github.com/printfn/fend) repository.
2. **De-encapsulation**: All modules within the crate have been made `pub` to allow `crunchie-core` to inspect and interact with internal structs, types, and logic directly. This was necessary to bypass strict visibility constraints that prevented structural analysis of math expressions.
3. **Built-in Exposition**: Added `get_builtin_functions()` and `get_builtin_constants()` to `src/lib.rs`. These functions expose the mathematical core of Fend (transcendental constants, physical constants, and pure math functions) so that the Crunchie Parser can identify them during the initial "brainless" sweep without needing to duplicate the entire symbol table.

## Rationale
Crunchie requires high structural awareness of expressions before they are evaluated. By vendoring and opening up `fend-core`, we can utilize Fend's robust arbitrary-precision engine while maintaining the ability to normalize, unroll, and manipulate the math topology in our own pipeline.
