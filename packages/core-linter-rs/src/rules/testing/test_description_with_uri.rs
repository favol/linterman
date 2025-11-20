use crate::LintIssue;
use regex::Regex;
use serde_json::Value;

/// Règle : test-description-with-uri
/// 
/// Vérifie que les descriptions de tests incluent des segments du chemin URI
/// pour un meilleur reporting et traçabilité.
/// 
/// Exemples valides :
/// - pm.test("GET /users returns 200", ...)
/// - pm.test("POST /users/123/orders", ...)
/// - pm.test("Test with " + location, ...) // utilise variable location
/// 
/// Sévérité : ERROR (-15%)
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
            // Vérifier si des tests existent dans les folders parents
            let has_tests_in_parent = parent_scripts.iter().any(|script| {
                Regex::new(r"pm\.test\s*\(").unwrap().is_match(script)
            });
            
            if has_tests_in_parent {
                // Skip : les tests au niveau folder ne peuvent pas inclure l'URI spécifique
                continue;
            }
            
            check_request_tests(item, issues, &current_path, item_name);
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

fn check_request_tests(item: &Value, issues: &mut Vec<LintIssue>, path: &str, item_name: &str) {
    // Extraire le script de test
    let test_script = extract_test_script(item);
    if test_script.is_empty() {
        return;
    }
    
    // Extraire le script pre-request pour détecter les variables
    let prerequest_script = extract_prerequest_script(item);
    
    // Extraire l'URI path
    let uri_path = extract_uri_path(item);
    if uri_path == "/unknown" {
        return;
    }
    
    // Extraire les segments du path
    let path_segments: Vec<&str> = uri_path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with(':') && !s.contains('{'))
        .collect();
    
    if path_segments.is_empty() {
        return;
    }
    
    // Détecter les variables de chemin définies
    let path_variables = extract_path_variables(&prerequest_script, &test_script);
    
    // Analyser chaque test pm.test (avec description simple ou concaténation)
    let test_pattern = Regex::new(r#"pm\.test\s*\(\s*([^,]+?)(?:,|\))"#).unwrap();
    
    for caps in test_pattern.captures_iter(&test_script) {
        if let Some(desc_match) = caps.get(1) {
            let raw_description = desc_match.as_str().trim();
            
            // Vérifier si le test utilise une variable de chemin (dans la description brute)
            let uses_path_variable = path_variables.iter().any(|var| {
                raw_description.contains(var)
            }) || raw_description.contains("location")
                || raw_description.contains("requestName");
            
            // Si utilise une variable, c'est valide
            if uses_path_variable {
                continue;
            }
            
            // Extraire la description textuelle (entre guillemets)
            let simple_desc_pattern = Regex::new(r#"["']([^"']+)["']"#).unwrap();
            if let Some(simple_caps) = simple_desc_pattern.captures(raw_description) {
                if let Some(text_match) = simple_caps.get(1) {
                    let test_description = text_match.as_str();
                    let test_desc_lower = test_description.to_lowercase();
                    
                    // Vérifier si au moins un segment du chemin est présent
                    let has_uri_segment = path_segments.iter().any(|segment| {
                        let segment_lower = segment.to_lowercase();
                        test_desc_lower.contains(&segment_lower)
                            || test_desc_lower.contains(&format!("/{}", segment_lower))
                            || test_desc_lower.contains(&format!("[/{}", segment_lower))
                    });
                    
                    if !has_uri_segment {
                        // Créer des suggestions
                        let max_segments = 3.min(path_segments.len());
                        let suggested_segments = &path_segments[path_segments.len() - max_segments..];
                        let suggested_path = format!("/{}", suggested_segments.join("/"));
                        
                        let suggestion = if path_variables.is_empty() {
                            format!(
                                "inclure un segment du chemin (ex: \"{}\") ou utiliser la variable location/requestName",
                                suggested_path
                            )
                        } else {
                            format!(
                                "inclure un segment du chemin (ex: \"{}\") ou utiliser la variable {}",
                                suggested_path,
                                path_variables.join(" ou ")
                            )
                        };
                        
                        // Générer la nouvelle description avec location
                        let new_description = format!("location + ' - {}'", test_description);
                        
                        issues.push(LintIssue {
                            rule_id: "test-description-with-uri".to_string(),
                            severity: "error".to_string(),
                            message: format!(
                                "🎯 Test \"{}\" dans \"{}\" devrait {}",
                                test_description, item_name, suggestion
                            ),
                            path: path.to_string(),
                            line: None,
                            fix: Some(serde_json::json!({
                                "type": "update_test_description",
                                "old_description": test_description,
                                "new_description": new_description,
                            })),
                        });
                    }
                }
            }
        }
    }
}

fn extract_test_script(item: &Value) -> String {
    if let Some(events) = item["event"].as_array() {
        for event in events {
            if event["listen"] == "test" {
                if let Some(exec) = event["script"]["exec"].as_array() {
                    return exec
                        .iter()
                        .filter_map(|line| line.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                }
            }
        }
    }
    String::new()
}

fn extract_prerequest_script(item: &Value) -> String {
    if let Some(events) = item["event"].as_array() {
        for event in events {
            if event["listen"] == "prerequest" {
                if let Some(exec) = event["script"]["exec"].as_array() {
                    return exec
                        .iter()
                        .filter_map(|line| line.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                }
            }
        }
    }
    String::new()
}

fn extract_uri_path(item: &Value) -> String {
    let url = if let Some(url_str) = item["request"]["url"].as_str() {
        url_str.to_string()
    } else if let Some(url_obj) = item["request"]["url"].as_object() {
        url_obj.get("raw")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        return "/unknown".to_string();
    };
    
    // Nettoyer l'URL des variables {{base_url}}
    let clean_url = Regex::new(r"\{\{[^}]+\}\}")
        .unwrap()
        .replace_all(&url, "http://example.com");
    
    // Extraire le path
    if let Ok(parsed_url) = url::Url::parse(&clean_url) {
        let path = parsed_url.path().to_string();
        // Supprimer les query params et fragments
        path.split('?').next()
            .unwrap_or(&path)
            .split('#').next()
            .unwrap_or(&path)
            .to_string()
    } else {
        // Fallback : extraire manuellement
        if let Some(path_match) = Regex::new(r"/[^?#]*").unwrap().find(&url) {
            path_match.as_str().to_string()
        } else {
            "/unknown".to_string()
        }
    }
}

fn extract_path_variables(prerequest_script: &str, test_script: &str) -> Vec<String> {
    let mut variables = Vec::new();
    
    // Patterns pour détecter les variables de chemin
    let patterns = vec![
        r#"pm\.environment\.set\s*\(\s*["']([^"']+)["']\s*,\s*[^)]*(?:path|location|uri|url)"#,
        r#"pm\.variables\.set\s*\(\s*["']([^"']+)["']\s*,\s*[^)]*(?:path|location|uri|url)"#,
        r#"let\s+(\w+)\s*=\s*[^;]*(?:path|location|uri|url)"#,
        r#"const\s+(\w+)\s*=\s*[^;]*(?:path|location|uri|url)"#,
    ];
    
    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(prerequest_script) {
                if let Some(var_match) = caps.get(1) {
                    variables.push(var_match.as_str().to_string());
                }
            }
            for caps in re.captures_iter(test_script) {
                if let Some(var_match) = caps.get(1) {
                    variables.push(var_match.as_str().to_string());
                }
            }
        }
    }
    
    // Dédupliquer
    variables.sort();
    variables.dedup();
    variables
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_test_with_uri_segment() {
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
                            "pm.test('GET /users returns 200', function() {",
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
    fn test_invalid_test_without_uri() {
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
        assert!(issues[0].message.contains("devrait inclure"));
    }

    #[test]
    fn test_valid_test_with_location_variable() {
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
                            "pm.test('Test ' + location, function() {",
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
    fn test_skip_folder_tests() {
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
                            "exec": ["pm.test('Generic test', function() {});"]
                        }
                    }]
                }],
                "event": [{
                    "listen": "test",
                    "script": {
                        "exec": ["pm.test('Folder level test', function() {});"]
                    }
                }]
            }]
        });
        
        let issues = check(&collection);
        // Devrait skip car il y a un test au niveau folder
        assert_eq!(issues.len(), 0);
    }
}
