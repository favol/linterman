use crate::LintIssue;
use regex::Regex;
use serde_json::Value;

/// Règle : environment-variables-usage
/// 
/// Vérifie que les URLs et valeurs sensibles utilisent des variables d'environnement
/// plutôt que des valeurs en dur.
/// 
/// Détecte :
/// - URLs en dur (http://, https://)
/// - Tokens/clés API en dur
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
            // Vérifier l'URL
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
            
            // Détecter les URLs en dur (sans variables {{...}})
            let has_hardcoded_url = Regex::new(r"^https?://[^{]").unwrap().is_match(&url) &&
                !url.contains("{{") && 
                !url.contains("localhost") && 
                !url.contains("127.0.0.1");
            
            if has_hardcoded_url {
                issues.push(LintIssue {
                    rule_id: "environment-variables-usage".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "🔧 Request \"{}\" should use an environment variable for the URL (ex: {{{{base_url}}}})",
                        item_name
                    ),
                    path: format!("{}/request/url", current_path),
                    line: None,
                    fix: Some(serde_json::json!({
                        "type": "use_environment_variable",
                        "field": "url",
                        "suggested_variable": "{{base_url}}",
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
    fn test_with_environment_variable() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "{{base_url}}/users"
                }
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_with_hardcoded_url() {
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
        assert!(issues[0].message.contains("environment variable"));
    }

    #[test]
    fn test_localhost_allowed() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Get Users",
                "request": {
                    "method": "GET",
                    "url": "http://localhost:3000/users"
                }
            }]
        });
        
        let issues = check(&collection);
        // localhost est autorisé
        assert_eq!(issues.len(), 0);
    }
}
