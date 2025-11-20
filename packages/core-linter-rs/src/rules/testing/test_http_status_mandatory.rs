use crate::LintIssue;
use regex::Regex;
use serde_json::Value;

/// Règle : test-http-status-mandatory
/// 
/// Vérifie que CHAQUE requête teste le code de statut HTTP.
/// 
/// Patterns acceptés :
/// - pm.response.to.have.status(200)
/// - pm.expect(pm.response.code).to.equal(200)
/// - pm.response.code === 200
/// - responseCode.code === 200
pub fn check(collection: &Value) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    
    // Patterns regex pour détecter les tests de statut HTTP
    let status_patterns = vec![
        r"pm\.response\.to\.have\.status\(",
        r"pm\.response\.to\.be\.success",
        r"pm\.expect\(pm\.response\.code\)",
        r"pm\.response\.code\s*===",
        r"responseCode\.code\s*===",
    ];
    
    let combined_pattern = status_patterns.join("|");
    let regex = Regex::new(&combined_pattern).unwrap();
    
    if let Some(items) = collection["item"].as_array() {
        check_items(items, &regex, &mut issues, "");
    }
    
    issues
}

fn check_items(items: &[Value], regex: &Regex, issues: &mut Vec<LintIssue>, parent_path: &str) {
    for (index, item) in items.iter().enumerate() {
        let item_name = item["name"].as_str().unwrap_or("unknown");
        let current_path = if parent_path.is_empty() {
            format!("/item[{}]", index)
        } else {
            format!("{}/item[{}]", parent_path, index)
        };
        
        // Si c'est une requête
        if item.get("request").is_some() {
            let has_status_test = check_request_for_status_test(item, regex);
            
            if !has_status_test {
                // Générer le code de test à ajouter avec la variable location
                let test_code = "pm.test(location + ' - Status code is 2xx', function() {\n    pm.response.to.be.success;\n});".to_string();
                
                issues.push(LintIssue {
                    rule_id: "test-http-status-mandatory".to_string(),
                    severity: "error".to_string(),
                    message: format!("La requête '{}' ne teste pas le code de statut HTTP", item_name),
                    path: current_path.clone(),
                    line: None,
                    fix: Some(serde_json::json!({
                        "type": "add_test",
                        "test_code": test_code,
                    })),
                });
            }
        }
        
        // Récursion pour les sous-dossiers
        if let Some(sub_items) = item["item"].as_array() {
            check_items(sub_items, regex, issues, &current_path);
        }
    }
}

fn check_request_for_status_test(item: &Value, regex: &Regex) -> bool {
    // Extraire le script de test
    if let Some(events) = item["event"].as_array() {
        for event in events {
            if event["listen"] == "test" {
                if let Some(script) = event["script"]["exec"].as_array() {
                    let test_script = script
                        .iter()
                        .filter_map(|line| line.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                    
                    if regex.is_match(&test_script) {
                        return true;
                    }
                }
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_with_status_test() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Valid Request",
                "request": { "url": "https://api.example.com" },
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
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_request_without_status_test() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [{
                "name": "Invalid Request",
                "request": { "url": "https://api.example.com" },
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": [
                            "pm.test('Body check', function() {",
                            "    pm.expect(pm.response.json()).to.be.an('object');",
                            "});"
                        ]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "test-http-status-mandatory");
        assert_eq!(issues[0].severity, "error");
    }
}
