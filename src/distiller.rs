use crate::model::{Atom, Workspace, Entity, Diagnostic, DiagnosticCode, Span, Position};
use std::collections::{HashSet, HashMap};
use fend_core::{lexer, Context, interrupt::Never, value::Value as FendValue};

/// The Distiller acts as the semantic bridge.
/// It converts Atom::Raw strings into typed atoms using Fend's lexer.
pub fn distill(workspace: &mut Workspace) {
    let mut ctx = Context::new();
    let int = Never;

    // 1. Scout for all identifiers (Variables, Constants, Functions)
    let known_identifiers = collect_known_identifiers(workspace);

    // 2. Identify all Raw atoms.
    let mut raw_ids: Vec<i32> = Vec::new();
    let mut id_to_pos = HashMap::new();

    for container in workspace.containers.values() {
        for entity in &container.contents {
            if let Some(Atom::Raw(_)) = workspace.atoms.get(&entity.id) {
                if !id_to_pos.contains_key(&entity.id) {
                    raw_ids.push(entity.id);
                    id_to_pos.insert(entity.id, entity.position);
                }
            }
        }
    }

    // Pass 1: Process atoms that result in a 1:1 replacement.
    // This populates known_identifiers for Pass 2.
    let mut remaining_ids = Vec::new();
    let mut current_known = known_identifiers;

    for id in raw_ids {
        let text = if let Some(Atom::Raw(s)) = workspace.atoms.get(&id) {
            s.clone()
        } else {
            continue;
        };
        
        let results = fend_munch(&text, &current_known, &mut ctx, &int);
        if results.len() == 1 && !matches!(results[0].0, Atom::Poison) {
            let atom = results[0].0.clone();
            if let Atom::Variable(s) = &atom {
                current_known.insert(s.clone());
            }
            workspace.atoms.insert(id, atom);
        } else {
            remaining_ids.push((id, text));
        }
    }

    // Pass 2: Process remaining atoms (expansions and poison).
    for (id, sym_text) in remaining_ids {
        let results = fend_munch(&sym_text, &current_known, &mut ctx, &int);
        let parent_pos = id_to_pos.get(&id).cloned().unwrap_or_default();
        
        if results.is_empty() || results.iter().any(|(s, _)| matches!(s, Atom::Poison)) {
            workspace.atoms.insert(id, Atom::Poison);
            
            workspace.diagnostics.push(Diagnostic {
                code: DiagnosticCode::MalformedSymbol,
                span: Span {
                    start: parent_pos,
                    end: parent_pos,
                },
            });
            continue;
        }

        if results.len() == 1 {
            workspace.atoms.insert(id, results[0].0.clone());
        } else {
            // 1:N Expansion (Container trick)
            let mut entities = Vec::new();
            for (part, local_offset) in results {
                let part_id = workspace.get_or_intern_atom_typed(part);
                let local_offset = local_offset as u32;
                let position = Position {
                    offset: parent_pos.offset + local_offset,
                    line: parent_pos.line,
                    col: parent_pos.col + local_offset,
                };
                entities.push(Entity { id: part_id, position });
            }
            
            let container_id = workspace.next_id;
            workspace.next_id += 1;
            workspace.containers.insert(container_id, crate::model::Container {
                contents: entities,
                corrupted: false,
                start_pos: parent_pos,
            });
            
            workspace.atoms.insert(id, Atom::Container(container_id));
        }
    }
}

fn collect_known_identifiers(workspace: &Workspace) -> HashSet<String> {
    let mut idents = HashSet::new();
    for sym in workspace.atoms.values() {
        match sym {
            Atom::Variable(s) | Atom::Constant(s) | Atom::Function(s) => {
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
) -> Vec<(Atom, usize)> {
    if s.is_empty() { return vec![]; }

    let lex = lexer::lex(s, ctx, int);
    let mut atoms = Vec::new();
    let attrs = fend_core::eval::Attrs::default();

    for token_res in lex {
        match token_res {
            Ok((token, offset)) => {
                match token {
                    lexer::Token::Num(n) => {
                        // Prevent multiple numbers in a single monolith (e.g. "5 5")
                        if atoms.iter().any(|(s, _)| matches!(s, Atom::Value(_))) {
                            return vec![(Atom::Poison, offset)];
                        }
                        atoms.push((Atom::Value(FendValue::Num(Box::new(n))), offset));
                    }
                    lexer::Token::Ident(ident) => {
                        let name = ident.as_str();
                        // If this is a split (monolith), it must be a known unit or identifier
                        if s.len() > name.len() {
                            let is_unit = fend_core::units::query_unit_static(name, attrs, ctx, int).is_ok();
                            if !is_unit && !known_identifiers.contains(name) {
                                return vec![(Atom::Poison, offset)];
                            }
                        }
                        atoms.push((Atom::Variable(name.to_string()), offset));
                    }
                    // Any structural Atom, StringLiteral, or Date found INSIDE a Raw monolith 
                    // is an error because the Parser should have split it out.
                    _ => return vec![(Atom::Poison, offset)],
                }
            }
            Err(_) => return vec![(Atom::Poison, 0)],
        }
    }

    if atoms.is_empty() {
        return vec![(Atom::Poison, 0)];
    }
    atoms
}