use crate::model::{Comment, Container, Diagnostic, ParserResult, Unit};
use crate::builtins;
use std::collections::HashMap;

struct ParserState {
    next_id: i32,
    stack: Vec<i32>,
    result: ParserResult,

    // Omnibus State Trackers
    active_sym: String,
    sym_start_offset: u32,
    line: u32,
    col: u32,
    in_comment: bool,
    comment_start_pos: crate::model::Position,
    skip_next: bool,
}

impl ParserState {
    fn new() -> Self {
        Self {
            next_id: 1,
            stack: Vec::new(),
            result: ParserResult {
                containers: HashMap::new(),
                symbols: HashMap::new(),
                comments: Vec::new(),
                diagnostics: Vec::new(),
            },
            active_sym: String::new(),
            sym_start_offset: 0,
            line: 0,
            col: 0,
            in_comment: false,
            comment_start_pos: crate::model::Position { offset: 0, line: 0, col: 0 },
            skip_next: false,
        }
    }

    /// Interns a symbol string and returns its ID
    fn get_symbol_id(&mut self, sym: &str) -> i32 {
        if let Some(&id) = self.result.symbols.get(sym) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.result.symbols.insert(sym.to_string(), id);
        id
    }

    /// Allocates a new container and returns its ID
    fn create_container(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.result.containers.insert(id, Container::default());
        id
    }

    /// Pushes a unit to the container currently at the top of the stack
    fn push_unit(&mut self, unit: Unit) {
        if let Some(&current_id) = self.stack.last() {
            if let Some(container) = self.result.containers.get_mut(&current_id) {
                container.contents.push(unit);
            }
        }
    }

    /// Flushes the active symbol into the current container
    fn flush_sym(&mut self) {
        if !self.active_sym.is_empty() {
            let sym = self.active_sym.clone();
            let id = self.get_symbol_id(&sym);
            let offset = self.sym_start_offset;
            self.push_unit(Unit { id, offset });
            self.active_sym.clear();
        }
    }
}

pub fn sweep<'a>(
    text: &str,
    builtins: &HashMap<String, i32>,
    constants: impl IntoIterator<Item = &'a str>,
) -> ParserResult {
    let mut state = ParserState::new();

    // 1. Initialization
    for (k, v) in builtins {
        state.result.symbols.insert(k.clone(), *v);
    }

    let mut current_constant_id = builtins::CONSTANTS_START_ID;
    for c in constants {
        state.result.symbols.insert(c.to_string(), current_constant_id);
        current_constant_id += 1;
    }
    
    // 2. Bootstrapping the Root
    let root_id = 0;
    state.result.containers.insert(root_id, Container::default());
    
    let first_line_id = state.create_container();
    
    // Push the line into the root, then push both to stack
    state.stack.push(root_id);
    state.push_unit(Unit { id: first_line_id, offset: 0 });
    state.stack.push(first_line_id);

    println!("Starting sweep... root_id: {}, first_line_id: {}", root_id, first_line_id);

    // 4. The Sweep Loop
    for (offset, char) in text.char_indices() {
        if state.skip_next {
            state.skip_next = false;
            continue;
        }

        let offset = offset as u32;
        let current_pos = crate::model::Position { offset, line: state.line, col: state.col };
        
        println!("  [Trace] char: {:?} | offset: {} | stack_depth: {}", char, offset, state.stack.len());

        //   A. Comment Handling State
        if state.in_comment {
            if char == '\n' {
                let span = crate::model::Span {
                    start: state.comment_start_pos,
                    end: current_pos,
                };
                state.result.comments.push(span);
                state.in_comment = false;
                // Note: We do NOT 'continue' here. The \n must also trigger 
                // the structural logic in Block C.
            } else {
                // Keep accumulating. We don't actually store the text (just the Span),
                // but we need to track line/col for the end position.
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
                let new_cid = state.create_container();
                state.push_unit(Unit { id: new_cid, offset });
                state.stack.push(new_cid);
            }
            ')' => {
                state.flush_sym();
                if state.stack.len() > 2 {
                    state.stack.pop();
                } else {
                    state.result.diagnostics.push(Diagnostic {
                        code: crate::model::DiagnosticCode::StrayCloser,
                        span: crate::model::Span { start: current_pos, end: current_pos },
                    });
                }
            }
            '\n' | ';' => {
                state.flush_sym();
                if state.stack.len() == 2 {
                    // Root level line termination
                    state.stack.pop();
                    let new_line_id = state.create_container();
                    // Root is always ID 0
                    if let Some(root) = state.result.containers.get_mut(&0) {
                        root.contents.push(Unit { id: new_line_id, offset });
                    }
                    state.stack.push(new_line_id);
                } else {
                    // Nested sequence marker
                    let op_id = state.get_symbol_id(",");
                    state.push_unit(Unit { id: op_id, offset });
                }
            }
            ' ' | '\t' => {
                state.flush_sym();
            }
            c if builtins::OPERATORS.contains(&c) => {
                state.flush_sym();
                let op_id = state.get_symbol_id(&c.to_string());
                state.push_unit(Unit { id: op_id, offset });
            }
            c if builtins::ILLEGAL_CHARS.contains(&c) => {
                state.flush_sym();
                state.result.diagnostics.push(Diagnostic {
                    code: crate::model::DiagnosticCode::IllegalCharacter,
                    span: crate::model::Span { start: current_pos, end: current_pos },
                });
            }
            _ => {
                if state.active_sym.is_empty() {
                    state.sym_start_offset = offset;
                }
                state.active_sym.push(char);
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
        let eof_pos = crate::model::Position { 
            offset: text.len() as u32, 
            line: state.line, 
            col: state.col 
        };
        state.result.comments.push(crate::model::Span {
            start: state.comment_start_pos,
            end: eof_pos,
        });
    }

    // Anything beyond [Root, Line] on the stack is unclosed
    while state.stack.len() > 2 {
        if let Some(cid) = state.stack.pop() {
            if let Some(container) = state.result.containers.get_mut(&cid) {
                container.valid = false;
            }
            
            let eof_pos = crate::model::Position { 
                offset: text.len() as u32, 
                line: state.line, 
                col: state.col 
            };
            state.result.diagnostics.push(Diagnostic {
                code: crate::model::DiagnosticCode::UnclosedContainer,
                span: crate::model::Span { start: eof_pos, end: eof_pos },
            });
        }
    }

    // 6. Return ParserResult
    state.result
}

