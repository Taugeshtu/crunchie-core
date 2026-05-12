use crate::model::{Container, Diagnostic, DiagnosticCode, Entity, Position, Span, Atom, Workspace};
use crate::builtins;
use std::collections::HashMap;

struct ParserState {
    workspace: Workspace,
    stack: Vec<i32>,

    // Omnibus State Trackers
    active_sym: String,
    sym_start_pos: Position,
    line: u32,
    col: u32,
    in_comment: bool,
    comment_start_pos: Position,
    skip_next: bool,
    skip_bytes: usize,
}

impl ParserState {
    fn new() -> Self {
        Self {
            workspace: Workspace::default(),
            stack: Vec::new(),
            active_sym: String::new(),
            sym_start_pos: Position::default(),
            line: 0,
            col: 0,
            in_comment: false,
            comment_start_pos: Position { offset: 0, line: 0, col: 0 },
            skip_next: false,
            skip_bytes: 0,
        }
    }

    /// Allocates a new container and returns its ID
    fn create_container(&mut self, start_pos: Position) -> i32 {
        let id = self.workspace.next_id;
        self.workspace.next_id += 1;
        self.workspace.containers.insert(id, Container {
            start_pos,
            ..Default::default()
        });
        self.workspace.atoms.insert(id, Atom::Container(id));
        id
    }

    /// Pushes an entity to the container currently at the top of the stack
    fn push_entity(&mut self, entity: Entity) {
        if let Some(&current_id) = self.stack.last() {
            if let Some(container) = self.workspace.containers.get_mut(&current_id) {
                container.contents.push(entity);
            }
        }
    }

    /// Flushes the active symbol into the current container
    fn flush_sym(&mut self) {
        if !self.active_sym.is_empty() {
            let sym = self.active_sym.clone();
            let id = self.workspace.get_or_intern_atom(&sym);
            let position = self.sym_start_pos;
            self.push_entity(Entity { id, position });
            self.active_sym.clear();
        }
    }
}

pub fn sweep<'a>(
    text: &str,
    builtins: &HashMap<String, i32>,
    constants: impl IntoIterator<Item = &'a str>,
) -> Workspace {
    let mut state = ParserState::new();

    // 1. Initialization
    for (k, v) in builtins {
        state.workspace.intern_map.insert(k.clone(), *v);
        let sym = if *v <= builtins::CONSTANTS_START_ID {
            Atom::Constant(k.clone())
        } else if *v <= builtins::FUNCTIONS_START_ID {
            Atom::Function(k.clone())
        } else if *v <= -1 {
            if let Some(op) = builtins::get_operator(k) {
                Atom::Operator(op)
            } else {
                Atom::Raw(k.clone())
            }
        } else {
            Atom::Raw(k.clone())
        };
        state.workspace.atoms.insert(*v, sym);
    }

    for c in constants {
        if !state.workspace.intern_map.contains_key(c) {
            let id = state.workspace.next_id;
            state.workspace.next_id += 1;
            state.workspace.intern_map.insert(c.to_string(), id);
            state.workspace.atoms.insert(id, Atom::Constant(c.to_string()));
        }
    }
    
    // 2. Bootstrapping the Root
    let root_id = 0;
    state.workspace.containers.insert(root_id, Container::default());
    state.stack.push(root_id);

    // 4. The Sweep Loop
    for (offset, char) in text.char_indices() {
        if state.skip_bytes > 0 {
            state.skip_bytes = state.skip_bytes.saturating_sub(char.len_utf8());
            if char == '\n' { state.line += 1; state.col = 0; } else { state.col += 1; }
            continue;
        }

        if state.skip_next {
            state.skip_next = false;
            continue;
        }

        let offset = offset as u32;
        let current_pos = Position { offset, line: state.line, col: state.col };

        //   A. Comment Handling State
        if state.in_comment {
            if char == '\n' {
                let span = Span {
                    start: state.comment_start_pos,
                    end: current_pos,
                };
                state.workspace.comments.push(span);
                state.in_comment = false;
                // Note: We do NOT 'continue' here. The \n must also trigger 
                // the structural logic in Block C.
            } else {
                if char == '\n' { state.line += 1; state.col = 0; } else { state.col += 1; }
                continue;
            }
        }

        //   B. Comment Triggers
        let is_slash_slash = char == '/' && text.as_bytes().get(offset as usize + 1) == Some(&b'/');
        if char == '#' || is_slash_slash {
            state.flush_sym();
            
            state.in_comment = true;
            state.comment_start_pos = current_pos;
            
            if is_slash_slash {
                state.skip_next = true;
                state.col += 2; // Advance column for both slashes
            } else {
                state.col += 1;
            }
            continue;
        }

        //   C. Structural Triggers
        match char {
            '(' => {
                state.flush_sym();
                let new_cid = state.create_container(current_pos);
                state.push_entity(Entity { id: new_cid, position: current_pos });
                state.stack.push(new_cid);
            }
            ')' => {
                state.flush_sym();
                if state.stack.len() > 1 {
                    state.stack.pop();
                } else {
                    state.workspace.diagnostics.push(Diagnostic {
                        code: DiagnosticCode::StrayCloser,
                        span: Span { start: current_pos, end: current_pos },
                    });
                }
            }
            ' ' | '\t' => {
                state.flush_sym();
            }
            c if builtins::ILLEGAL_CHARS.contains(&c) => {
                state.flush_sym();
                state.workspace.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::IllegalCharacter,
                    span: Span { start: current_pos, end: current_pos },
                });
            }
            _ => {
                let remaining = &text[offset as usize..];
                if let Some(&op) = builtins::PUNCTUATION_OPERATORS.iter().find(|&&op| remaining.starts_with(op)) {
                    let is_exponent_sign = (op == "+" || op == "-") && {
                        let sym = &state.active_sym;
                        if sym.ends_with('e') || sym.ends_with('E') {
                            let prefix = &sym[..sym.len() - 1];
                            prefix.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_') && prefix.chars().any(|c| c.is_ascii_digit())
                        } else {
                            false
                        }
                    };

                    if is_exponent_sign {
                        state.active_sym.push_str(op);
                        state.skip_bytes = op.len() - char.len_utf8();
                    } else {
                        state.flush_sym();
                        let op_id = state.workspace.get_or_intern_atom(op);
                        state.push_entity(Entity { id: op_id, position: current_pos });
                        state.skip_bytes = op.len() - char.len_utf8();
                    }
                } else {
                    if state.active_sym.is_empty() {
                        state.sym_start_pos = current_pos;
                    }
                    state.active_sym.push(char);
                }
            }
        }

        //   D. Line/Col Maintenance
        if char == '\n' {
            state.line += 1;
            state.col = 0;
        } else {
            state.col += 1;
        }
    }

    // 5. Cleanup
    state.flush_sym();

    if state.in_comment {
        let eof_pos = Position { 
            offset: text.len() as u32, 
            line: state.line, 
            col: state.col 
        };
        state.workspace.comments.push(Span {
            start: state.comment_start_pos,
            end: eof_pos,
        });
    }

    // Anything beyond [Root] on the stack is unclosed
    while state.stack.len() > 1 {
        if let Some(cid) = state.stack.pop() {
            let start_pos = if let Some(container) = state.workspace.containers.get_mut(&cid) {
                container.corrupted = true;
                container.start_pos
            } else {
                Position { offset: 0, line: 0, col: 0 }
            };
            
            let eof_pos = Position { 
                offset: text.len() as u32, 
                line: state.line, 
                col: state.col 
            };
            state.workspace.diagnostics.push(Diagnostic {
                code: DiagnosticCode::UnclosedContainer,
                span: Span { start: start_pos, end: eof_pos },
            });
        }
    }

    // 6. Return Workspace
    state.workspace
}