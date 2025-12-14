use std::env;
use std::fs;
use std::io::{self, Read};
use postman_linter_core::{run_linter, LintConfig};
use serde::Deserialize;
use std::collections::HashMap;

/// Structure pour parser le fichier de config exporté depuis l'IHM
#[derive(Deserialize)]
struct ExportedConfig {
    version: String,
    #[serde(rename = "enabledRules")]
    enabled_rules: Vec<String>,
    #[serde(rename = "customTemplates")]
    custom_templates: Option<HashMap<String, String>>,
}

fn print_usage() {
    eprintln!("Usage: postman-linter [OPTIONS] [COLLECTION_FILE]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --config <FILE>    Load rules configuration from JSON file");
    eprintln!("  --rules <RULES>    Comma-separated list of rule IDs to enable");
    eprintln!("  --help             Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cat collection.json | postman-linter");
    eprintln!("  postman-linter collection.json");
    eprintln!("  postman-linter --config linterman-rules-config.json collection.json");
    eprintln!("  postman-linter --rules test-http-status-mandatory,hardcoded-secrets collection.json");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut config_file: Option<String> = None;
    let mut rules_arg: Option<String> = None;
    let mut collection_file: Option<String> = None;
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    config_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a file path");
                    std::process::exit(1);
                }
            }
            "--rules" | "-r" => {
                if i + 1 < args.len() {
                    rules_arg = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --rules requires a comma-separated list");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                collection_file = Some(arg.to_string());
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
    }
    
    // Lire la collection (depuis fichier ou stdin)
    let collection_json = if let Some(file_path) = collection_file {
        fs::read_to_string(&file_path)
            .unwrap_or_else(|e| {
                eprintln!("Error reading collection file '{}': {}", file_path, e);
                std::process::exit(1);
            })
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    };
    
    // Parser la collection
    let collection: serde_json::Value = serde_json::from_str(&collection_json)
        .unwrap_or_else(|e| {
            eprintln!("Error parsing collection JSON: {}", e);
            std::process::exit(1);
        });
    
    // Construire la configuration
    let mut rules: Option<Vec<String>> = None;
    
    // Charger depuis le fichier de config si spécifié
    if let Some(config_path) = config_file {
        let config_json = fs::read_to_string(&config_path)
            .unwrap_or_else(|e| {
                eprintln!("Error reading config file '{}': {}", config_path, e);
                std::process::exit(1);
            });
        
        let exported_config: ExportedConfig = serde_json::from_str(&config_json)
            .unwrap_or_else(|e| {
                eprintln!("Error parsing config file: {}", e);
                std::process::exit(1);
            });
        
        rules = Some(exported_config.enabled_rules);
        
        // Note: custom_templates is ignored in the open-source CLI
        // Template customization is a SaaS-only feature
        if exported_config.custom_templates.is_some() {
            eprintln!("ℹ️  Note: custom_templates ignored (SaaS-only feature)");
        }
        
        eprintln!("✅ Loaded config: {} rules enabled", rules.as_ref().map(|r| r.len()).unwrap_or(0));
    }
    
    // Override avec --rules si spécifié
    if let Some(rules_str) = rules_arg {
        rules = Some(rules_str.split(',').map(|s| s.trim().to_string()).collect());
    }
    
    let config = LintConfig {
        local_only: true,
        rules,
        fix: None,
        custom_templates: None, // SaaS-only feature
    };
    
    // Exécuter le linter
    let result = run_linter(&collection, &config);
    
    // Afficher le résultat en JSON
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
