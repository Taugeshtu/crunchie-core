use super::*;
use crate::printer::print_workspace;

fn run_pipeline(input: &str) -> String {
    let builtins = builtins::generate_symbol_map();
    let config = config::Config::default();
    let constants = config.constants.keys().map(|s| s.as_str());

    let mut workspace = parse(input, &builtins, constants);
    let mut engine_result = evaluate(input, &mut workspace, &config);
    
    // Merge engine diagnostics back into workspace for the printer
    workspace.diagnostics.append(&mut engine_result.diagnostics);
    
    let mut out = print_workspace(&workspace);
    
    if !engine_result.edits.is_empty() {
        out.push_str("\n  Edits:\n");
        for edit in engine_result.edits {
            out.push_str(&format!("    - Insert '{}' at offset {}\n", edit.new_text, edit.span.start.offset));
        }
    }
    
    out
}

#[test]
fn test_distiller_basics() {
    let cases = [
        "5",
        "x = 5kg",
        "10cm3",
        "0xFF",
        "5M",
        "5 + 2",
        "sin(PI)",
        "cos(TAU)",
        "sqrt(E)",
    ];
    let output = cases.iter().map(|c| format!("Input: {}\n{}", c, run_pipeline(c))).collect::<Vec<_>>().join("\n---\n");
    insta::assert_snapshot!(output);
}

#[test]
fn test_distiller_poison() {
    let output = format!("Input: 1.2.3\n{}", run_pipeline("1.2.3"));
    insta::assert_snapshot!(output);
}

#[test]
fn test_distiller_smart_splitting_garbage() {
    let output = format!("Input: 65kg123\n{}", run_pipeline("65kg123"));
    insta::assert_snapshot!(output);
}

#[test]
fn test_janitor_cases() {
    let cases = [
        "(5)",
        "((5))",
        "3 + (1 + 2)",
        "x = 1; y = 2",
        "x = 1\ny = 2",
        "(1; 2)",
        "(1, \n 2)",
        "(,1,)",
        "()",
    ];
    let output = cases.iter().map(|c| format!("Input: {}\n{}", c, run_pipeline(c))).collect::<Vec<_>>().join("\n---\n");
    insta::assert_snapshot!(output);
}

#[test]
fn test_unroller_basics() {
    let cases = [
        "1 + 2 * 3",
        "5 cm",
        "5 PI",
        "x = 10; 5x",
        "3(1+2)",
        "x = 5",
        "x = ",
        "x = 10; x = ",
        "sin(PI)",
    ];
    let output = cases.iter().map(|c| format!("Input: {}\n{}", c, run_pipeline(c))).collect::<Vec<_>>().join("\n---\n");
    insta::assert_snapshot!(output);
}

#[test]
fn test_stray_closer() {
    let output = format!("Input: 5)\n{}", run_pipeline("5)"));
    insta::assert_snapshot!(output);
}

#[test]
fn test_unclosed_container() {
    let output = format!("Input: x = (5\n{}", run_pipeline("x = (5"));
    insta::assert_snapshot!(output);
}

#[test]
fn test_illegal_math_pipeline() {
    let cases = [
        "5 + + 3",
        "* 5",
        "10 /",
        "1 to",
        "to cm",
        "()()",
        "5 5 5",
        "x = = 5",
        "x = y = 5",
        "1.5.5",
        "10e-",
        "0b102",
        "0xG",
        "(5",
        "1m + 2s to",
    ];
    let output = cases.iter().map(|c| format!("Input: {}\n{}", c, run_pipeline(c))).collect::<Vec<_>>().join("\n---\n");
    insta::assert_snapshot!(output);
}
