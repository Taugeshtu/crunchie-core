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

/// Stage 2: Semantic Analysis & Evaluation
/// Processes the ParsedBuffer, evaluates the math, and finds errors.
pub fn evaluate(_text: &str, _parsed: &ParserResult, _config: &Config) -> EngineResult {
    unimplemented!("Engine evaluation not yet implemented")
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
