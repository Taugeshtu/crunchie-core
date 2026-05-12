use crate::model::{OpCode, FendOp};
use std::collections::HashMap;

pub const PUNCTUATION_OPERATORS: &[&str] = &[
    "+=", "-=", "*=", "/=", ">=", "<=", "==", "!=", ">>", "<<",
    "+", "-", "*", "/", "=", "^", ",", ";", "\n", ">", "<"
];
pub const ALPHANUMERIC_OPERATORS: &[&str] = &["to"];
pub const FUNCTIONS: &[&str] = &["sin", "cos", "tan", "log", "sqrt"];
pub const ILLEGAL_CHARS: &[char] = &['~', '`', '@', '[', ']', '{', '}', '\\', '|'];

pub const FUNCTIONS_START_ID: i32 = -10_000;
pub const CONSTANTS_START_ID: i32 = -20_000;

pub fn get_operator(s: &str) -> Option<OpCode> {
    match s {
        "+" => Some(OpCode::Fend(FendOp::Add)),
        "-" => Some(OpCode::Fend(FendOp::Sub)),
        "*" => Some(OpCode::Fend(FendOp::Mul)),
        "/" => Some(OpCode::Fend(FendOp::Div)),
        "^" => Some(OpCode::Fend(FendOp::Pow)),
        "=" => Some(OpCode::Fend(FendOp::Equals)),
        "==" => Some(OpCode::Fend(FendOp::DoubleEquals)),
        "!=" => Some(OpCode::Fend(FendOp::NotEquals)),
        ">>" => Some(OpCode::Fend(FendOp::ShiftRight)),
        "<<" => Some(OpCode::Fend(FendOp::ShiftLeft)),
        "to" => Some(OpCode::Fend(FendOp::To)),
        ";" => Some(OpCode::Fend(FendOp::Semicolon)),
        "\n" => Some(OpCode::Sequence),
        "," => Some(OpCode::Comma),
        ">" => Some(OpCode::Greater),
        "<" => Some(OpCode::Less),
        ">=" => Some(OpCode::GreaterEqual),
        "<=" => Some(OpCode::LessEqual),
        "+=" => Some(OpCode::AddAssign),
        "-=" => Some(OpCode::SubAssign),
        "*=" => Some(OpCode::MulAssign),
        "/=" => Some(OpCode::DivAssign),
        _ => None,
    }
}

/// Generates the initial symbol map containing operators and functions.
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
    for &func in fend_core::get_builtin_functions() {
        m.insert(func.to_string(), func_id);
        func_id -= 1;
    }

    let mut const_id = CONSTANTS_START_ID;
    for &c in fend_core::get_builtin_constants() {
        m.insert(c.to_string(), const_id);
        const_id -= 1;
    }

    m
}
