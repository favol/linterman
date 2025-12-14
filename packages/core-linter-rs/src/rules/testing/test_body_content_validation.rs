use crate::LintIssue;
use crate::utils;
use regex::Regex;
use serde_json::Value;

/// Règle : test-body-content-validation
/// 
/// Vérifie que les tests valident le contenu du body, pas seulement le statut HTTP.
/// Patterns détectés :
/// - pm.response.json()
/// - pm.response.to.have.jsonSchema
/// - .to.have.property()
/// - .to.include() / .to.eql() / .to.equal()
/// 
/// Skip : DELETE, 204 No Content, endpoints sans body
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
            check_request_body_validation(item, issues, &current_path, item_name, parent_scripts);
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

fn check_request_body_validation(
    item: &Value,
    issues: &mut Vec<LintIssue>,
    path: &str,
    item_name: &str,
    parent_scripts: &[String],
) {
    // Extraire le script de test
    let test_script = utils::extract_test_scripts(item).join("\n");
    
    // Vérifier si des tests existent (dans la requête ou les parents)
    let has_test_in_parent = !parent_scripts.is_empty() && 
        parent_scripts.iter().any(|s| !s.trim().is_empty());
    
    // Seulement si des tests existent
    if test_script.is_empty() && !has_test_in_parent {
        return;
    }
    
    // Patterns pour les tests de contenu du body
    let body_patterns = vec![
        r"pm\.response\.json\(\)",
        r"pm\.response\.to\.have\.jsonSchema",
        r"responseJson",
        r"jsonData",
        r"pm\.response\.text\(\)",
        r"\.to\.have\.property\(",
        r"\.to\.include\(",
        r"\.to\.eql\(",
        r"\.to\.equal\(",
        r"\.to\.be\.",
    ];
    
    // Vérifier dans le script de la requête
    let has_body_test = body_patterns.iter().any(|pattern| {
        if let Ok(re) = Regex::new(pattern) {
            re.is_match(&test_script)
        } else {
            false
        }
    });
    
    // Si pas trouvé, vérifier dans les scripts parents
    let has_test_in_parents = if !has_body_test {
        parent_scripts.iter().any(|parent_script| {
            body_patterns.iter().any(|pattern| {
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
    
    // Patterns pour détecter si c'est probablement une requête sans body
    let no_body_patterns = vec![
        r"204",
        r"(?i)no.*content",
        r"(?i)delete",
    ];
    
    let method = item["request"]["method"].as_str().unwrap_or("");
    let probably_no_body = no_body_patterns.iter().any(|pattern| {
        if let Ok(re) = Regex::new(pattern) {
            re.is_match(&test_script) ||
            re.is_match(method) ||
            re.is_match(item_name) ||
            parent_scripts.iter().any(|s| re.is_match(s))
        } else {
            false
        }
    });
    
    // Avertissement seulement si pas de test de body ET probablement pas un endpoint sans body
    if !has_body_test && !has_test_in_parents && !probably_no_body {
        issues.push(LintIssue {
            rule_id: "test-body-content-validation".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "⚠️ Request \"{}\" should validate response content (body, properties, schema)",
                item_name
            ),
            path: path.to_string(),
            line: None,
            fix: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_with_body_validation() {
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
                            "});",
                            "pm.test('Body has users', function() {",
                            "    const jsonData = pm.response.json();",
                            "    pm.expect(jsonData).to.have.property('users');",
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
    fn test_request_without_body_validation() {
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
        assert!(issues[0].message.contains("should validate response content"));
    }

    #[test]
    fn test_delete_request_skipped() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Delete User",
                "request": {
                    "method": "DELETE",
                    "url": "https://api.example.com/users/123"
                },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "pm.test('Status is 204', function() {",
                            "    pm.response.to.have.status(204);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // DELETE devrait être skip
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_204_no_content_skipped() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Update User",
                "request": {
                    "method": "PUT",
                    "url": "https://api.example.com/users/123"
                },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "pm.test('Status is 204 No Content', function() {",
                            "    pm.response.to.have.status(204);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // 204 No Content devrait être skip
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_request_without_tests_skipped() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                }
            }]
        });
        
        let issues = check(&collection);
        // Pas de tests du tout, devrait être skip
        assert_eq!(issues.len(), 0);
    }
}
