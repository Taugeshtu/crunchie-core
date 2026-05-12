use crate::model::{Entity, Atom, Workspace, Container, Diagnostic, DiagnosticCode, Span, Position};

/// The Janitor scrubs the topological soup.
/// 1. Breaks Root into Lines.
/// 2. Normalizes sequences (\n, ; -> ,) inside containers.
/// 3. Flattens inert containers (1 child).
/// 4. Trims leading/trailing/stuttering sequences.
pub fn scrub(workspace: &mut Workspace) {
    let newline_id = workspace.intern_map.get("\n").cloned().unwrap_or(-1);
    let semicolon_id = workspace.intern_map.get(";").cloned().unwrap_or(-1);
    let comma_id = workspace.intern_map.get(",").cloned().unwrap_or(-1);

    // 1. Root Split
    let root_contents = if let Some(root) = workspace.containers.get(&0) {
        root.contents.clone()
    } else {
        return;
    };

    let mut new_root_contents = Vec::new();
    let mut current_line_entities = Vec::new();

    for entity in root_contents {
        if entity.id == newline_id || entity.id == semicolon_id {
            if !current_line_entities.is_empty() {
                let cleaned = process_sequence(workspace, current_line_entities, comma_id, newline_id, semicolon_id);
                if !cleaned.is_empty() {
                    let line_id = maybe_promote_line(workspace, cleaned);
                    new_root_contents.push(Entity { id: line_id, offset: 0 }); // TODO: better offset
                }
                current_line_entities = Vec::new();
            }
        } else {
            current_line_entities.push(entity);
        }
    }

    if !current_line_entities.is_empty() {
        let cleaned = process_sequence(workspace, current_line_entities, comma_id, newline_id, semicolon_id);
        if !cleaned.is_empty() {
            let line_id = maybe_promote_line(workspace, cleaned);
            new_root_contents.push(Entity { id: line_id, offset: 0 });
        }
    }

    if let Some(root) = workspace.containers.get_mut(&0) {
        root.contents = new_root_contents;
    }
}

/// Creates a line container and aggressively unwraps it if it only contains
/// a single other container, ensuring lines are as flat as possible.
fn maybe_promote_line(workspace: &mut Workspace, contents: Vec<Entity>) -> i32 {
    let mut current_contents = contents;
    let mut corrupted = false;

    // Aggressive De-stacking: If we only contain a single other container, 
    // we take its contents and repeat.
    while current_contents.len() == 1 {
        let id = current_contents[0].id;
        if let Some(Atom::Container(cid)) = workspace.atoms.get(&id) {
            if let Some(inner) = workspace.containers.get(cid) {
                corrupted |= inner.corrupted;
                current_contents = inner.contents.clone();
                continue;
            }
        }
        break;
    }

    let id = workspace.next_id;
    workspace.next_id += 1;
    workspace.containers.insert(id, Container {
        contents: current_contents,
        corrupted,
        start_pos: Position { offset: 0, line: 0, col: 0 },
    });
    workspace.atoms.insert(id, Atom::Container(id));
    id
}

fn process_sequence(
    workspace: &mut Workspace,
    entities: Vec<Entity>,
    comma_id: i32,
    newline_id: i32,
    semicolon_id: i32,
) -> Vec<Entity> {
    let mut rebuilt = Vec::new();

    for entity in entities {
        if let Some(Atom::Container(cid)) = workspace.atoms.get(&entity.id).cloned() {
            // Recurse into nested container
            let inner_contents = if let Some(c) = workspace.containers.get(&cid) {
                c.contents.clone()
            } else {
                Vec::new()
            };

            let mut cleaned_inner = process_sequence(workspace, inner_contents, comma_id, newline_id, semicolon_id);
            
            // Redundant Nesting Collapse (Step A): 
            // If the cleaned container only contains a single other container, unwrap it.
            while cleaned_inner.len() == 1 {
                let child_id = cleaned_inner[0].id;
                if let Some(Atom::Container(inner_cid)) = workspace.atoms.get(&child_id) {
                    if let Some(inner_c) = workspace.containers.get(inner_cid) {
                        let inner_corrupted = inner_c.corrupted;
                        let inner_contents = inner_c.contents.clone();
                        
                        // OR corrupted flags upward
                        if let Some(c) = workspace.containers.get_mut(&cid) {
                            c.corrupted |= inner_corrupted;
                        }
                        cleaned_inner = inner_contents;
                        continue;
                    }
                }
                break;
            }
            
            // Update the container with its (potentially unwrapped) contents
            if let Some(c) = workspace.containers.get_mut(&cid) {
                c.contents = cleaned_inner;
            }
            rebuilt.push(entity);
        } else if entity.id == newline_id || entity.id == semicolon_id {
            // Coerce to comma
            rebuilt.push(Entity { id: comma_id, offset: entity.offset });
        } else {
            rebuilt.push(entity);
        }
    }

    // Post-process: Stuttering, Leading/Trailing
    let mut final_entities = Vec::new();
    let mut last_was_seq = true; // effectively trims leading

    for entity in rebuilt {
        let is_seq = entity.id == comma_id;
        if is_seq {
            if !last_was_seq {
                final_entities.push(entity);
            } else {
                // StraySequence: redundant or leading
                workspace.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::StraySequence,
                    span: Span { 
                        start: Position { offset: entity.offset, line: 0, col: 0 },
                        end: Position { offset: entity.offset + 1, line: 0, col: 0 } 
                    },
                });
            }
            last_was_seq = true;
        } else {
            final_entities.push(entity);
            last_was_seq = false;
        }
    }

    // Trim trailing
    if let Some(last) = final_entities.last() {
        if last.id == comma_id {
            let offset = last.offset;
            final_entities.pop();
            workspace.diagnostics.push(Diagnostic {
                code: DiagnosticCode::StraySequence,
                span: Span { 
                    start: Position { offset, line: 0, col: 0 }, 
                    end: Position { offset: offset + 1, line: 0, col: 0 } 
                },
            });
        }
    }

    final_entities
}
