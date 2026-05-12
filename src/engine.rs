use crate::model::{Workspace, Atom, OpCode, FendOp, EngineResult, Diagnostic, DiagnosticCode, TextEdit, Position};
use crate::config::Config;
use std::collections::{HashMap, HashSet};
use fend_core::{Context, interrupt::Never, ast::{Expr, Bop, evaluate}};
use fend_core::ident::Ident;
use fend_core::value::Value;
use fend_core::eval::Attrs;

pub struct Executioner<'a> {
    pub workspace: &'a Workspace,
    pub config: &'a Config,
    pub text: &'a str,
    pub ctx: Context,
    pub state_map: HashMap<i32, Value>,
    pub pos_map: HashMap<i32, Position>,
    pub poison_set: HashSet<i32>,
    pub diagnostics: Vec<Diagnostic>,
    pub edits: Vec<TextEdit>,
}

impl<'a> Executioner<'a> {
    pub fn new(workspace: &'a Workspace, config: &'a Config, text: &'a str) -> Self {
        Self {
            workspace,
            config,
            text,
            ctx: Context::new(),
            state_map: HashMap::new(),
            pos_map: HashMap::new(),
            poison_set: HashSet::new(),
            diagnostics: Vec::new(),
            edits: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> EngineResult {
        let mut result = EngineResult::default();

        let root_contents = if let Some(root) = self.workspace.containers.get(&0) {
            root.contents.clone()
        } else {
            return result;
        };

        let int = Never;
        let attrs = Attrs::default();

        for entity in root_contents {
            if let Some(Atom::Container(cid)) = self.workspace.atoms.get(&entity.id).cloned() {
                let container = if let Some(c) = self.workspace.containers.get(&cid) {
                    c
                } else {
                    continue;
                };

                // 0. Position Mapping
                for child in &container.contents {
                    self.pos_map.insert(child.id, child.position);
                }

                // 1. Line Poison Check
                if container.corrupted {
                    for child in &container.contents {
                        self.poison_set.insert(child.id);
                    }
                    continue;
                }
                let mut line_poisoned = false;
                for child in &container.contents {
                    if let Some(Atom::Poison) = self.workspace.atoms.get(&child.id) {
                        line_poisoned = true;
                        break;
                    }
                }
                if line_poisoned {
                    for child in &container.contents {
                        self.poison_set.insert(child.id);
                    }
                    continue;
                }

                // 2. Tape Execution
                for (i, child) in container.contents.iter().enumerate() {
                    let is_last = i == container.contents.len() - 1;
                    
                    if self.poison_set.contains(&child.id) {
                        continue;
                    }

                    match self.workspace.atoms.get(&child.id).cloned() {
                        Some(Atom::Value(v)) => {
                            self.state_map.insert(child.id, v);
                        }
                        Some(Atom::Variable(_) | Atom::Constant(_)) => {
                            // We do not eagerly evaluate variables or constants.
                            // They are resolved dynamically by Fend when the AST is evaluated,
                            // allowing them to serve as LHS identifiers without throwing errors prematurely.
                        }
                        Some(Atom::Instruction { op, args }) => {
                            // Check if any args are poisoned
                            if args.iter().any(|&aid| self.poison_set.contains(&aid)) {
                                self.poison_set.insert(child.id);
                                continue;
                            }

                            // If this is the last instruction, we can resolve intents (Assignments, Queries, Assertions)
                            if is_last {
                                if let OpCode::Fend(FendOp::Equals) = op {
                                    self.handle_intent(&args, child.id, child.position);
                                    continue;
                                }
                            }

                            // Evaluate standard instruction
                            let expr_opt = self.build_expr(op, &args);
                            if let Some(expr) = expr_opt {
                                let mut spans = Vec::new();
                                match evaluate(expr, None, attrs, &mut spans, &mut self.ctx, &int) {
                                    Ok(val) => {
                                        self.state_map.insert(child.id, val);
                                    }
                                    Err(_) => {
                                        self.poison_set.insert(child.id);
                                        let span = self.get_instruction_span(&args, child.position);
                                        self.diagnostics.push(Diagnostic {
                                            code: DiagnosticCode::MalformedExpression, // Or a new EvaluationError
                                            span,
                                        });
                                    }
                                }
                            } else {
                                self.poison_set.insert(child.id);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        result.diagnostics = self.diagnostics.clone();
        result.edits = self.edits.clone();
        result
    }

    fn get_instruction_span(&self, args: &[i32], op_pos: Position) -> crate::model::Span {
        let mut min_pos = op_pos;
        let mut max_pos = op_pos;
        
        for &arg_id in args {
            if let Some(p) = self.pos_map.get(&arg_id) {
                if p.offset < min_pos.offset { min_pos = *p; }
                if p.offset > max_pos.offset { max_pos = *p; }
            }
        }
        
        crate::model::Span {
            start: min_pos,
            end: max_pos,
        }
    }

    fn handle_intent(&mut self, args: &[i32], id: i32, position: Position) {
        // args can be length 1 (Query) or length 2 (Assignment or Assertion)
        if args.len() == 1 {
            // Query
            let left_id = args[0];
            let expr = self.get_expr_for_id(left_id);
            if let Some(e) = expr {
                let mut spans = Vec::new();
                let int = Never;
                let attrs = Attrs::default();
                if let Ok(v) = evaluate(e, None, attrs, &mut spans, &mut self.ctx, &int) {
                    spans.clear();
                    if v.format(0, &mut spans, attrs, false, &mut self.ctx, &int).is_ok() {
                        let formatted: String = spans.iter().map(|s| s.string.clone()).collect();
                        let insert_offset = position.offset + 1; // Insert after the '='
                        let pos = self.workspace.get_position(insert_offset, self.text);
                        self.edits.push(TextEdit {
                            span: crate::model::Span {
                                start: pos,
                                end: pos,
                            },
                            new_text: format!(" {}", formatted),
                            value: Some(v),
                        });
                    }
                } else {
                    self.poison_set.insert(id);
                }
            }
        } else if args.len() == 2 {
            let left_id = args[0];
            let right_id = args[1];
            
            let is_lhs_var = if let Some(Atom::Variable(_)) = self.workspace.atoms.get(&left_id) {
                true
            } else {
                false
            };

            if is_lhs_var {
                // Assignment
                if let Some(Atom::Variable(v)) = self.workspace.atoms.get(&left_id).cloned() {
                    if let Some(r_expr) = self.get_expr_for_id(right_id) {
                        let expr = Expr::Assign(Ident::new_string(v), Box::new(r_expr));
                        let mut spans = Vec::new();
                        let int = Never;
                        let attrs = Attrs::default();
                        if evaluate(expr, None, attrs, &mut spans, &mut self.ctx, &int).is_err() {
                            self.poison_set.insert(id);
                        }
                    }
                }
            } else {
                // Assertion
                if let (Some(l_expr), Some(r_expr)) = (self.get_expr_for_id(left_id), self.get_expr_for_id(right_id)) {
                    let expr = Expr::Equality(true, Box::new(l_expr), Box::new(r_expr));
                    let mut spans = Vec::new();
                    let int = Never;
                    let attrs = Attrs::default();
                    match evaluate(expr, None, attrs, &mut spans, &mut self.ctx, &int) {
                        Ok(Value::Bool(true)) => { /* pass */ },
                        _ => {
                            self.poison_set.insert(id);
                            let span = self.get_instruction_span(args, position);
                            self.diagnostics.push(Diagnostic {
                                code: DiagnosticCode::MalformedExpression, // Or AssertionFailed
                                span,
                            });
                        }
                    }
                }
            }
        }
    }

    fn get_expr_for_id(&self, id: i32) -> Option<Expr> {
        if let Some(val) = self.state_map.get(&id) {
            Some(Expr::Literal(val.clone()))
        } else if let Some(Atom::Variable(v) | Atom::Constant(v)) = self.workspace.atoms.get(&id) {
            Some(Expr::Ident(Ident::new_string(v.clone())))
        } else if let Some(Atom::Function(f)) = self.workspace.atoms.get(&id) {
            Some(Expr::Ident(Ident::new_string(f.clone())))
        } else if let Some(Atom::Value(val)) = self.workspace.atoms.get(&id) {
            Some(Expr::Literal(val.clone()))
        } else {
            None
        }
    }

    fn build_expr(&self, op: OpCode, args: &[i32]) -> Option<Expr> {
        if let OpCode::Call = op {
            if args.len() == 2 {
                let func = self.get_expr_for_id(args[0])?;
                let arg = self.get_expr_for_id(args[1])?;
                return Some(Expr::ApplyFunctionCall(Box::new(func), Box::new(arg)));
            }
        }

        if let OpCode::Fend(fop) = op {
            match fop {
                FendOp::To => {
                    if args.len() == 2 {
                        let lhs = self.get_expr_for_id(args[0])?;
                        let rhs = self.get_expr_for_id(args[1])?;
                        return Some(Expr::As(Box::new(lhs), Box::new(rhs)));
                    }
                }
                FendOp::Add => {
                    if args.len() == 2 {
                        return Some(Expr::Bop(Bop::Plus, Box::new(self.get_expr_for_id(args[0])?), Box::new(self.get_expr_for_id(args[1])?)));
                    }
                }
                FendOp::Sub => {
                    if args.len() == 2 {
                        return Some(Expr::Bop(Bop::Minus, Box::new(self.get_expr_for_id(args[0])?), Box::new(self.get_expr_for_id(args[1])?)));
                    }
                }
                FendOp::Mul => {
                    if args.len() == 2 {
                        return Some(Expr::Bop(Bop::Mul, Box::new(self.get_expr_for_id(args[0])?), Box::new(self.get_expr_for_id(args[1])?)));
                    }
                }
                FendOp::Div => {
                    if args.len() == 2 {
                        return Some(Expr::Bop(Bop::Div, Box::new(self.get_expr_for_id(args[0])?), Box::new(self.get_expr_for_id(args[1])?)));
                    }
                }
                FendOp::Mod => {
                    if args.len() == 2 {
                        return Some(Expr::Bop(Bop::Mod, Box::new(self.get_expr_for_id(args[0])?), Box::new(self.get_expr_for_id(args[1])?)));
                    }
                }
                FendOp::Pow => {
                    if args.len() == 2 {
                        return Some(Expr::Bop(Bop::Pow, Box::new(self.get_expr_for_id(args[0])?), Box::new(self.get_expr_for_id(args[1])?)));
                    }
                }
                _ => {}
            }
        }

        None
    }
}
