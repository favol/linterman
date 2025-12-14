use crate::LintIssue;
use serde_json::Value;

/// Règle : request-examples-required
/// 
/// Vérifie la présence et la qualité des exemples de réponse pour chaque requête :
/// - Présence d'au moins un exemple de réponse
/// - Qualité des exemples (nom, contenu du body)
/// - Documentation des paramètres de query
/// 
/// Sévérité : ERROR (-15%)
pub fn check(collection: &Value) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    
    if let Some(items) = collection["item"].as_array() {
        check_items(items, &mut issues, "");
    }
    
    issues
}

fn check_items(items: &[Value], issues: &mut Vec<LintIssue>, parent_path: &str) {
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
            check_request_documentation(item, issues, &current_path, item_name);
        }
        
        // Si c'est un folder, récurser
        if let Some(sub_items) = item["item"].as_array() {
            check_items(sub_items, issues, &current_path);
        }
    }
}

fn check_request_documentation(item: &Value, issues: &mut Vec<LintIssue>, path: &str, item_name: &str) {
    // 1. Vérifier les exemples de réponse
    let responses = item["response"].as_array();
    
    if responses.is_none() || responses.unwrap().is_empty() {
        issues.push(LintIssue {
            rule_id: "request-examples-required".to_string(),
            severity: "error".to_string(),
            message: format!("📋 Request \"{}\" has no response examples", item_name),
            path: path.to_string(),
            line: None,
            fix: None,
        });
    } else {
        // Vérifier la qualité des exemples existants
        for (resp_index, response) in responses.unwrap().iter().enumerate() {
            // Vérifier le nom de l'exemple
            if response["name"].as_str().is_none() || response["name"].as_str().unwrap().is_empty() {
                issues.push(LintIssue {
                    rule_id: "documentation-completeness".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "🏷️ Example #{} for \"{}\" is missing name",
                        resp_index + 1,
                        item_name
                    ),
                    path: format!("{}/response[{}]", path, resp_index),
                    line: None,
                    fix: None,
                });
            }
            
            // Vérifier le contenu (sauf pour 204 No Content)
            let is_204_no_content = response["code"].as_u64() == Some(204)
                || response["status"].as_str() == Some("No Content")
                || response["name"]
                    .as_str()
                    .map(|n| n.to_lowercase().contains("no content"))
                    .unwrap_or(false);
            
            let has_body = response["body"].as_str().is_some() 
                && !response["body"].as_str().unwrap().is_empty();
            
            if !has_body && !is_204_no_content {
                issues.push(LintIssue {
                    rule_id: "documentation-completeness".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "📄 Example #{} for \"{}\" is missing content",
                        resp_index + 1,
                        item_name
                    ),
                    path: format!("{}/response[{}]", path, resp_index),
                    line: None,
                    fix: None,
                });
            }
        }
    }
    
    // 2. Vérifier la documentation des paramètres de query
    if let Some(query_params) = item["request"]["url"]["query"].as_array() {
        let mut undocumented_params = Vec::new();
        
        for param in query_params {
            let param_key = param["key"].as_str().unwrap_or("paramètre sans nom");
            let param_desc = param["description"].as_str().unwrap_or("");
            
            if param_desc.trim().is_empty() {
                undocumented_params.push(param_key.to_string());
            }
        }
        
        if !undocumented_params.is_empty() {
            issues.push(LintIssue {
                rule_id: "documentation-completeness".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "📝 Request \"{}\" has undocumented parameters: {}",
                    item_name,
                    undocumented_params.join(", ")
                ),
                path: format!("{}/request/url/query", path),
                line: None,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_complete_documentation() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": {
                        "raw": "https://api.example.com/users?limit=10",
                        "query": [{
                            "key": "limit",
                            "value": "10",
                            "description": "Number of users to return"
                        }]
                    }
                },
                "response": [{
                    "name": "Success Response",
                    "code": 200,
                    "status": "OK",
                    "body": "{\"users\": []}"
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_missing_response_examples() {
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
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("has no response examples"));
    }

    #[test]
    fn test_example_without_name() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                },
                "response": [{
                    "code": 200,
                    "body": "{\"users\": []}"
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("is missing name"));
    }

    #[test]
    fn test_example_without_body() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                },
                "response": [{
                    "name": "Success",
                    "code": 200
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("is missing content"));
    }

    #[test]
    fn test_204_no_content_is_valid() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Delete User",
                "request": {
                    "method": "DELETE",
                    "url": "https://api.example.com/users/123"
                },
                "response": [{
                    "name": "No Content",
                    "code": 204,
                    "status": "No Content"
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_undocumented_query_params() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": {
                        "raw": "https://api.example.com/users?limit=10&offset=0",
                        "query": [
                            {
                                "key": "limit",
                                "value": "10",
                                "description": "Number of users"
                            },
                            {
                                "key": "offset",
                                "value": "0"
                            }
                        ]
                    }
                },
                "response": [{
                    "name": "Success",
                    "code": 200,
                    "body": "{}"
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("has undocumented parameters"));
        assert!(issues[0].message.contains("offset"));
    }
}
