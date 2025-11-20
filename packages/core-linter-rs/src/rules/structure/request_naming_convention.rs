use crate::LintIssue;
use regex::Regex;
use serde_json::Value;

/// Règle : request-naming-convention
/// 
/// Vérifie que les noms de requêtes suivent la convention : [METHOD] Description
/// Exemples valides :
/// - "GET Users List"
/// - "POST Create User"
/// - "DELETE Remove Item"
/// 
/// Sévérité : WARNING (-8%)
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
            let method = item["request"]["method"].as_str().unwrap_or("");
            
            // Vérifier si le nom commence par la méthode HTTP
            let naming_pattern = Regex::new(r"^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+").unwrap();
            
            if !naming_pattern.is_match(item_name) && !method.is_empty() {
                issues.push(LintIssue {
                    rule_id: "request-naming-convention".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "📝 Requête \"{}\" : le nom devrait commencer par la méthode HTTP (ex: \"{} {}\")",
                        item_name, method, item_name
                    ),
                    path: current_path.clone(),
                    line: None,
                    fix: Some(serde_json::json!({
                        "type": "rename_request",
                        "suggested_name": format!("{} {}", method, item_name),
                    })),
                });
            }
        }
        
        // Si c'est un folder, récurser
        if let Some(sub_items) = item["item"].as_array() {
            check_items(sub_items, issues, &current_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_naming_convention() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "GET Users List",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                }
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_invalid_naming_convention() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Users List",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/users"
                }
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("devrait commencer par la méthode HTTP"));
    }

    #[test]
    fn test_folder_not_checked() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Users Folder",
                "item": [{
                    "name": "GET User Details",
                    "request": {
                        "method": "GET",
                        "url": "https://api.example.com/users/123"
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // Folder name n'est pas vérifié, seulement les requêtes
        assert_eq!(issues.len(), 0);
    }
}
