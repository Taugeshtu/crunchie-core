pub mod config;
pub mod model;
pub mod parser;
pub mod builtins;

use config::Config;
use model::{EngineResult, ParserResult, TextEdit};
use std::collections::HashMap;

/// Stage 1: Structural Extraction
/// Performs the One-Pass Sweep. Returns a structural tree.
pub fn parse<'a>(
    text: &str,
    builtins: &HashMap<String, i32>,
    constants: impl IntoIterator<Item = &'a str>,
) -> ParserResult {
    parser::sweep(text, builtins, constants)
}

/// Stage 2.1: The Janitor
/// Scrubs the raw structural soup for mathematical sanity.
pub fn janitor(raw: ParserResult) -> ParserResult {
    raw // Stub
}

/// Stage 2.2: The Distiller
/// Assigns roles to symbols and marries values to their units.
pub fn distiller(_cleaned: ParserResult) -> model::CoupledResult {
    model::CoupledResult { lines: Vec::new() } // Stub
}

/// Stage 2.3: The Unroller
/// Flattens the nested hierarchy into a linear "Tape" of instructions.
pub fn unroller(_coupled: model::CoupledResult) -> model::Tape {
    model::Tape {
        instructions: Vec::new(),
        assignments: HashMap::new(),
        queries: Vec::new(),
    } // Stub
}

/// Stage 2.4: The Executioner
/// Final pass that performs the actual computation.
pub fn executioner(_tape: model::Tape, _config: &Config) -> EngineResult {
    EngineResult::default() // Stub
}

/// Stage 2: Semantic Analysis & Evaluation
/// Processes the ParsedBuffer, evaluates the math, and finds errors.
pub fn evaluate(_text: &str, parsed: &ParserResult, config: &Config) -> EngineResult {
    // For now, we clone 'parsed' to pass it to the janitor since evaluate takes a reference.
    // In the future, evaluate might be refactored or its sub-stages might take references.
    let cleaned = janitor(parsed.clone());
    let coupled = distiller(cleaned);
    let tape = unroller(coupled);
    executioner(tape, config)
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

    let parsed = parse(text, &builtins, constants);
    let mut engine_result = evaluate(text, &parsed, config);


    // Merge diagnostics from parsing and engine
    let mut diagnostics = parsed.diagnostics;
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


