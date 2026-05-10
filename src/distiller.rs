use crate::model::{Symbol, Workspace, Entity, OpCode, Diagnostic, DiagnosticCode, Span, Position};
use std::collections::{HashSet, HashMap};

/// The Distiller acts as the semantic bridge.
/// It converts Symbol::Raw strings into typed symbols like Quantity, PhysUnit, or Variable.
pub fn distill(workspace: &mut Workspace, known_units: &HashSet<String>) {
    let mut expansions: HashMap<i32, Result<Vec<Symbol>, DiagnosticCode>> = HashMap::new();

    // 1. Identify all Raw symbols and munch them once
    for (&id, sym) in &workspace.symbols {
        if let Symbol::Raw(s) = sym {
            expansions.insert(id, munch(s, known_units));
        }
    }

    // 2. Apply transformations to containers
    for container in workspace.containers.values_mut() {
        let mut new_contents = Vec::new();
        let mut changed = false;

        for entity in &container.contents {
            if let Some(res) = expansions.get(&entity.id) {
                match res {
                    Ok(symbols) => {
                        if symbols.len() == 1 {
                            // 1:1 Replacement (in-place in symbols map)
                            workspace.symbols.insert(entity.id, symbols[0].clone());
                            new_contents.push(*entity);
                        } else {
                            // 1:N Expansion
                            changed = true;
                            for sym in symbols {
                                let new_id = workspace.next_id;
                                workspace.next_id += 1;
                                workspace.symbols.insert(new_id, sym.clone());
                                new_contents.push(Entity { id: new_id, offset: entity.offset });
                            }
                        }
                    }
                    Err(code) => {
                        // Poisoning
                        workspace.symbols.insert(entity.id, Symbol::Poison);
                        workspace.diagnostics.push(Diagnostic {
                            code: *code,
                            span: Span {
                                start: Position { offset: entity.offset, line: 0, col: 0 }, // TODO: accurate line/col
                                end: Position { offset: entity.offset, line: 0, col: 0 }, // TODO: accurate end
                            },
                        });
                        new_contents.push(*entity);
                    }
                }
            } else {
                new_contents.push(*entity);
            }
        }

        if changed {
            container.contents = new_contents;
        }
    }
}

pub fn munch(s: &str, known_units: &HashSet<String>) -> Result<Vec<Symbol>, DiagnosticCode> {
    if s.is_empty() { return Ok(vec![]); }

    // Phase 1: Lexical Split
    let (num_str, suffix_str) = split_lexical(s);

    if num_str.is_empty() {
        // No number. Check if it's a known unit.
        if known_units.contains(s) {
            return Ok(vec![Symbol::PhysUnit(s.to_string())]);
        }
        // Fallback to variable.
        return Ok(vec![Symbol::Variable(s.to_string())]);
    }

    // Phase 2: Numeric Evaluation
    let clean_num = num_str.replace('_', "");
    let val = if clean_num.starts_with("0x") {
        u64::from_str_radix(&clean_num[2..], 16).map(|v| v as f64).map_err(|_| ())
    } else if clean_num.starts_with("0b") {
        u64::from_str_radix(&clean_num[2..], 2).map(|v| v as f64).map_err(|_| ())
    } else {
        clean_num.parse::<f64>().map_err(|_| ())
    };

    let val = match val {
        Ok(v) => v,
        Err(_) => return Err(DiagnosticCode::MalformedNumber),
    };

    if suffix_str.is_empty() {
        return Ok(vec![Symbol::Quantity(val)]);
    }

    // Phase 3: Suffix Resolution
    // 1. SI multipliers
    if suffix_str == "k" || suffix_str == "K" {
        return Ok(vec![Symbol::Quantity(val * 1000.0)]);
    }
    if suffix_str == "M" {
        return Ok(vec![Symbol::Quantity(val * 1_000_000.0)]);
    }

    // 2. Pure Physical Unit
    if known_units.contains(suffix_str) {
        return Ok(vec![Symbol::Quantity(val), Symbol::PhysUnit(suffix_str.to_string())]);
    }

    // 3. Power Suffix Expansion (cm3)
    if suffix_str.len() > 1 {
        let last_char = suffix_str.chars().last().unwrap();
        if last_char.is_ascii_digit() {
            if let Some(power) = last_char.to_digit(10) {
                if (2..=5).contains(&power) {
                    let prefix = &suffix_str[..suffix_str.len() - 1];
                    if known_units.contains(prefix) {
                        return Ok(vec![
                            Symbol::Quantity(val),
                            Symbol::PhysUnit(prefix.to_string()),
                            Symbol::Operator(OpCode::Pow),
                            Symbol::Quantity(power as f64),
                        ]);
                    }
                }
            }
        }
    }

    // 4. Garbage fallback
    Err(DiagnosticCode::MalformedSymbol)
}

fn split_lexical(s: &str) -> (&str, &str) {
    if s.starts_with("0x") {
        let end = s[2..].find(|c: char| !c.is_ascii_hexdigit() && c != '_')
            .map(|i| i + 2)
            .unwrap_or(s.len());
        return (&s[..end], &s[end..]);
    }
    if s.starts_with("0b") {
        let end = s[2..].find(|c: char| c != '0' && c != '1' && c != '_')
            .map(|i| i + 2)
            .unwrap_or(s.len());
        return (&s[..end], &s[end..]);
    }

    // Decimal / Scientific
    let mut end = 0;
    let chars: Vec<char> = s.chars().collect();
    let mut saw_e = false;

    while end < chars.len() {
        let c = chars[end];
        if c.is_ascii_digit() || c == '.' || c == '_' {
            end += 1;
        } else if (c == 'e' || c == 'E') && !saw_e {
            // Check if next is digit or +/-
            if end + 1 < chars.len() {
                let next = chars[end + 1];
                if next.is_ascii_digit() || next == '+' || next == '-' {
                    saw_e = true;
                    end += 2; // skip e and sign/digit
                    
                    // Consume any following digits
                    while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '_') {
                        end += 1;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    (&s[..end], &s[end..])
}

pub fn get_default_units() -> HashSet<String> {
    let mut units = HashSet::new();
    for u in &["m", "cm", "mm", "km", "kg", "g", "mg", "s", "min", "h", "mph", "kph", "lbs", "oz", "degC", "degF", "K", "in", "ft", "yd", "mi"] {
        units.insert(u.to_string());
    }
    units
}
