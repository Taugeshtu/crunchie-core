# Overview

The Crunchie Engine is a multi-stage pipeline that takes a text buffer through a structural "brainless" parse, into semantic refinement, into structural normalization, and finally into physically-aware computation. It follows a linear, data-oriented flow designed for performance and robust error propagation (Poisoning).

# Pipeline

## [[Stage0_Parser]]
The Parser performs the "Brainless Sweep," turning raw text into a flat, ID-addressed map of **Atoms** and **Containers**. It identifies structural boundaries (parentheses and comments) but ignores mathematical meaning.

## [[Stage1_Distiller]]
The Distiller is the "Semantic Bridge." It refines the meanings of IDs by munching raw strings into typed Atoms (Values, Variables, etc.). Because it runs before the Janitor, any new containers created by Distiller expansions are normalized by the Janitor.

## [[Stage2_Janitor]]
The Janitor scrubs the topology to ensure it is "math-ready." It performs **Boundary Splitting** (to establish lines) and **Recursive Scrubbing** (to remove redundant nesting and normalize sequence markers).

## [[Stage3_Unroller]]
The Unroller flattens the nested hierarchy into a linear "Tape" of instructions. It respects mathematical precedence and allocates virtual registers for the final execution.

## [[Stage4_Executioner]]
The Executioner is the final pass that talks to the `fend-core` arithmetic engine. It performs the solve loop, handles poisoning, and generates result fills. Unlike Numbat, Fend maintains its own internal unit and variable state via a `Context`.
