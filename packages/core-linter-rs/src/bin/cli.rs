use std::io::{self, Read};
use postman_linter_core::{run_linter, LintConfig};

fn main() {
    // Lire depuis stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Failed to read from stdin");
    
    // Parser la collection
    let collection: serde_json::Value = serde_json::from_str(&buffer)
        .expect("Failed to parse collection JSON");
    
    // Configuration par défaut
    let config = LintConfig {
        local_only: true,
        rules: None,
        fix: None,
    };
    
    // Exécuter le linter
    let result = run_linter(&collection, &config);
    
    // Afficher le résultat en JSON
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
