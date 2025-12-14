use crate::LintIssue;
use crate::utils;
use regex::Regex;
use serde_json::Value;

/// Règle : test-response-time-mandatory
/// 
/// Vérifie que chaque requête a un test de temps de réponse.
/// Patterns détectés :
/// - pm.response.responseTime
/// - pm.expect(...responseTime...)
/// - responseTime.to.be.below
/// - "Temps de réponse" / "response time"
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
            check_request_response_time(item, issues, &current_path, item_name, parent_scripts);
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

fn check_request_response_time(
    item: &Value,
    issues: &mut Vec<LintIssue>,
    path: &str,
    item_name: &str,
    parent_scripts: &[String],
) {
    // Extraire le script de test
    let test_script = utils::extract_test_scripts(item).join("\n");
    
    // Patterns pour détecter les tests de temps de réponse
    let response_time_patterns = vec![
        r"responseTime",
        r"response_time",
        r"pm\.response\.responseTime",
        r"pm\.expect\(.*responseTime.*\)",
        r"responseTime.*\.to\.be\.below",
        r"responseTime.*\.to\.be\.lessThan",
        r"(?i)temps de réponse",
        r"(?i)response time",
    ];
    
    // Vérifier dans le script de la requête
    let has_response_time_test = response_time_patterns.iter().any(|pattern| {
        if let Ok(re) = Regex::new(pattern) {
            re.is_match(&test_script)
        } else {
            false
        }
    });
    
    // Si pas trouvé, vérifier dans les scripts parents
    let has_test_in_parents = if !has_response_time_test {
        parent_scripts.iter().any(|parent_script| {
            response_time_patterns.iter().any(|pattern| {
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
    
    if !has_response_time_test && !has_test_in_parents {
        issues.push(LintIssue {
            rule_id: "test-response-time-mandatory".to_string(),
            severity: "warning".to_string(),
            message: format!("⏱️ Request \"{}\" is missing response time test", item_name),
            path: path.to_string(),
            line: None,
            fix: Some(serde_json::json!({
                "type": "add_response_time_test",
                "suggested_code": "pm.test(location + \" - Response time is less than 200ms\", function () {\n    pm.expect(pm.response.responseTime).to.be.below(200);\n});",
            })),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_with_response_time_test() {
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
                            "pm.test('Response time is acceptable', function() {",
                            "    pm.expect(pm.response.responseTime).to.be.below(200);",
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
    fn test_request_without_response_time_test() {
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
        assert!(issues[0].message.contains("is missing response time test"));
    }

    #[test]
    fn test_folder_with_response_time_test() {
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
                            "pm.test('Response time OK', function() {",
                            "    pm.expect(pm.response.responseTime).to.be.below(500);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // Devrait être OK car le test est au niveau folder parent
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_french_response_time_pattern() {
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
                            "pm.test('Temps de réponse acceptable', function() {",
                            "    pm.expect(pm.response.responseTime).to.be.below(200);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 0);
    }
}
