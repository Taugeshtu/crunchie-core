use super::*;
use std::collections::HashMap;

/// Helper to turn the flat topology back into nested vectors for easy assertions,
/// matching the logic from the Python prototype's `reconstruct` function.
fn resolve_symbol(id: i32, workspace: &Workspace, reverse_intern: &HashMap<i32, String>) -> serde_json::Value {
    match workspace.symbols.get(&id) {
        Some(model::Symbol::ContainerRef(child_cid)) => {
            serde_json::Value::Array(resolve_container(*child_cid, workspace, reverse_intern))
        }
        Some(model::Symbol::Quantity(q)) => serde_json::Value::String(format!("Q:{}", q)),
        Some(model::Symbol::Variable(v)) => serde_json::Value::String(format!("V:{}", v)),
        Some(model::Symbol::PhysUnit(u)) => serde_json::Value::String(format!("U:{}", u)),
        Some(model::Symbol::Operator(op)) => serde_json::Value::String(format!("O:{:?}", op)),
        Some(model::Symbol::Constant(c)) => serde_json::Value::String(format!("C:{}", c)),
        Some(model::Symbol::Function(f)) => serde_json::Value::String(format!("F:{}", f)),
        Some(model::Symbol::Instruction { op, args }) => {
            let arg_strs: Vec<String> = args.iter().map(|&aid| {
                match resolve_symbol(aid, workspace, reverse_intern) {
                    serde_json::Value::String(s) => s,
                    _ => "?".to_string()
                }
            }).collect();
            serde_json::Value::String(format!("I:{:?}({})", op, arg_strs.join(", ")))
        }
        Some(model::Symbol::Poison) => serde_json::Value::String("POISON".to_string()),
        Some(model::Symbol::Raw(s)) => serde_json::Value::String(format!("R:{}", s)),
        _ => serde_json::Value::String(reverse_intern.get(&id).cloned().unwrap_or_else(|| {
            format!("?{}?", id)
        })),
    }
}

fn resolve_container(cid: i32, workspace: &Workspace, reverse_intern: &HashMap<i32, String>) -> Vec<serde_json::Value> {
    let mut res = Vec::new();
    if let Some(container) = workspace.containers.get(&cid) {
        for entity in &container.contents {
            res.push(resolve_symbol(entity.id, workspace, reverse_intern));
        }
    }
    res
}

fn reconstruct(workspace: &Workspace) -> Vec<serde_json::Value> {
    let mut reverse_intern = HashMap::new();
    for (k, &v) in &workspace.intern_map {
        reverse_intern.insert(v, k.clone());
    }
    resolve_container(0, workspace, &reverse_intern)
}

#[test]
fn test_distiller_basics() {
    let cases = [
        ("5", r#"[["Q:5"]]"#),
        ("x = 5kg", r#"[["V:x", "O:Assign", "Q:5", "U:kg"]]"#),
        ("10cm3", r#"[["Q:10", "U:cm", "O:Pow", "Q:3"]]"#),
        ("0xFF", r#"[["Q:255"]]"#),
        ("5M", r#"[["Q:5000000"]]"#),
        ("5 + 2", r#"[["Q:5", "O:Add", "Q:2"]]"#),
        ("sin(PI)", r#"[["F:sin", "C:PI"]]"#), // Flattened by Janitor
    ];

    let builtins = builtins::generate_symbol_map();
    let config = config::Config::default();
    let constants = config.constants.keys().map(|s| s.as_str());
    let units = distiller::get_default_units();

    for (input, expected_json) in cases {
        let mut workspace = parse(input, &builtins, constants.clone());
        janitor(&mut workspace);
        distiller(&mut workspace, &units);
        
        let reconstructed = serde_json::Value::Array(reconstruct(&workspace));
        let expected_value: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        
        assert_eq!(reconstructed, expected_value, "Failed on distiller input: {:?}", input);
    }
}

#[test]
fn test_distiller_poison() {
    let builtins = builtins::generate_symbol_map();
    let units = distiller::get_default_units();
    
    let mut workspace = parse("1.2.3", &builtins, std::iter::empty::<&str>());
    distiller(&mut workspace, &units);
    
    assert!(workspace.symbols.values().any(|s| matches!(s, model::Symbol::Poison)));
    assert!(workspace.diagnostics.iter().any(|d| matches!(d.code, model::DiagnosticCode::MalformedSymbol)));
}

#[test]
fn test_distiller_smart_splitting_garbage() {
    let builtins = builtins::generate_symbol_map();
    let units = distiller::get_default_units();
    
    let mut workspace = parse("65kg123", &builtins, std::iter::empty::<&str>());
    distiller(&mut workspace, &units);
    
    assert!(workspace.symbols.values().any(|s| matches!(s, model::Symbol::Poison)));
    assert!(workspace.diagnostics.iter().any(|d| matches!(d.code, model::DiagnosticCode::MalformedSymbol)));
}

#[test]
fn test_parser_raw() {
    let cases = [
        ("5", r#"["R:5"]"#),
        ("x = 5 kg", r#"["R:x", "O:Assign", "R:5", "R:kg"]"#),
        ("3 + (1 + 2)", r#"["R:3", "O:Add", ["R:1", "O:Add", "R:2"]]"#),
    ];

    let builtins = builtins::generate_symbol_map();

    for (input, expected_json) in cases {
        let result = parse(input, &builtins, std::iter::empty::<&str>());
        let reconstructed = serde_json::Value::Array(reconstruct(&result));
        let expected_value: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(reconstructed, expected_value, "Failed on raw parse input: {:?}", input);
    }
}

#[test]
fn test_janitor_cases() {
    let cases = [
        ("5", r#"[["R:5"]]"#),
        ("x = 5 kg", r#"[["R:x", "O:Assign", "R:5", "R:kg"]]"#),
        ("(5)", r#"[["R:5"]]"#),
        ("((5))", r#"[["R:5"]]"#),
        ("3 + (1 + 2)", r#"[["R:3", "O:Add", ["R:1", "O:Add", "R:2"]]]"#),
        ("x = 1; y = 2", r#"[["R:x", "O:Assign", "R:1"], ["R:y", "O:Assign", "R:2"]]"#),
        ("x = 1\ny = 2", r#"[["R:x", "O:Assign", "R:1"], ["R:y", "O:Assign", "R:2"]]"#),
        ("(1; 2)", r#"[["R:1", "O:Sequence", "R:2"]]"#),
        ("(1, \n 2)", r#"[["R:1", "O:Sequence", "R:2"]]"#),
        ("(,1,)", r#"[["R:1"]]"#),
        ("()", r#"[[]]"#), // Empty but healthy should survive?
    ];

    let builtins = builtins::generate_symbol_map();

    for (input, expected_json) in cases {
        let mut workspace = parse(input, &builtins, std::iter::empty::<&str>());
        janitor(&mut workspace);
        let reconstructed = serde_json::Value::Array(reconstruct(&workspace));
        
        let expected_value: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        
        assert_eq!(reconstructed, expected_value, "Failed on janitor input: {:?}", input);
    }
}

#[test]
fn test_unroller_basics() {
    let cases = [
        ("1 + 2 * 3", r#"[["I:Mul(Q:2, Q:3)", "I:Add(Q:1, I:Mul(Q:2, Q:3))"]]"#),
        ("5 cm", r#"[["I:Mul(Q:5, U:cm)"]]"#),
        ("5 PI", r#"[["I:Mul(Q:5, C:PI)"]]"#),
        ("x = 10; 5x", r#"[["I:Assign(V:x, Q:10)"], ["I:Mul(Q:5, V:x)"]]"#),
        ("x = 5", r#"[["I:Assign(V:x, Q:5)"]]"#),
        ("x = ", r#"[["I:Assign(V:x)"]]"#),
        ("sin(PI)", r#"[["I:Call(F:sin, C:PI)"]]"#),
    ];

    let builtins = builtins::generate_symbol_map();
    let config = config::Config::default();
    let constants = config.constants.keys().map(|s| s.as_str());
    let units = distiller::get_default_units();

    for (input, expected_json) in cases {
        let mut workspace = parse(input, &builtins, constants.clone());
        janitor(&mut workspace);
        distiller(&mut workspace, &units);
        unroller(&mut workspace);
        
        let reconstructed = serde_json::Value::Array(reconstruct(&workspace));
        let expected_value: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        
        assert_eq!(reconstructed, expected_value, "Failed on unroller input: {:?}", input);
    }
}

#[test]
fn test_stray_closer() {
    let builtins = builtins::generate_symbol_map();
    let result = parse("5)", &builtins, std::iter::empty::<&str>());
    
    assert!(result.diagnostics.iter().any(|d| matches!(d.code, model::DiagnosticCode::StrayCloser)));
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
