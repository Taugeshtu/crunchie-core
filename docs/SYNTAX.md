# Crunchie Grammar Guide

This document outlines how to write math in Crunchie. Crunchie is designed to be a "Conversation with a Buffer," meaning the syntax is minimal and focused on readability.

## 1. Numbers and Units
Crunchie understands physical units natively. You can write them together or separately:
*   `10m + 50 cm`
*   `5 kg * 2`
*   `0xFF` (Hexadecimal)
*   `0b0010_1000` (Binary, and also underscore for visual separation)
*   `1e-5` (Scientific notation)
*   `3 cm^2 + 1 cm2` (shorthand "unit raised to power" supports powers of 2, 3, 4, 5)
*   `3k + 5K + 1M` (scale suffixes, only `k`/`K` and `M`)

## 2. Variables and Constants
Assign values to names to reuse them later.
*   `radius = 10cm`
*   `area = PI * radius^2`
*   `x = 5; y = 10` (Multiple statements on one line)

## 3. Grouping
Use parentheses to control the order of operations.
*   `5 * (2 + 3)`

## 4. Physical Conversions
Use the `to` operator to convert between units.
*   `100km / 2h to mph`

## 5. Comments
Use `#` or `//` for notes. Everything from the symbol to the end of the line is ignored by the solver.
*   `# This is a comment`
*   `x = 10 // Setting the base value`

## 6. Calculation Modes
The engine interprets your intent based on the trailing operators:
*   `x = 10` -> **Assignment**: Store the value.
*   `10 + 5 = 15` -> **Assertion**: Validate the math (Errors if not true).
*   `10 + 5 = ` -> **Query**: Ask the engine to calculate and fill in the result.

## 7. Error Handling (Poisoning)
If you make a mistake on one line (like an unclosed parenthesis), Crunchie "Poisons" that line and anything that depends on it. However, independent calculations on other lines will continue to work normally.
