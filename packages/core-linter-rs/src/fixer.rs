use crate::LintIssue;
use serde_json::Value;

/// Module de correction automatique des collections Postman
/// 
/// Ce module applique les corrections suggérées par les règles de linting
/// pour générer une collection corrigée automatiquement.

/// Applique toutes les corrections possibles à une collection
pub fn apply_fixes(collection: &mut Value, issues: &[LintIssue]) -> usize {
    let mut fixes_applied = 0;
    
    for issue in issues {
        if let Some(fix) = &issue.fix {
            if apply_single_fix(collection, &issue.path, fix) {
                fixes_applied += 1;
            }
        }
    }
    
    fixes_applied
}

/// Applique une correction unique
fn apply_single_fix(collection: &mut Value, path: &str, fix: &Value) -> bool {
    let fix_type = fix["type"].as_str().unwrap_or("");
    
    match fix_type {
        "rename_request" => apply_rename_request(collection, path, fix),
        "add_test" | "add_response_time_test" => apply_add_test(collection, path, fix),
        "update_test_description" | "fix_test_description_uri" => apply_update_test_description(collection, path, fix),
        "update_threshold" | "adjust_threshold" => apply_update_threshold(collection, path, fix),
        _ => false,
    }
}

/// Correction : Renommer une requête
fn apply_rename_request(collection: &mut Value, path: &str, fix: &Value) -> bool {
    if let Some(suggested_name) = fix["suggested_name"].as_str() {
        if let Some(item) = get_item_by_path_mut(collection, path) {
            item["name"] = Value::String(suggested_name.to_string());
            return true;
        }
    }
    false
}

/// Correction : Ajouter un test
fn apply_add_test(collection: &mut Value, path: &str, fix: &Value) -> bool {
    let test_code = fix["test_code"].as_str()
        .or_else(|| fix["suggested_code"].as_str());
    
    if let Some(test_code) = test_code {
        if let Some(item) = get_item_by_path_mut(collection, path) {
            // Créer ou récupérer le tableau d'events
            if !item["event"].is_array() {
                item["event"] = Value::Array(vec![]);
            }
            
            let events = item["event"].as_array_mut().unwrap();
            
            // Si le test utilise la variable 'location', ajouter le prerequest
            if test_code.contains("location") {
                let has_prerequest = events.iter().any(|e| e["listen"] == "prerequest");
                if !has_prerequest {
                    events.push(serde_json::json!({
                        "listen": "prerequest",
                        "script": {
                            "exec": [
                                "// Définir la variable location pour les tests",
                                "pm.environment.set('location', pm.request.url.getPath());"
                            ],
                            "type": "text/javascript"
                        }
                    }));
                }
            }
            
            // Chercher un event "test" existant
            let mut test_event_found = false;
            for event in events.iter_mut() {
                if event["listen"] == "test" {
                    // Vérifier si le test existe déjà
                    if let Some(exec) = event["script"]["exec"].as_array_mut() {
                        let test_exists = exec.iter().any(|line| {
                            if let Some(line_str) = line.as_str() {
                                // Vérifier si le test est similaire (même pattern)
                                line_str.contains("Status code") && test_code.contains("Status code")
                                || line_str.contains("responseTime") && test_code.contains("responseTime")
                                || line_str.contains("response time") && test_code.contains("response time")
                            } else {
                                false
                            }
                        });
                        
                        // Ajouter seulement si le test n'existe pas déjà
                        if !test_exists {
                            exec.push(Value::String(test_code.to_string()));
                        }
                    }
                    test_event_found = true;
                    break;
                }
            }
            
            // Si pas d'event "test", en créer un
            if !test_event_found {
                events.push(serde_json::json!({
                    "listen": "test",
                    "script": {
                        "exec": [test_code],
                        "type": "text/javascript"
                    }
                }));
            }
            
            return true;
        }
    }
    false
}

/// Correction : Mettre à jour la description d'un test
fn apply_update_test_description(collection: &mut Value, path: &str, fix: &Value) -> bool {
    if let Some(old_desc) = fix["old_description"].as_str() {
        if let Some(new_desc) = fix["new_description"].as_str() {
            if let Some(item) = get_item_by_path_mut(collection, path) {
                // Si la nouvelle description utilise 'location', ajouter le prerequest
                if new_desc.contains("location") {
                    if !item["event"].is_array() {
                        item["event"] = Value::Array(vec![]);
                    }
                    let events = item["event"].as_array_mut().unwrap();
                    let has_prerequest = events.iter().any(|e| e["listen"] == "prerequest");
                    if !has_prerequest {
                        events.push(serde_json::json!({
                            "listen": "prerequest",
                            "script": {
                                "exec": [
                                    "// Définir la variable location pour les tests",
                                    "pm.environment.set('location', pm.request.url.getPath());"
                                ],
                                "type": "text/javascript"
                            }
                        }));
                    }
                }
                
                if let Some(events) = item["event"].as_array_mut() {
                    for event in events {
                        if event["listen"] == "test" {
                            if let Some(exec) = event["script"]["exec"].as_array_mut() {
                                for line in exec.iter_mut() {
                                    if let Some(line_str) = line.as_str() {
                                        // Remplacer "old_desc" par new_desc dans pm.test()
                                        if line_str.contains(&format!("\"{}\"", old_desc)) || 
                                           line_str.contains(&format!("'{}'", old_desc)) {
                                            let new_line = line_str
                                                .replace(&format!("\"{}\"", old_desc), new_desc)
                                                .replace(&format!("'{}'", old_desc), new_desc);
                                            *line = Value::String(new_line);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return true;
            }
        }
    }
    false
}

/// Correction : Mettre à jour un seuil de temps de réponse
fn apply_update_threshold(collection: &mut Value, path: &str, fix: &Value) -> bool {
    let new_threshold = fix["new_threshold"].as_i64()
        .or_else(|| fix["suggested_threshold"].as_i64());
    
    if let Some(new_threshold) = new_threshold {
        if let Some(item) = get_item_by_path_mut(collection, path) {
            if let Some(events) = item["event"].as_array_mut() {
                for event in events {
                    if event["listen"] == "test" {
                        if let Some(exec) = event["script"]["exec"].as_array_mut() {
                            for line in exec.iter_mut() {
                                if let Some(line_str) = line.as_str() {
                                    // Remplacer les seuils >2000 par 2000
                                    if line_str.contains("responseTime") && line_str.contains("below") {
                                        // Regex pour trouver le nombre
                                        let re = regex::Regex::new(r"\.below\((\d+)\)").unwrap();
                                        if let Some(caps) = re.captures(line_str) {
                                            if let Some(threshold_str) = caps.get(1) {
                                                if let Ok(threshold) = threshold_str.as_str().parse::<i64>() {
                                                    if threshold > 2000 {
                                                        let new_line = line_str.replace(
                                                            &format!(".below({})", threshold),
                                                            &format!(".below({})", new_threshold)
                                                        );
                                                        *line = Value::String(new_line);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return true;
            }
        }
    }
    false
}

/// Récupère un item par son path (mutable)
fn get_item_by_path_mut<'a>(collection: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current = collection;
    
    for part in parts {
        if part.starts_with("item[") && part.ends_with(']') {
            let index_str = &part[5..part.len() - 1];
            if let Ok(index) = index_str.parse::<usize>() {
                if let Some(items) = current["item"].as_array_mut() {
                    if index < items.len() {
                        current = &mut items[index];
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
    
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rename_request() {
        let mut collection = json!({
            "item": [{
                "name": "Users List",
                "request": {
                    "method": "GET"
                }
            }]
        });

        let issues = vec![LintIssue {
            rule_id: "request-naming-convention".to_string(),
            severity: "warning".to_string(),
            message: "Test".to_string(),
            path: "/item[0]".to_string(),
            line: None,
            fix: Some(json!({
                "type": "rename_request",
                "suggested_name": "GET Users List"
            })),
        }];

        let fixes_applied = apply_fixes(&mut collection, &issues);
        
        assert_eq!(fixes_applied, 1);
        assert_eq!(collection["item"][0]["name"], "GET Users List");
    }

    #[test]
    fn test_add_test() {
        let mut collection = json!({
            "item": [{
                "name": "GET Users",
                "request": {
                    "method": "GET"
                }
            }]
        });

        let issues = vec![LintIssue {
            rule_id: "test-http-status-mandatory".to_string(),
            severity: "error".to_string(),
            message: "Test".to_string(),
            path: "/item[0]".to_string(),
            line: None,
            fix: Some(json!({
                "type": "add_test",
                "test_code": "pm.test('Status code is 200', function() { pm.response.to.have.status(200); });"
            })),
        }];

        let fixes_applied = apply_fixes(&mut collection, &issues);
        
        assert_eq!(fixes_applied, 1);
        assert!(collection["item"][0]["event"].is_array());
        assert_eq!(collection["item"][0]["event"][0]["listen"], "test");
    }
}
