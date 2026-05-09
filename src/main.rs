use std::collections::HashMap;

fn main() {
    let test_cases = [
        "5",
        "x = 5",
        "3 + (1 + 2)",
        "x = 5 # comment",
        "x=1; y=2",
        "z = (3, 5\n 7)",
        "z = (3; 5)",
        "outer = (\n    1 + 1, // first\n    (2 + 2) # second\n)",
        "x = (5 6)",
        "-5",
    ];

    let builtins = crunchie_core::parser::default_builtins();
    let constants = vec!["PI", "TAU", "E"];

    for (i, &case) in test_cases.iter().enumerate() {
        println!("========================================");
        println!("Test Case {}: {:?}", i, case);
        println!("========================================");
        
        let result = crunchie_core::parse(case, &builtins, constants.clone());
        
        // Print the reconstructed flat topology for debugging
        println!("--- Parser Result Topology ---");
        for (id, container) in &result.containers {
            let contents: Vec<String> = container.contents.iter().map(|u| {
                if let Some(c) = result.containers.get(&u.id) {
                    format!("Container({})", u.id)
                } else {
                    // Try to find the symbol string by ID
                    let sym = result.symbols.iter().find(|(_, v)| **v == u.id).map(|(k, _)| k.as_str()).unwrap_or("?");
                    format!("Sym({}, id={})", sym, u.id)
                }
            }).collect();
            println!("Container {} (valid={}): {:?}", id, container.valid, contents);
        }
        
        if !result.diagnostics.is_empty() {
            println!("Diagnostics: {:?}", result.diagnostics);
        }
        println!("\n");
    }
}
