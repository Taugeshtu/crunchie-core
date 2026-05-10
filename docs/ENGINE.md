# Overview

The Crunchie Engine is a multi-stage pipeline that takes a text buffer through a structural "brainless" parse, into semantic markup, into physically-aware computation. It follows a linear, data-oriented flow designed for performance and robust error propagation (Poisoning).

# Pipeline

## [[Engine-Parser]]
The Parser performs the "Brainless Sweep," turning raw text into a flat, ID-addressed map of symbols and containers.

## [[Engine-Janitor]]
The Janitor scrubs the raw structural soup to ensure it is "math-ready." It handles the removal of empty containers and the normalization of sequence markers.

## [[Engine-Distiller]]
The Distiller is the "Air-Gap" between the world of raw IDs and the world of Physical Math. It assigns roles to symbols and marries values to their units (Coupling).

## [[Engine-Unroller]]
The Unroller flattens the nested hierarchy into a linear "Tape" of instructions. It respects mathematical precedence and allocates virtual registers.

## [[Engine-Executioner]]
The Executioner is the final pass that talks to the Numbat physics engine. It performs the solve loop, handles poisoning, and generates fill results.
