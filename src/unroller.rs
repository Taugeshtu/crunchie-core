use crate::model::{OpCode, FendOp, Symbol, Workspace, Entity, Container};

pub fn unroll(workspace: &mut Workspace) {
    let root_contents = if let Some(root) = workspace.containers.get(&0) {
        root.contents.clone()
    } else {
        return;
    };

    for entity in root_contents {
        if let Some(Symbol::ContainerRef(cid)) = workspace.symbols.get(&entity.id).cloned() {
            let rpn = get_rpn(workspace, cid, entity.offset);
            let mut tape = Vec::new();
            
            if let Some(_) = generate_tape(workspace, rpn, &mut tape) {
                let tape_id = workspace.next_id;
                workspace.next_id += 1;
                
                let mut tape_entities = Vec::new();
                for instr_id in tape {
                    tape_entities.push(Entity { id: instr_id, offset: entity.offset });
                }
                
                let start_pos = workspace.containers.get(&cid).map(|c| c.start_pos).unwrap_or_default();
                workspace.containers.insert(tape_id, Container {
                    contents: tape_entities,
                    corrupted: false,
                    start_pos,
                });
                
                workspace.symbols.insert(entity.id, Symbol::ContainerRef(tape_id));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntityKind {
    Quantity,
    ContainerRef,
    Variable,
    Constant,
    Function,
    Operator(OpCode),
    Other,
}

fn get_entity_kind(workspace: &Workspace, entity: &Entity) -> EntityKind {
    match workspace.symbols.get(&entity.id) {
        Some(Symbol::Value(_)) => EntityKind::Quantity,
        Some(Symbol::Variable(_)) => EntityKind::Variable,
        Some(Symbol::Constant(_)) => EntityKind::Constant,
        Some(Symbol::Function(_)) => EntityKind::Function,
        Some(Symbol::Operator(op)) => EntityKind::Operator(*op),
        Some(Symbol::ContainerRef(_)) => EntityKind::ContainerRef,
        _ => EntityKind::Other,
    }
}

fn should_inject_mul(left: EntityKind, right: EntityKind) -> bool {
    match (left, right) {
        (EntityKind::Quantity, EntityKind::ContainerRef) => true,
        (EntityKind::Quantity, EntityKind::Variable) => true,
        (EntityKind::Quantity, EntityKind::Constant) => true,
        (EntityKind::ContainerRef, EntityKind::ContainerRef) => true,
        (EntityKind::ContainerRef, EntityKind::Variable) => true,
        (EntityKind::ContainerRef, EntityKind::Constant) => true,
        (EntityKind::Variable, EntityKind::ContainerRef) => true,
        (EntityKind::Constant, EntityKind::ContainerRef) => true,
        _ => false,
    }
}

fn get_precedence(op: OpCode) -> u8 {
    match op {
        OpCode::Call => 11,
        OpCode::Fend(FendOp::Pow) => 10,
        OpCode::Fend(FendOp::Mul) | OpCode::Fend(FendOp::Div) | OpCode::Fend(FendOp::Mod) => 9,
        OpCode::Fend(FendOp::Add) | OpCode::Fend(FendOp::Sub) => 8,
        OpCode::Fend(FendOp::To) => 7,
        OpCode::Fend(FendOp::Equals) | OpCode::Fend(FendOp::DoubleEquals) | OpCode::Fend(FendOp::NotEquals) |
        OpCode::Greater | OpCode::Less | OpCode::GreaterEqual | OpCode::LessEqual |
        OpCode::AddAssign | OpCode::SubAssign | OpCode::MulAssign | OpCode::DivAssign => 6,
        OpCode::Sequence | OpCode::Fend(FendOp::Semicolon) => 5,
        OpCode::Comma => 4,
        _ => 0,
    }
}

fn get_rpn(workspace: &mut Workspace, container_id: i32, parent_offset: u32) -> Vec<Entity> {
    let contents = workspace.containers.get(&container_id).map(|c| c.contents.clone()).unwrap_or_default();
    
    let mut output_queue = Vec::new();
    let mut operator_stack = Vec::new();
    let mut last_kind = None;

    let mul_op_id = workspace.get_or_intern_symbol_typed(Symbol::Operator(OpCode::Fend(FendOp::Mul)));

    for mut entity in contents {
        // --- Inherited Offset Support ---
        if entity.offset == u32::MAX {
            entity.offset = parent_offset;
        }

        let kind = get_entity_kind(workspace, &entity);
        
        if let Some(lk) = last_kind {
            if should_inject_mul(lk, kind) {
                process_operator(workspace, mul_op_id, &mut operator_stack, &mut output_queue, entity.offset);
            }
        }

        match kind {
            EntityKind::Quantity | EntityKind::Variable | EntityKind::Constant => {
                output_queue.push(entity);
                last_kind = Some(kind);
            }
            EntityKind::ContainerRef => {
                if let Some(Symbol::ContainerRef(cid)) = workspace.symbols.get(&entity.id) {
                    let nested_rpn = get_rpn(workspace, *cid, entity.offset);
                    output_queue.extend(nested_rpn);
                }
                last_kind = Some(kind);
            }
            EntityKind::Operator(_op) => {
                process_operator(workspace, entity.id, &mut operator_stack, &mut output_queue, entity.offset);
                last_kind = Some(kind);
            }
            EntityKind::Function => {
                operator_stack.push(entity);
                last_kind = Some(kind);
            }
            _ => {
                output_queue.push(entity);
                last_kind = Some(kind);
            }
        }
    }
    
    while let Some(op) = operator_stack.pop() {
        output_queue.push(op);
    }
    
    output_queue
}

fn process_operator(
    workspace: &Workspace,
    op_id: i32,
    operator_stack: &mut Vec<Entity>,
    output_queue: &mut Vec<Entity>,
    offset: u32
) {
    let current_prec = if let Some(Symbol::Operator(op)) = workspace.symbols.get(&op_id) {
        get_precedence(*op)
    } else if let Some(Symbol::Function(_)) = workspace.symbols.get(&op_id) {
        get_precedence(OpCode::Call)
    } else {
        0
    };

    while let Some(top_entity) = operator_stack.last() {
        let top_prec = if let Some(Symbol::Operator(top_op)) = workspace.symbols.get(&top_entity.id) {
            get_precedence(*top_op)
        } else if let Some(Symbol::Function(_)) = workspace.symbols.get(&top_entity.id) {
            get_precedence(OpCode::Call)
        } else {
            0
        };

        if top_prec >= current_prec && top_prec != 0 {
            output_queue.push(operator_stack.pop().unwrap());
        } else {
            break;
        }
    }
    operator_stack.push(Entity { id: op_id, offset });
}

fn generate_tape(workspace: &mut Workspace, rpn: Vec<Entity>, tape: &mut Vec<i32>) -> Option<i32> {
    let mut value_stack = Vec::new();
    
    for entity in rpn {
        let sym = workspace.symbols.get(&entity.id).cloned();
        match sym {
            Some(Symbol::Operator(op)) => {
                if matches!(op, OpCode::Fend(FendOp::Equals) | OpCode::AddAssign | OpCode::SubAssign | OpCode::MulAssign | OpCode::DivAssign) {
                    let right = value_stack.pop();
                    let left = value_stack.pop();
                    
                    match (left, right) {
                        (Some(l), Some(r)) => {
                            let instr_id = workspace.next_id;
                            workspace.next_id += 1;
                            workspace.symbols.insert(instr_id, Symbol::Instruction { op, args: vec![l, r] });
                            tape.push(instr_id);
                            value_stack.push(instr_id);
                        }
                        (None, Some(r)) => {
                            // Query (one operand provided, it's popped as 'right')
                            let instr_id = workspace.next_id;
                            workspace.next_id += 1;
                            workspace.symbols.insert(instr_id, Symbol::Instruction { op, args: vec![r] });
                            tape.push(instr_id);
                            value_stack.push(instr_id);
                        }
                        (Some(l), None) => {
                            // Query (shouldn't happen with pop order but just in case)
                            let instr_id = workspace.next_id;
                            workspace.next_id += 1;
                            workspace.symbols.insert(instr_id, Symbol::Instruction { op, args: vec![l] });
                            tape.push(instr_id);
                            value_stack.push(instr_id);
                        }
                        _ => {
                            // Malformed
                        }
                    }
                } else {
                    let right = value_stack.pop();
                    let left = value_stack.pop();
                    
                    if let (Some(l), Some(r)) = (left, right) {
                        let instr_id = workspace.next_id;
                        workspace.next_id += 1;
                        workspace.symbols.insert(instr_id, Symbol::Instruction { op, args: vec![l, r] });
                        tape.push(instr_id);
                        value_stack.push(instr_id);
                    } else {
                        // Malformed
                    }
                }
            }
            Some(Symbol::Function(_)) => {
                let arg = value_stack.pop();
                if let Some(a) = arg {
                    let instr_id = workspace.next_id;
                    workspace.next_id += 1;
                    workspace.symbols.insert(instr_id, Symbol::Instruction { op: OpCode::Call, args: vec![entity.id, a] });
                    tape.push(instr_id);
                    value_stack.push(instr_id);
                }
            }
            _ => {
                value_stack.push(entity.id);
            }
        }
    }
    value_stack.pop()
}
