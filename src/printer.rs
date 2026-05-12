use crate::model::{Workspace, Atom, Entity, Container};
use std::collections::HashMap;

/// Prints the workspace topology in a human-readable format.
pub fn print_workspace(workspace: &Workspace) -> String {
    let mut out = String::new();
    let mut reverse_intern = HashMap::new();
    for (k, &v) in &workspace.intern_map {
        reverse_intern.insert(v, k.clone());
    }

    out.push_str("Workspace {\n");
    
    if !workspace.diagnostics.is_empty() {
        out.push_str("  Diagnostics:\n");
        for diag in &workspace.diagnostics {
            out.push_str(&format!("    - {:?} at offset {:?}\n", diag.code, diag.span.start.offset));
        }
    }

    out.push_str("  Lines:\n");
    if let Some(root) = workspace.containers.get(&0) {
        for (i, entity) in root.contents.iter().enumerate() {
            out.push_str(&format!("    Line {}:\n", i + 1));
            print_entity(workspace, entity, &reverse_intern, 6, &mut out);
        }
    } else {
        out.push_str("    (No root container)\n");
    }

    out.push_str("}\n");
    out
}

fn print_entity(
    workspace: &Workspace,
    entity: &Entity,
    reverse_intern: &HashMap<i32, String>,
    indent: usize,
    out: &mut String,
) {
    let spaces = " ".repeat(indent);
    match workspace.atoms.get(&entity.id) {
        Some(Atom::Container(cid)) => {
            if let Some(container) = workspace.containers.get(cid) {
                out.push_str(&format!("{}[Container {}] {}{}\n", 
                    spaces, 
                    cid, 
                    if container.corrupted { "(CORRUPTED) " } else { "" },
                    if container.contents.is_empty() { "(empty)" } else { "" }
                ));
                for child in &container.contents {
                    print_entity(workspace, child, reverse_intern, indent + 2, out);
                }
            }
        }
        Some(Atom::Instruction { op, args }) => {
            out.push_str(&format!("{}[Instr] {:?}\n", spaces, op));
            for arg_id in args {
                print_entity(workspace, &Entity { id: *arg_id, offset: 0 }, reverse_intern, indent + 2, out);
            }
        }
        Some(Atom::Value(v)) => {
            let mut ctx = fend_core::Context::new();
            let mut spans = Vec::new();
            v.format(0, &mut spans, fend_core::eval::Attrs::default(), false, &mut ctx, &fend_core::interrupt::Never).unwrap();
            let formatted: String = spans.iter().map(|s| s.string.clone()).collect();
            out.push_str(&format!("{}[Value] {}\n", spaces, formatted));
        }
        Some(Atom::Variable(v)) => out.push_str(&format!("{}[Var] {}\n", spaces, v)),
        Some(Atom::Operator(op)) => out.push_str(&format!("{}[Op] {:?}\n", spaces, op)),
        Some(Atom::Constant(c)) => out.push_str(&format!("{}[Const] {}\n", spaces, c)),
        Some(Atom::Function(f)) => out.push_str(&format!("{}[Func] {}\n", spaces, f)),
        Some(Atom::Poison) => out.push_str(&format!("{}[POISON]\n", spaces)),
        Some(Atom::Raw(s)) => out.push_str(&format!("{}[Raw] '{}'\n", spaces, s)),
        _ => out.push_str(&format!("{}[Unknown ID] {}\n", spaces, entity.id)),
    }
}
