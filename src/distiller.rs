use crate::model::{Symbol, Workspace, Entity, Diagnostic, DiagnosticCode, Span, Position};
use std::collections::{HashSet, HashMap};
use fend_core::{lexer, Context, interrupt::Never, value::Value as FendValue};

/// The Distiller acts as the semantic bridge.
/// It converts Symbol::Raw strings into typed symbols using Fend's lexer.
pub fn distill(workspace: &mut Workspace) {
    let mut ctx = Context::new();
    let int = Never;

    // 1. Scout for all identifiers (Variables, Constants, Functions)
    let known_identifiers = collect_known_identifiers(workspace);

    // 2. Identify all Raw symbols and at least one representative offset for each.
    // This avoids borrowing conflicts and ensures we have provenance for diagnostics.
    let mut raw_work = HashMap::new(); // Map<ID, (offset, text)>
    for container in workspace.containers.values() {
        for entity in &container.contents {
            if let Some(Symbol::Raw(s)) = workspace.symbols.get(&entity.id) {
                if !raw_work.contains_key(&entity.id) {
                    raw_work.insert(entity.id, (entity.offset, s.clone()));
                }
            }
        }
    }

    for (id, (offset, sym_text)) in raw_work {
        let results = fend_munch(&sym_text, &known_identifiers, &mut ctx, &int);
        
        if results.is_empty() || results.iter().any(|s| matches!(s, Symbol::Poison)) {
            workspace.symbols.insert(id, Symbol::Poison);
            
            workspace.diagnostics.push(Diagnostic {
                code: DiagnosticCode::MalformedSymbol,
                span: Span {
                    start: Position { offset, line: 0, col: 0 },
                    end: Position { offset, line: 0, col: 0 },
                },
            });
            continue;
        }

        if results.len() == 1 {
            // 1:1 Replacement
            workspace.symbols.insert(id, results[0].clone());
        } else {
            // 1:N Expansion (ContainerRef trick)
            let mut entities = Vec::new();
            for part in results {
                let part_id = workspace.get_or_intern_symbol_typed(part);
                entities.push(Entity { id: part_id, offset: u32::MAX }); // Inherit parent offset
            }
            
            let container_id = workspace.next_id;
            workspace.next_id += 1;
            workspace.containers.insert(container_id, crate::model::Container {
                contents: entities,
                corrupted: false,
                start_pos: Position::default(),
            });
            
            workspace.symbols.insert(id, Symbol::ContainerRef(container_id));
        }
    }
}

fn collect_known_identifiers(workspace: &Workspace) -> HashSet<String> {
    let mut idents = HashSet::new();
    for sym in workspace.symbols.values() {
        match sym {
            Symbol::Variable(s) | Symbol::Constant(s) | Symbol::Function(s) => {
                idents.insert(s.clone());
            }
            _ => {}
        }
    }
    idents
}

fn fend_munch(
    s: &str, 
    known_identifiers: &HashSet<String>,
    ctx: &mut Context,
    int: &Never
) -> Vec<Symbol> {
    if s.is_empty() { return vec![]; }

    let lex = lexer::lex(s, ctx, int);
    let mut symbols = Vec::new();
    let attrs = fend_core::eval::Attrs::default();

    for token_res in lex {
        match token_res {
            Ok(token) => {
                match token {
                    lexer::Token::Num(n) => {
                        // Prevent multiple numbers in a single monolith (e.g. "5 5")
                        if symbols.iter().any(|s| matches!(s, Symbol::Value(_))) {
                            return vec![Symbol::Poison];
                        }
                        symbols.push(Symbol::Value(FendValue::Num(Box::new(n))));
                    }
                    lexer::Token::Ident(ident) => {
                        let name = ident.as_str();
                        // If this is a split (monolith), it must be a known unit or identifier
                        if s.len() > name.len() {
                            let is_unit = fend_core::units::query_unit_static(name, attrs, ctx, int).is_ok();
                            if !is_unit && !known_identifiers.contains(name) {
                                return vec![Symbol::Poison];
                            }
                        }
                        symbols.push(Symbol::Variable(name.to_string()));
                    }
                    // Any structural Symbol, StringLiteral, or Date found INSIDE a Raw monolith 
                    // is an error because the Parser should have split it out.
                    _ => return vec![Symbol::Poison],
                }
            }
            Err(_) => return vec![Symbol::Poison],
        }
    }

    if symbols.is_empty() {
        return vec![Symbol::Poison];
    }
    symbols
}
