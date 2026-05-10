use std::collections::HashMap;

pub const PUNCTUATION_OPERATORS: &[&str] = &[
    "+=", "-=", ">=", "<=", "==", "!=", ">>", "<<",
    "+", "-", "*", "/", "=", "^", ",", ";", "\n"
];
pub const ALPHANUMERIC_OPERATORS: &[&str] = &["to"];
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
    for &op in PUNCTUATION_OPERATORS {
        m.insert(op.to_string(), op_id);
        op_id -= 1;
    }

    for &kw in ALPHANUMERIC_OPERATORS {
        m.insert(kw.to_string(), op_id);
        op_id -= 1;
    }

    let mut func_id = FUNCTIONS_START_ID;
    for &func in FUNCTIONS {
        m.insert(func.to_string(), func_id);
        func_id -= 1;
    }

    m
}
