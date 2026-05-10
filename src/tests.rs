use super::*;
use std::collections::HashMap;

/// Helper to turn the flat topology back into nested vectors for easy assertions,
/// matching the logic from the Python prototype's `reconstruct` function.
fn reconstruct(workspace: &Workspace) -> Vec<serde_json::Value> {
    // Create a reverse lookup for symbols using intern_map and fallback to symbols map
    let mut reverse_intern = HashMap::new();
    for (k, &v) in &workspace.intern_map {
        reverse_intern.insert(v, k.clone());
    }

    fn resolve_container(cid: i32, workspace: &Workspace, reverse_intern: &HashMap<i32, String>) -> Vec<serde_json::Value> {
        let mut res = Vec::new();
        if let Some(container) = workspace.containers.get(&cid) {
            for entity in &container.contents {
                if let Some(model::Symbol::ContainerRef(child_cid)) = workspace.symbols.get(&entity.id) {
                    res.push(serde_json::Value::Array(resolve_container(*child_cid, workspace, reverse_intern)));
                } else {
                    let sym_str = reverse_intern.get(&entity.id).cloned().unwrap_or_else(|| {
                        if let Some(model::Symbol::Raw(s)) = workspace.symbols.get(&entity.id) {
                            s.clone()
                        } else {
                            format!("?{}?", entity.id)
                        }
                    });
                    res.push(serde_json::Value::String(sym_str));
                }
            }
        }
        res
    }

    // Root is always ID 0
    resolve_container(0, workspace, &reverse_intern)
}

#[test]
fn test_parser_cases() {
    let cases = [
        ("5", r#"["5"]"#),
        ("x = 5", r#"["x", "=", "5"]"#),
        ("3 + (1 + 2)", r#"["3", "+", ["1", "+", "2"]]"#),
        ("x = 5 # comment", r#"["x", "=", "5"]"#),
        ("x=1; y=2", r#"["x", "=", "1", ";", "y", "=", "2"]"#),
        ("z = (3, 5\n 7)", r#"["z", "=", ["3", ",", "5", "\n", "7"]]"#),
        ("z = (3; 5)", r#"["z", "=", ["3", ";", "5"]]"#),
        ("-5", r#"["-", "5"]"#),
        ("x += 5", r#"["x", "+=", "5"]"#),
        ("10 to cm", r#"["10", "to", "cm"]"#),
        ("1e-5", r#"["1e-5"]"#),
        ("1.2e+10", r#"["1.2e+10"]"#),
        ("1e-5kg", r#"["1e-5kg"]"#),
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
    
    // Find the dynamically created nested container (which will be the only one besides Root ID 0)
    let (_, container) = result.containers.iter().find(|&(&id, _)| id != 0).unwrap();
    assert!(container.corrupted);
    assert!(result.diagnostics.iter().any(|d| matches!(d.code, model::DiagnosticCode::UnclosedContainer)));
}
