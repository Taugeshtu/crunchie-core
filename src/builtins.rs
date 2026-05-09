use std::collections::HashMap;

pub const OPERATORS: &[char] = &['+', '-', '*', '/', '=', '^', ','];
pub const FUNCTIONS: &[&str] = &["sin", "cos", "tan", "log", "sqrt"];
pub const ILLEGAL_CHARS: &[char] = &['~', '`', '@', '[', ']', '{', '}', '\\', '|'];

pub const FUNCTIONS_START_ID: i32 = -1_000_000;
pub const CONSTANTS_START_ID: i32 = 1_000_000;

/// Generates the initial symbol map containing operators and functions.
/// Operators get IDs -1 and down.
/// Functions get IDs FUNCTIONS_START_ID and down.
pub fn generate_symbol_map() -> HashMap<String, i32> {
    let mut m = HashMap::new();
    
    let mut op_id = -1;
    for &op in OPERATORS {
        m.insert(op.to_string(), op_id);
        op_id -= 1;
    }

    let mut func_id = FUNCTIONS_START_ID;
    for &func in FUNCTIONS {
        m.insert(func.to_string(), func_id);
        func_id -= 1;
    }

    m
}
