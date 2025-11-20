use crate::LintIssue;
use crate::utils;
use regex::Regex;
use serde_json::Value;

/// Règle : test-schema-validation-recommended
/// 
/// Vérifie que les requêtes JSON ont des tests de validation de schéma.
/// Patterns détectés :
/// - pm.response.to.have.jsonSchema()
/// - jsonSchema
/// - Schema_Validation
/// 
/// Sévérité : WARNING (-8%)
pub fn check(collection: &Value) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    
    if let Some(items) = collection["item"].as_array() {
        check_items(items, &mut issues, "", &[]);
    }
    
    issues
}

fn check_items(
    items: &[Value],
    issues: &mut Vec<LintIssue>,
    parent_path: &str,
    parent_scripts: &[String],
) {
    for (index, item) in items.iter().enumerate() {
        let default_name = format!("Item-{}", index + 1);
        let item_name = item["name"].as_str().unwrap_or(&default_name);
        let current_path = if parent_path.is_empty() {
            format!("/item[{}]", index)
        } else {
            format!("{}/item[{}]", parent_path, index)
        };
        
        // Si c'est une requête
        if item.get("request").is_some() {
            check_request_schema_validation(item, issues, &current_path, item_name, parent_scripts);
        }
        
        // Si c'est un folder, récurser avec les scripts du folder
        if let Some(sub_items) = item["item"].as_array() {
            let mut updated_scripts = parent_scripts.to_vec();
            
            // Ajouter les scripts de test du folder actuel
            if let Some(events) = item["event"].as_array() {
                for event in events {
                    if event["listen"] == "test" {
                        if let Some(exec) = event["script"]["exec"].as_array() {
                            let script = exec
                                .iter()
                                .filter_map(|line| line.as_str())
                                .collect::<Vec<&str>>()
                                .join("\n");
                            updated_scripts.push(script);
                        }
                    }
                }
            }
            
            check_items(sub_items, issues, &current_path, &updated_scripts);
        }
    }
}

fn check_request_schema_validation(
    item: &Value,
    issues: &mut Vec<LintIssue>,
    path: &str,
    item_name: &str,
    parent_scripts: &[String],
) {
    // Extraire le script de test
    let test_script = utils::extract_test_scripts(item).join("\n");
    
    // Patterns pour détecter la validation de schéma
    let schema_patterns = vec![
        r"pm\.response\.to\.have\.jsonSchema\s*\(",
        r"jsonSchema",
        r"Schema_Validation",
    ];
    
    // Vérifier dans le script de la requête
    let has_schema_validation = schema_patterns.iter().any(|pattern| {
        if let Ok(re) = Regex::new(pattern) {
            re.is_match(&test_script)
        } else {
            false
        }
    });
    
    // Si pas trouvé, vérifier dans les scripts parents
    let has_schema_in_parents = if !has_schema_validation {
        parent_scripts.iter().any(|parent_script| {
            schema_patterns.iter().any(|pattern| {
                if let Ok(re) = Regex::new(pattern) {
                    re.is_match(parent_script)
                } else {
                    false
                }
            })
        })
    } else {
        false
    };
    
    let is_covered = has_schema_validation || has_schema_in_parents;
    
    // Vérifier si la requête retourne probablement du JSON
    let method = item["request"]["method"].as_str().unwrap_or("");
    let url = if let Some(url_str) = item["request"]["url"].as_str() {
        url_str.to_string()
    } else if let Some(url_obj) = item["request"]["url"].as_object() {
        url_obj.get("raw")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };
    
    let likely_json_response = (method == "GET" || method == "POST") &&
        !url.contains("/download") &&
        !url.contains("/file");
    
    if likely_json_response && !is_covered {
        issues.push(LintIssue {
            rule_id: "test-schema-validation-recommended".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "🛡️ Requête \"{}\" : validation de schéma JSON recommandée pour améliorer la robustesse des tests",
                item_name
            ),
            path: path.to_string(),
            line: None,
            fix: Some(serde_json::json!({
                "type": "add_schema_validation",
                "suggested_code": "// Définir le schéma JSON attendu\nconst schema = {\n    \"type\": \"object\",\n    \"properties\": {\n        // Définir les propriétés attendues\n    },\n    \"required\": []\n};\n\n// Test de validation de schéma\nif (pm.response.code === 200) {\n    pm.test(requestName + \" - Schema_Validation\", () => {\n        pm.response.to.have.jsonSchema(schema);\n    });\n}",
            })),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_with_schema_validation() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "const schema = { type: 'object' };",
                            "pm.test('Schema validation', function() {",
                            "    pm.response.to.have.jsonSchema(schema);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_request_without_schema_validation() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "pm.test('Status is 200', function() {",
                            "    pm.response.to.have.status(200);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("validation de schéma"));
    }

    #[test]
    fn test_folder_with_schema_validation() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Users Folder",
                "item": [{
                    "name": "Get User",
                    "request": {
                        "method": "GET",
                        "url": "https://api.example.com/users/123"
                    },
                    "event": [{
                        "listen": "test",
                        "script": {
                            "exec": ["pm.test('Status OK', function() {});"]
                        }
                    }]
                }],
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "const schema = { type: 'object' };",
                            "pm.response.to.have.jsonSchema(schema);"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // Devrait être OK car le schéma est au niveau folder parent
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_download_endpoint_skipped() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Download File",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/download/file.pdf"
                },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "pm.test('Status is 200', function() {",
                            "    pm.response.to.have.status(200);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // Download endpoint devrait être skip
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_post_request_needs_schema() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Create User",
                "request": {
                    "method": "POST",
                    "url": "https://api.example.com/users"
                },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "pm.test('Status is 201', function() {",
                            "    pm.response.to.have.status(201);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("validation de schéma"));
    }
}
