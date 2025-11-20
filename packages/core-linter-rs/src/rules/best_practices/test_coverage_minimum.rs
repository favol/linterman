use crate::LintIssue;
use crate::utils;
use serde_json::Value;

/// Règle : test-coverage-minimum
/// 
/// Vérifie qu'un minimum de requêtes ont des tests.
/// Recommandation : Au moins 80% des requêtes devraient avoir des tests.
/// 
/// Sévérité : WARNING (-8%)
pub fn check(collection: &Value) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    
    let (total_requests, requests_with_tests) = count_test_coverage(collection);
    
    if total_requests > 0 {
        let coverage_percent = (requests_with_tests as f32 / total_requests as f32) * 100.0;
        
        if coverage_percent < 80.0 {
            issues.push(LintIssue {
                rule_id: "test-coverage-minimum".to_string(),
                severity: "warning".to_string(),
                message: format!(
                    "📊 Couverture de tests insuffisante : {:.1}% ({}/{} requêtes testées). Minimum recommandé : 80%",
                    coverage_percent, requests_with_tests, total_requests
                ),
                path: "/".to_string(),
                line: None,
                fix: None,
            });
        }
    }
    
    issues
}

fn count_test_coverage(collection: &Value) -> (usize, usize) {
    let mut total = 0;
    let mut with_tests = 0;
    
    if let Some(items) = collection["item"].as_array() {
        count_items(items, &mut total, &mut with_tests);
    }
    
    (total, with_tests)
}

fn count_items(items: &[Value], total: &mut usize, with_tests: &mut usize) {
    for item in items {
        // Si c'est une requête
        if item.get("request").is_some() {
            *total += 1;
            
            let test_scripts = utils::extract_test_scripts(item);
            if !test_scripts.is_empty() && test_scripts.iter().any(|s| !s.trim().is_empty()) {
                *with_tests += 1;
            }
        }
        
        // Si c'est un folder, récurser
        if let Some(sub_items) = item["item"].as_array() {
            count_items(sub_items, total, with_tests);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_good_coverage() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [
                {
                    "name": "Request 1",
                    "request": { "method": "GET", "url": "https://api.example.com/1" },
                    "event": [{ "listen": "test", "script": { "exec": ["pm.test('test', () => {});"] } }]
                },
                {
                    "name": "Request 2",
                    "request": { "method": "GET", "url": "https://api.example.com/2" },
                    "event": [{ "listen": "test", "script": { "exec": ["pm.test('test', () => {});"] } }]
                },
                {
                    "name": "Request 3",
                    "request": { "method": "GET", "url": "https://api.example.com/3" },
                    "event": [{ "listen": "test", "script": { "exec": ["pm.test('test', () => {});"] } }]
                },
                {
                    "name": "Request 4",
                    "request": { "method": "GET", "url": "https://api.example.com/4" },
                    "event": [{ "listen": "test", "script": { "exec": ["pm.test('test', () => {});"] } }]
                }
            ]
        });
        
        let issues = check(&collection);
        // 100% coverage
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_low_coverage() {
        let collection = json!({
            "info": { "name": "Test" },
            "item": [
                {
                    "name": "Request 1",
                    "request": { "method": "GET", "url": "https://api.example.com/1" },
                    "event": [{ "listen": "test", "script": { "exec": ["pm.test('test', () => {});"] } }]
                },
                {
                    "name": "Request 2",
                    "request": { "method": "GET", "url": "https://api.example.com/2" }
                },
                {
                    "name": "Request 3",
                    "request": { "method": "GET", "url": "https://api.example.com/3" }
                },
                {
                    "name": "Request 4",
                    "request": { "method": "GET", "url": "https://api.example.com/4" }
                },
                {
                    "name": "Request 5",
                    "request": { "method": "GET", "url": "https://api.example.com/5" }
                }
            ]
        });
        
        let issues = check(&collection);
        // 20% coverage (1/5)
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Couverture de tests insuffisante"));
    }
}
