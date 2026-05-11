use crate::model::{OpCode, Symbol, Workspace, Entity, Diagnostic, DiagnosticCode, Span, Container};

pub fn unroll(workspace: &mut Workspace) {
    let root_contents = if let Some(root) = workspace.containers.get(&0) {
        root.contents.clone()
    } else {
        return;
    };

    for entity in root_contents {
        if let Some(Symbol::ContainerRef(cid)) = workspace.symbols.get(&entity.id).cloned() {
            let rpn = get_rpn(workspace, cid);
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
    PhysUnit,
    ContainerRef,
    Variable,
    Constant,
    Function,
    Operator(OpCode),
    Other,
}

fn get_entity_kind(workspace: &Workspace, entity: &Entity) -> EntityKind {
    match workspace.symbols.get(&entity.id) {
        Some(Symbol::Quantity(_)) => EntityKind::Quantity,
        Some(Symbol::PhysUnit(_)) => EntityKind::PhysUnit,
        Some(Symbol::ContainerRef(_)) => EntityKind::ContainerRef,
        Some(Symbol::Variable(_)) => EntityKind::Variable,
        Some(Symbol::Constant(_)) => EntityKind::Constant,
        Some(Symbol::Function(_)) => EntityKind::Function,
        Some(Symbol::Operator(op)) => EntityKind::Operator(*op),
        _ => EntityKind::Other,
    }
}

fn should_inject_mul(left: EntityKind, right: EntityKind) -> bool {
    match (left, right) {
        (EntityKind::Quantity, EntityKind::PhysUnit) => true,
        (EntityKind::Quantity, EntityKind::ContainerRef) => true,
        (EntityKind::Quantity, EntityKind::Variable) => true,
        (EntityKind::Quantity, EntityKind::Constant) => true,
        (EntityKind::ContainerRef, EntityKind::ContainerRef) => true,
        _ => false,
    }
}

fn get_precedence(op: OpCode) -> u8 {
    match op {
        OpCode::Pow => 10,
        OpCode::Mul | OpCode::Div => 9,
        OpCode::Add | OpCode::Sub => 8,
        OpCode::To => 7,
        OpCode::Assign => 6,
        OpCode::Sequence => 5,
        OpCode::Call => 11,
    }
}

fn get_rpn(workspace: &mut Workspace, container_id: i32) -> Vec<Entity> {
    let contents = workspace.containers.get(&container_id).map(|c| c.contents.clone()).unwrap_or_default();
    
    let mut output_queue = Vec::new();
    let mut operator_stack = Vec::new();
    let mut last_kind = None;

    let mul_op_id = workspace.get_or_intern_symbol("*");

    for entity in contents {
        let kind = get_entity_kind(workspace, &entity);
        
        if let Some(lk) = last_kind {
            if should_inject_mul(lk, kind) {
                process_operator(workspace, mul_op_id, &mut operator_stack, &mut output_queue, entity.offset);
            }
        }

        match kind {
            EntityKind::Quantity | EntityKind::Variable | EntityKind::Constant | EntityKind::PhysUnit => {
                output_queue.push(entity);
                last_kind = Some(kind);
            }
            EntityKind::ContainerRef => {
                if let Some(Symbol::ContainerRef(cid)) = workspace.symbols.get(&entity.id) {
                    let nested_rpn = get_rpn(workspace, *cid);
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
                if op == OpCode::Assign {
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
                            // TODO: Add diagnostic
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
                        // TODO: Add diagnostic
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
