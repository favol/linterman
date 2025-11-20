use serde_json::Value;

/// Utilitaires pour analyser les collections Postman
/// Inspiré de folderScriptHelpers.js du projet source

/// Extrait les scripts de test d'un item (folder ou request)
pub fn extract_test_scripts(item: &Value) -> Vec<String> {
    let mut scripts = Vec::new();
    
    if let Some(events) = item["event"].as_array() {
        for event in events {
            if event["listen"] == "test" {
                if let Some(exec) = event["script"]["exec"].as_array() {
                    let script = exec
                        .iter()
                        .filter_map(|line| line.as_str())
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    scripts.push(script);
                }
            }
        }
    }
    
    scripts
}

/// Extrait les scripts pre-request d'un item
pub fn extract_prerequest_scripts(item: &Value) -> Vec<String> {
    let mut scripts = Vec::new();
    
    if let Some(events) = item["event"].as_array() {
        for event in events {
            if event["listen"] == "prerequest" {
                if let Some(exec) = event["script"]["exec"].as_array() {
                    let script = exec
                        .iter()
                        .filter_map(|line| line.as_str())
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    scripts.push(script);
                }
            }
        }
    }
    
    scripts
}

/// Collecte tous les scripts hérités depuis les folders parents
/// C'est une fonctionnalité clé du projet source pour éviter les faux positifs
pub fn collect_inherited_scripts(collection: &Value, item_path: &str) -> InheritedScripts {
    let mut test_scripts = Vec::new();
    let mut prerequest_scripts = Vec::new();
    
    // Parser le chemin pour remonter la hiérarchie
    let path_parts: Vec<&str> = item_path.split('/').collect();
    
    // Parcourir la collection pour trouver les folders parents
    let mut current = collection;
    for part in path_parts.iter() {
        if part.starts_with("item[") {
            let index = part
                .trim_start_matches("item[")
                .trim_end_matches(']')
                .parse::<usize>()
                .unwrap_or(0);
            
            if let Some(items) = current["item"].as_array() {
                if let Some(item) = items.get(index) {
                    // Collecter les scripts de ce niveau
                    test_scripts.extend(extract_test_scripts(item));
                    prerequest_scripts.extend(extract_prerequest_scripts(item));
                    
                    current = item;
                }
            }
        }
    }
    
    InheritedScripts {
        test_scripts,
        prerequest_scripts,
    }
}

#[derive(Debug)]
pub struct InheritedScripts {
    pub test_scripts: Vec<String>,
    pub prerequest_scripts: Vec<String>,
}

impl InheritedScripts {
    /// Vérifie si un pattern regex est présent dans les scripts hérités
    pub fn has_pattern(&self, pattern: &regex::Regex) -> bool {
        self.test_scripts.iter().any(|script| pattern.is_match(script))
    }
    
    /// Vérifie si une variable est définie dans les pre-request scripts
    pub fn has_variable(&self, var_name: &str) -> bool {
        let set_pattern = format!(r#"pm\.environment\.set\s*\(\s*['"]{}['"]"#, var_name);
        let regex = regex::Regex::new(&set_pattern).unwrap();
        
        self.prerequest_scripts.iter().any(|script| regex.is_match(script))
    }
}

/// Détecte si une requête est un DELETE avec code 204 (No Content)
/// Ces requêtes n'ont pas besoin de tests de body
pub fn is_delete_with_204(item: &Value) -> bool {
    if let Some(request) = item.get("request") {
        if let Some(method) = request["method"].as_str() {
            if method == "DELETE" {
                // Vérifier si le test attend un 204
                let test_scripts = extract_test_scripts(item);
                for script in test_scripts {
                    if script.contains("204") || script.contains("No Content") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Extrait le nom de la requête
pub fn get_request_name(item: &Value) -> String {
    item["name"].as_str().unwrap_or("unknown").to_string()
}

/// Vérifie si un item est une requête (vs un folder)
pub fn is_request(item: &Value) -> bool {
    item.get("request").is_some()
}

/// Vérifie si un item est un folder
pub fn is_folder(item: &Value) -> bool {
    item.get("request").is_none() && item.get("item").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_test_scripts() {
        let item = json!({
            "name": "Test Request",
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
        });
        
        let scripts = extract_test_scripts(&item);
        assert_eq!(scripts.len(), 1);
        assert!(scripts[0].contains("pm.response.to.have.status(200)"));
    }

    #[test]
    fn test_is_delete_with_204() {
        let item = json!({
            "name": "Delete User",
            "request": {
                "method": "DELETE",
                "url": "https://api.example.com/users/1"
            },
            "event": [{
                "listen": "test",
                "script": {
                    "exec": ["pm.response.to.have.status(204);"]
                }
            }]
        });
        
        assert!(is_delete_with_204(&item));
    }

    #[test]
    fn test_is_request() {
        let request = json!({
            "name": "Get Users",
            "request": { "url": "https://api.example.com" }
        });
        
        let folder = json!({
            "name": "Users",
            "item": []
        });
        
        assert!(is_request(&request));
        assert!(!is_request(&folder));
    }
}
