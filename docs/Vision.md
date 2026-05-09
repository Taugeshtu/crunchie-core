# Crunchie Vision

> "Math is a conversation with a buffer, not a series of buttons on a virtual Casio."

Crunchie is a number-crunching engine built for people who live in text editors and terminal emulators. It rejects the "Calculator App" metaphor in favor of a **Work Management** approach to calculation.

## Core Tenets

### 1. We have skeuomorphism at home
It's called the fucking numpad. Calculator for a machine with a physical keyboard needs only a text box.

### 2. It's never just "PI times five"
You'll need more than one step of calculation. Text box must be multiline. Calculations live in a buffer; Crunchie lints, validates, and auto-completes. Whether it's an LSP in VSCode or a scratchpad in the terminal, the buffer is the source of truth.

### 3. Spreadsheets are just memberwise math on vectors
Spreadsheets are cool; for daily driving they are overkill, but the idea of calculating over an array is great. Let's support that by doing memberwise math on vectors

### 4. Topology over Semantics
The parser is intentionally "brainless." It understands *where* things are (Containment and Separation) but not *what* they mean. This allows the Engine to be flexible, context-aware, and incredibly fast.

### 5. Strict but Isolated
One error shouldn't kill the whole session. Malformed containers "poison" their dependents but leave independent math untouched. We value robust, incremental progress over fragile, all-or-nothing evaluation.

### 6. Pure & Portable
Crunchie-core is a pure library. It doesn't know about the disk, the network, or the OS. It only knows about the buffer you give it and the variables you provide. This makes it embeddable anywhere—from a compositor tray to a web-based notebook.

## The Goal
To make calculation as friction-less as typing a sentence. No "Eval" button, no "Clear" command—just math that happens as you think & type.
