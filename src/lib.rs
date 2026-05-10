pub mod config;
pub mod model;
pub mod parser;
pub mod builtins;
pub mod janitor;
pub mod distiller;
pub mod unroller;

use config::Config;
use model::{EngineResult, TextEdit, Workspace};
use std::collections::HashMap;

/// Stage 1: Structural Extraction
/// Performs the One-Pass Sweep. Returns a structural tree.
pub fn parse<'a>(
    text: &str,
    builtins: &HashMap<String, i32>,
    constants: impl IntoIterator<Item = &'a str>,
) -> Workspace {
    parser::sweep(text, builtins, constants)
}

/// Stage 2.1: The Janitor
/// Scrubs the raw structural soup for mathematical sanity.
pub fn janitor(workspace: &mut Workspace) {
    janitor::scrub(workspace);
}

/// Stage 2.2: The Distiller
/// Assigns roles to symbols and marries values to their units.
pub fn distiller(workspace: &mut Workspace) {
    let units = distiller::get_default_units();
    distiller::distill(workspace, &units);
}

/// Stage 2.3: The Unroller
/// Flattens the nested hierarchy into a linear "Tape" of instructions.
pub fn unroller(workspace: &mut Workspace) {
    unroller::unroll(workspace);
}

/// Stage 2.4: The Executioner
/// Final pass that performs the actual computation.
// pub fn executioner(_tape: model::Tape, _config: &Config) -> EngineResult {
//     EngineResult::default() // Stub
// }

/// Stage 2: Semantic Analysis & Evaluation
/// Processes the Workspace, evaluates the math, and finds errors.
pub fn evaluate(_text: &str, workspace: &mut Workspace, _config: &Config) -> EngineResult {
    janitor(workspace);
    distiller(workspace);
    unroller(workspace);
    // executioner(workspace, config)
    EngineResult::default()
}

/// Stage 3: Utility for applying fills
/// Applies text edits (insertions, replacements) to the original text.
pub fn apply_edits(_text: &str, _edits: &[TextEdit]) -> String {
    unimplemented!("Applying edits not yet implemented")
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
