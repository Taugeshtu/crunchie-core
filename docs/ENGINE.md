# Overview

The Crunchie Engine is a multi-stage pipeline that transforms a structural "brainless" parse into semantic, physically-aware computation. It follows a linear, data-oriented flow designed for performance and robust error propagation (Poisoning).

The goal is to move from "Topological Soup" to a "Physical Solve" with minimal overhead and maximum clarity.

# Pipeline

## Janitor
The Janitor scrubs the raw structural soup to ensure it is "math-ready." It handles the removal of empty containers and the normalization of sequence markers.

[[Engine-Janitor]]

## Distiller
The Distiller is the "Air-Gap" between the world of raw IDs and the world of Physical Math. It assigns roles to symbols and marries values to their units (Coupling).

[[Engine-Distiller]]

## Unroller
The Unroller flattens the nested hierarchy into a linear "Tape" of instructions. It respects mathematical precedence and allocates virtual registers.

[[Engine-Unroller]]

## Executioner
The Executioner is the final pass that talks to the Numbat physics engine. It performs the solve loop, handles poisoning, and generates fill results.

[[Engine-Executioner]]
