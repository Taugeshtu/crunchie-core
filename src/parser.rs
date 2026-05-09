use crate::model::{Comment, Container, Diagnostic, ParserResult, Unit};
use std::collections::HashMap;

pub const CONSTANTS_START_ID: i32 = 1_000_000;

pub fn default_builtins() -> HashMap<String, i32> {
    let mut m = HashMap::new();
    let builtins = [
        ("+", -1), ("-", -2), ("*", -3), ("/", -4), ("=", -5), ("^", -6),
        (",", -7)
    ];
    for (k, v) in builtins {
        m.insert(k.to_string(), v);
    }
    m
}

struct ParserState {
    next_id: i32,
    stack: Vec<i32>,
    result: ParserResult,
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

    let mut current_constant_id = CONSTANTS_START_ID;
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

    // 3. State Trackers for the Loop
    // - `active_sym`: String accumulator for the current symbol being read.
    // - `sym_start_offset`: Where the current symbol started.
    // - Trackers for line/col to build Spans later.
    // - Trackers for comment states (in_comment, start_pos, text_accumulator).

    // 4. The Sweep Loop
    // for (offset, char) in text.char_indices() {
        //   Calculate current Position (offset, line, col)

        //   A. Comment Handling State
        //      - If currently `in_comment`, accumulate text. 
        //      - If `\n`, finalize the comment, push to lists, reset state.
        //      - (Note: \n still needs to be processed by the structural logic below, 
        //        so we don't 'continue' on a newline that ends a comment).
        
        //   B. Comment Triggers
        //      - If `#` or `//`, flush the `active_sym`, flip `in_comment` to true, `continue`.

        //   C. Structural Triggers
        //      match char {
        //          '(' => {
        //              - flush `active_sym`
        //              - Allocate new ID, create Container, push to parent's contents.
        //              - Push new ID to stack.
        //          }
        //          ')' => {
        //              - flush `active_sym`
        //              - if stack depth > 2: pop stack.
        //              - else: emit Diagnostic (StrayCloser).
        //          }
        //          '\n' | ';' => {
        //              - flush `active_sym`
        //              - THE TWIN RULE:
        //                - If depth == 2 (Root level): pop line, start new line container, push to stack.
        //                - If depth > 2 (Nested): lookup `,` operator ID, push to parent's contents.
        //          }
        //          ' ' | '\t' => {
        //              - flush `active_sym` (Whitespace just ends symbols)
        //          }
        //          '+' | '-' | '*' | '/' | '=' | '^' | ',' => {
        //              - flush `active_sym`
        //              - lookup operator ID, push to parent's contents.
        //          }
        //          _ => {
        //              - This is a standard character.
        //              - If `active_sym` is empty, record `sym_start_offset`.
        //              - Push char to `active_sym`.
        //          }
        //      }
        
        //   D. Line/Col Maintenance
        //      - If char == '\n', line += 1, col = 0. Else col += 1.
    // }

    // 5. Cleanup
    // - Flush any trailing `active_sym`.
    // - If EOF reached while `in_comment`, finalize the trailing comment.
    // - If `stack` depth > 2, pop remaining containers and mark them `valid = false`.

    // 6. Return ParserResult
    todo!("Implement the sweep")
}
