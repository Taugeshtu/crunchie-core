use super::*;
use std::collections::HashMap;

/// Helper to turn the flat topology back into nested vectors for easy assertions,
/// matching the logic from the Python prototype's `reconstruct` function.
fn reconstruct(result: &ParserResult) -> Vec<serde_json::Value> {
    // Create a reverse lookup for symbols
    let mut reverse_symbols = HashMap::new();
    for (k, v) in &result.symbols {
        reverse_symbols.insert(*v, k.clone());
    }

    fn resolve_container(cid: i32, result: &ParserResult, reverse_symbols: &HashMap<i32, String>) -> Vec<serde_json::Value> {
        let mut res = Vec::new();
        if let Some(container) = result.containers.get(&cid) {
            for unit in &container.contents {
                if result.containers.contains_key(&unit.id) {
                    res.push(serde_json::Value::Array(resolve_container(unit.id, result, reverse_symbols)));
                } else {
                    let sym = reverse_symbols.get(&unit.id).cloned().unwrap_or_else(|| format!("?{}?", unit.id));
                    res.push(serde_json::Value::String(sym));
                }
            }
        }
        res
    }

    // Root is always ID 0
    resolve_container(0, result, &reverse_symbols)
}

#[test]
fn test_parser_cases() {
    let cases = [
        ("5", r#"[["5"]]"#),
        ("x = 5", r#"[["x", "=", "5"]]"#),
        ("3 + (1 + 2)", r#"[["3", "+", ["1", "+", "2"]]]"#),
        ("x = 5 # comment", r#"[["x", "=", "5"]]"#),
        ("x=1; y=2", r#"[["x", "=", "1"], ["y", "=", "2"]]"#),
        ("z = (3, 5\n 7)", r#"[["z", "=", ["3", ",", "5", "\n", "7"]]]"#),
        ("z = (3; 5)", r#"[["z", "=", ["3", ";", "5"]]]"#),
        ("-5", r#"[["-", "5"]]"#),
    ];

    let builtins = builtins::generate_symbol_map();

    for (input, expected_json) in cases {
        let result = parse(input, &builtins, std::iter::empty::<&str>());
        let reconstructed = serde_json::Value::Array(reconstruct(&result));
        
        let expected_value: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        
        assert_eq!(reconstructed, expected_value, "Failed on input: {:?}", input);
    }
}

#[test]
fn test_unclosed_container() {
    let builtins = builtins::generate_symbol_map();
    let result = parse("x = (5", &builtins, std::iter::empty::<&str>());
    
    // Check that the nested container (ID 3) is marked invalid
    let container = result.containers.get(&3).unwrap();
    assert!(!container.valid);
    assert!(result.diagnostics.iter().any(|d| matches!(d.code, model::DiagnosticCode::UnclosedContainer)));
}
