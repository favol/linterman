use crate::LintIssue;
use regex::Regex;
use serde_json::Value;

/// Règle : collection-overview-template
/// 
/// Vérifie que l'Overview de la collection respecte le template fixe.
/// Template en dur (paramétrable plus tard) :
/// - Sections requises : Prérequis, Présentation, Mode d'emploi, Reste à faire
/// - Métadonnées requises : Référent, Version de collection
/// 
/// Sévérité : ERROR (-15%)
pub fn check(collection: &Value) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    
    let description = collection["info"]["description"]
        .as_str()
        .unwrap_or("");
    
    // Vérifier les sections obligatoires
    let required_sections = vec![
        ("Prérequis", vec!["prérequis", "prerequis", "requirements", "pré-requis"]),
        ("Présentation", vec!["présentation", "presentation", "description", "overview"]),
        ("Mode d'emploi", vec!["mode d'emploi", "mode d emploi", "utilisation", "usage", "how to use", "instructions"]),
        ("Reste à faire", vec!["reste à faire", "todo", "à faire", "remaining", "next steps"]),
    ];
    
    for (section_name, patterns) in required_sections {
        let has_section = patterns.iter().any(|pattern| {
            description.to_lowercase().contains(&pattern.to_lowercase())
        });
        
        if !has_section {
            issues.push(LintIssue {
                rule_id: "collection-overview-template".to_string(),
                severity: "error".to_string(),
                message: format!("❌ Section de documentation manquante : \"{}\"", section_name),
                path: "/info/description".to_string(),
                line: None,
                fix: None,
            });
        }
    }
    
    // Extraire les métadonnées
    let metadata = extract_collection_metadata(description);
    
    // Vérifier la présence des colonnes dans la documentation
    let has_referent_column = Regex::new(r"(?i)référent").unwrap().is_match(description) &&
        (Regex::new(r"(?i)\|.*référent.*\|").unwrap().is_match(description) ||
         Regex::new(r"(?i)référent\s*:").unwrap().is_match(description));
    
    let has_version_column = Regex::new(r"(?i)version.*collection").unwrap().is_match(description) &&
        (Regex::new(r"(?i)\|.*version.*collection.*\|").unwrap().is_match(description) ||
         Regex::new(r"(?i)version.*collection\s*:").unwrap().is_match(description));
    
    if !has_referent_column {
        issues.push(LintIssue {
            rule_id: "collection-documentation-structure".to_string(),
            severity: "error".to_string(),
            message: "👤 Tableau de documentation manquant : colonne \"Référent\" non présente".to_string(),
            path: "/info/description".to_string(),
            line: None,
            fix: None,
        });
    } else if metadata.referent.is_none() {
        issues.push(LintIssue {
            rule_id: "collection-documentation-structure".to_string(),
            severity: "error".to_string(),
            message: "👤 Référent manquant : la colonne \"Référent\" est présente mais vide".to_string(),
            path: "/info/description".to_string(),
            line: None,
            fix: None,
        });
    }
    
    if !has_version_column {
        issues.push(LintIssue {
            rule_id: "collection-documentation-structure".to_string(),
            severity: "error".to_string(),
            message: "🔢 Tableau de documentation manquant : colonne \"Version de collection\" non présente".to_string(),
            path: "/info/description".to_string(),
            line: None,
            fix: None,
        });
    } else if metadata.collection_version.is_none() {
        issues.push(LintIssue {
            rule_id: "collection-documentation-structure".to_string(),
            severity: "error".to_string(),
            message: "🔢 Version de collection manquante : la colonne \"Version de collection\" est présente mais vide".to_string(),
            path: "/info/description".to_string(),
            line: None,
            fix: None,
        });
    }
    
    // Vérifier la longueur minimale
    if description.len() < 100 {
        issues.push(LintIssue {
            rule_id: "collection-documentation-structure".to_string(),
            severity: "error".to_string(),
            message: "📝 Description de collection trop courte (minimum 100 caractères requis)".to_string(),
            path: "/info/description".to_string(),
            line: None,
            fix: None,
        });
    }
    
    issues
}

#[derive(Debug)]
struct CollectionMetadata {
    collection_version: Option<String>,
    referent: Option<String>,
    gitlab_collection_link: Option<String>,
    gitlab_newman_report_link: Option<String>,
}

/// Extrait les métadonnées de la documentation
fn extract_collection_metadata(description: &str) -> CollectionMetadata {
    let mut metadata = CollectionMetadata {
        collection_version: None,
        referent: None,
        gitlab_collection_link: None,
        gitlab_newman_report_link: None,
    };
    
    // D'abord, essayer d'extraire depuis un tableau Markdown
    extract_from_table(description, &mut metadata);
    
    // Si pas trouvé, essayer avec des patterns regex simples
    if metadata.collection_version.is_none() {
        let version_patterns = vec![
            r"(?i)version.*collection\s*:?\s*([v]?\d+\.\d+\.\d+)",
            r"(?i)version\s+de\s+collection\s*:?\s*([v]?\d+\.\d+\.\d+)",
            r"(?i)collection\s+version\s*:?\s*([v]?\d+\.\d+\.\d+)",
        ];
        
        for pattern in version_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(description) {
                    if let Some(version) = caps.get(1) {
                        let mut v = version.as_str().trim().to_string();
                        if !v.starts_with('v') {
                            v = format!("v{}", v);
                        }
                        metadata.collection_version = Some(v);
                        break;
                    }
                }
            }
        }
    }
    
    if metadata.referent.is_none() {
        let referent_patterns = vec![
            r"(?i)référent\s*:?\s*([^\n\r\|*]+)",
            r"(?i)referent\s*:?\s*([^\n\r\|*]+)",
            r"(?i)contact\s*:?\s*([^\n\r\|*]+)",
            r"(?i)responsable\s*:?\s*([^\n\r\|*]+)",
        ];
        
        for pattern in referent_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(description) {
                    if let Some(referent) = caps.get(1) {
                        let r = referent.as_str()
                            .trim()
                            .replace('|', "")
                            .replace('*', "")
                            .trim()
                            .to_string();
                        
                        if !r.is_empty() && !Regex::new(r"^[\*\-\s]*$").unwrap().is_match(&r) {
                            metadata.referent = Some(r);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // Extraire les liens Gitlab
    if let Ok(re) = Regex::new(r"(?i)\[Collection[^\]]*\]\((https?://[^\)]+)\)") {
        if let Some(caps) = re.captures(description) {
            if let Some(url) = caps.get(1) {
                let u = url.as_str().trim();
                if !u.to_lowercase().contains("null") {
                    metadata.gitlab_collection_link = Some(u.to_string());
                }
            }
        }
    }
    
    if let Ok(re) = Regex::new(r"(?i)\[Rapport\s+Newman[^\]]*\]\((https?://[^\)]+)\)") {
        if let Some(caps) = re.captures(description) {
            if let Some(url) = caps.get(1) {
                let u = url.as_str().trim();
                if !u.to_lowercase().contains("null") {
                    metadata.gitlab_newman_report_link = Some(u.to_string());
                }
            }
        }
    }
    
    metadata
}

/// Extrait les métadonnées depuis un tableau Markdown
fn extract_from_table(description: &str, metadata: &mut CollectionMetadata) {
    let lines: Vec<&str> = description.lines().collect();
    let mut in_table = false;
    let mut headers: Vec<String> = Vec::new();
    let mut header_indices: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for (_i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Détecter le début d'un tableau
        if trimmed.contains('|') && !in_table {
            headers = trimmed
                .split('|')
                .map(|h| h.trim().replace('*', "").to_lowercase())
                .filter(|h| !h.is_empty())
                .collect();
            
            // Créer un index des colonnes
            for (idx, header) in headers.iter().enumerate() {
                header_indices.insert(header.clone(), idx);
            }
            
            in_table = true;
            continue;
        }
        
        // Ignorer la ligne de séparation
        if in_table && trimmed.starts_with('|') && trimmed.contains("---") {
            continue;
        }
        
        // Parser les lignes de données
        if in_table && trimmed.contains('|') {
            let values: Vec<String> = trimmed
                .split('|')
                .map(|v| v.trim().replace('*', "").to_string())
                .filter(|v| !v.is_empty())
                .collect();
            
            // Si on a 2 colonnes (clé/valeur), traiter différemment
            if headers.len() == 2 && values.len() == 2 {
                let key = values[0].trim().to_lowercase();
                let val = values[1].trim();
                
                if val.is_empty() || val == "---" {
                    continue;
                }
                
                // Extraire version
                if key.contains("version") && key.contains("collection") {
                    let mut v = val.to_string();
                    if !v.starts_with('v') && v.chars().next().unwrap_or(' ').is_numeric() {
                        v = format!("v{}", v);
                    }
                    metadata.collection_version = Some(v);
                }
                
                // Extraire référent
                if key.contains("référent") || key.contains("referent") {
                    metadata.referent = Some(val.to_string());
                }
            } else {
                // Format classique : headers en première ligne, valeurs en lignes suivantes
                for (j, value) in values.iter().enumerate() {
                    if j >= headers.len() {
                        break;
                    }
                    
                    let header = &headers[j];
                    let val = value.trim();
                    
                    if val.is_empty() || val == "---" {
                        continue;
                    }
                    
                    // Extraire version
                    if header.contains("version") && header.contains("collection") {
                        let mut v = val.to_string();
                        if !v.starts_with('v') && v.chars().next().unwrap_or(' ').is_numeric() {
                            v = format!("v{}", v);
                        }
                        metadata.collection_version = Some(v);
                    }
                    
                    // Extraire référent
                    if header.contains("référent") || header.contains("referent") {
                        metadata.referent = Some(val.to_string());
                    }
                }
            }
        }
        
        // Sortir du tableau si ligne vide
        if in_table && trimmed.is_empty() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_perfect_collection() {
        let collection = json!({
            "info": {
                "name": "Perfect Collection",
                "description": r#"Collection parfaite sans aucun défaut pour tester le score 100%

## Prérequis
Aucun prérequis spécifique pour cette collection de démonstration.

## Présentation
Cette collection démontre une API REST parfaitement documentée et testée selon toutes les bonnes pratiques.

## Mode d'emploi
1. Configurer les variables d'environnement (base_url, etc.)
2. Exécuter les requêtes dans l'ordre
3. Vérifier que tous les tests passent

## Reste à faire
Aucune amélioration nécessaire - collection parfaite !

| Métadonnée | Valeur |
|------------|--------|
| Référent | John Doe |
| Version de collection | 2.0.0 |
| Statut | Production Ready |"#
            }
        });

        let issues = check(&collection);
        
        // Debug: afficher les issues
        for issue in &issues {
            println!("Issue: {}", issue.message);
        }
        
        assert_eq!(issues.len(), 0, "Should have no issues for perfect collection");
    }

    #[test]
    fn test_complete_documentation() {
        let collection = json!({
            "info": {
                "name": "Test Collection",
                "description": r#"
# Présentation
Cette collection teste l'API avec une description suffisamment longue pour passer la validation de 100 caractères minimum.

## Prérequis
- Node.js
- Postman

## Mode d'emploi
1. Importer la collection
2. Lancer les tests

## Reste à faire
- Ajouter plus de tests

| Métadonnée | Valeur |
|------------|--------|
| Référent | John Doe |
| Version de collection | v1.0.0 |
                "#
            }
        });
        
        let issues = check(&collection);
        // Devrait avoir 0 issues si tout est correct
        for issue in &issues {
            println!("Issue: {}", issue.message);
        }
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_missing_sections() {
        let collection = json!({
            "info": {
                "name": "Test Collection",
                "description": "Description courte sans sections requises"
            }
        });
        
        let issues = check(&collection);
        assert!(issues.len() > 0);
        // Vérifier qu'au moins une section manquante est détectée
        let has_missing_section = issues.iter().any(|i| 
            i.message.contains("Section de documentation manquante")
        );
        assert!(has_missing_section, "Should detect missing sections");
    }

    #[test]
    fn test_missing_metadata() {
        let collection = json!({
            "info": {
                "name": "Test Collection",
                "description": r#"
# Présentation
Test

## Prérequis
Test

## Mode d'emploi
Test

## Reste à faire
Test

Description longue de plus de 100 caractères pour passer la validation de longueur minimale.
                "#
            }
        });
        
        let issues = check(&collection);
        assert!(issues.iter().any(|i| i.message.contains("Référent")));
        assert!(issues.iter().any(|i| i.message.contains("Version")));
    }
}
