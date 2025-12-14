use crate::LintIssue;
use crate::utils;
use regex::Regex;
use serde_json::Value;

/// Règle : response-time-threshold
/// 
/// Vérifie que les tests de temps de réponse utilisent des seuils raisonnables.
/// Seuils recommandés :
/// - < 200ms : Excellent
/// - < 500ms : Bon
/// - < 1000ms : Acceptable
/// - > 2000ms : Trop élevé (WARNING)
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
            let test_script = utils::extract_test_scripts(item).join("\n");
            
            // Détecter les seuils de temps de réponse trop élevés (> 2000ms)
            let threshold_pattern = Regex::new(r"responseTime.*\.to\.be\.below\((\d+)\)").unwrap();
            
            for caps in threshold_pattern.captures_iter(&test_script) {
                if let Some(threshold_match) = caps.get(1) {
                    if let Ok(threshold) = threshold_match.as_str().parse::<u32>() {
                        if threshold > 2000 {
                            issues.push(LintIssue {
                                rule_id: "response-time-threshold".to_string(),
                                severity: "warning".to_string(),
                                message: format!(
                                    "⏱️ Request \"{}\" has response time threshold too high ({}ms > 2000ms recommended)",
                                    item_name, threshold
                                ),
                                path: current_path.clone(),
                                line: None,
                                fix: Some(serde_json::json!({
                                    "type": "adjust_threshold",
                                    "current_threshold": threshold,
                                    "suggested_threshold": 2000,
                                })),
                            });
                        }
                    }
                }
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
    fn test_acceptable_threshold() {
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
                            "pm.test('Response time', function() {",
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
    fn test_high_threshold() {
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
                            "pm.test('Response time', function() {",
                            "    pm.expect(pm.response.responseTime).to.be.below(5000);",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("threshold too high"));
    }
}
