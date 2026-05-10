# Unified Identity Space

The Crunchie Engine uses a unified partitioned ID space to allow for high-performance symbol categorization. By checking the range of an ID, the engine can immediately determine its "quadrant" without performing string comparisons.

| ID Range | Occupant Type | Range Logic |
| :--- | :--- | :--- |
| **`<= -1,000,000`** | **Built-in Functions** | `FUNCTIONS_START_ID` and down (e.g., sin, cos) |
| **`-999,999` to `-1`** | **Built-in Operators** | Starting at `-1` and down (e.g., +, -, =) |
| **`0`** | **Root Container** | The hardcoded parent of all lines. |
| **`1` to `999,999`** | **User Symbols & Containers** | Sequential `next_id`. Includes variables, numbers, and nested groups. |
| **`>= 1,000,000`** | **Constants** | `CONSTANTS_START_ID` and up (e.g., PI, TAU). |

## Usage
Because the Parser interns all symbols during the sweep, the **Distiller** and **Unroller** can use these ranges for fast branching.

*   **Positive IDs < 1M**: May be a Container or a raw User Symbol (needs Munching).
*   **High Positive IDs**: Treated as a constant value.
*   **Negative IDs**: Fast-pathed to Operator or Function logic.
