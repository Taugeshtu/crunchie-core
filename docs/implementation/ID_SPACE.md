# Unified Identity Space

The Crunchie Engine uses a unified partitioned ID space. By checking the range of an ID, the engine can quickly classify predefined symbols during the initial parse phase.

| ID Range | Occupant Type | Range Logic |
| :--- | :--- | :--- |
| **`<= -20,000`** | **Constants** | `CONSTANTS_START_ID` and down (e.g., PI, TAU). |
| **`-19,999` to `-10,000`** | **Built-in Functions** | `FUNCTIONS_START_ID` and down (e.g., sin, cos). |
| **`-9,999` to `-1`** | **Built-in Operators** | Starting at `-1` and down (e.g., +, -, =). |
| **`0`** | **Root Container** | The hardcoded parent of all lines. |
| **`1` to `2,147,483,647`** | **User Symbols & Containers** | Sequential `next_id`. Includes variables, numbers, nested groups, and dynamically registered configuration constants. |

## Usage
Because the Parser interns all symbols during the sweep, it uses these ranges to instantly populate the `Workspace` with correctly typed `Atom` enum variants without string comparisons.

*   **Negative IDs**: Fast-pathed to predefined Operator, Function, or Constant logic.
*   **Positive IDs**: Treated as raw User Symbols (needs Munching), structural Containers, or unrolled Instructions.
