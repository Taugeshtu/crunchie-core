pub mod config;
pub mod model;
pub mod parser;
pub mod builtins;
pub mod janitor;
pub mod distiller;
pub mod unroller;
pub mod engine;
pub mod printer;

use config::Config;
use model::{EngineResult, TextEdit, Workspace};
use std::collections::HashMap;

/// Stage 0: Structural Extraction (Parser)
/// Performs the One-Pass Sweep. Returns a structural tree.
pub fn parse<'a>(
    text: &str,
    builtins: &HashMap<String, i32>,
    constants: impl IntoIterator<Item = &'a str>,
) -> Workspace {
    parser::sweep(text, builtins, constants)
}

/// Stage 1: The Distiller
/// Assigns roles to atoms and marries values to their units.
pub fn distiller(workspace: &mut Workspace) {
    distiller::distill(workspace);
}

/// Stage 2: The Janitor
/// Scrubs the raw structural soup for mathematical sanity.
pub fn janitor(workspace: &mut Workspace) {
    janitor::scrub(workspace);
}

/// Stage 3: The Unroller
/// Flattens the nested hierarchy into a linear "Tape" of instructions.
pub fn unroller(workspace: &mut Workspace) {
    unroller::unroll(workspace);
}

/// Stage 4: The Executioner
/// Final pass that performs the actual computation using Fend.
pub fn executioner(workspace: &mut Workspace, config: &Config, text: &str) -> EngineResult {
    let mut exec = engine::Executioner::new(workspace, config, text);
    exec.execute()
}

/// Semantic Analysis & Evaluation
/// Processes the Workspace, evaluates the math, and finds errors.
pub fn evaluate(text: &str, workspace: &mut Workspace, config: &Config) -> EngineResult {
    distiller(workspace);
    janitor(workspace);
    unroller(workspace);
    executioner(workspace, config, text)
}

/// Utility for applying fills
/// Applies text edits (insertions, replacements) to the original text.
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut result = text.to_string();
    let mut sorted_edits = edits.to_vec();
    
    // Sort edits by offset in descending order to avoid shifting issues
    sorted_edits.sort_by(|a, b| b.span.start.offset.cmp(&a.span.start.offset));

    for edit in sorted_edits {
        let start = edit.span.start.offset as usize;
        let end = edit.span.end.offset as usize;
        
        if start <= result.len() && end <= result.len() && start <= end {
            result.replace_range(start..end, &edit.new_text);
        }
    }

    result
}

/// Convenience Wrapper
/// Runs the entire pipeline and returns the modified buffer and any diagnostics.
pub fn process_buffer(text: &str, config: &Config) -> (String, Vec<model::Diagnostic>) {
    let builtins = builtins::generate_symbol_map();
    let constants = config.constants.keys().map(|s| s.as_str());

    let mut workspace = parse(text, &builtins, constants);
    let mut engine_result = evaluate(text, &mut workspace, config);


    // Merge diagnostics from parsing and engine
    let mut diagnostics = workspace.diagnostics.clone();
    diagnostics.append(&mut engine_result.diagnostics);

    let final_text = if config.generate_fills && !engine_result.edits.is_empty() {
        apply_edits(text, &engine_result.edits)
    } else {
        text.to_string()
    };

    (final_text, diagnostics)
}

#[cfg(test)]
mod tests;
