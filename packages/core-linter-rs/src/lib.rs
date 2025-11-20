use wasm_bindgen::prelude::*;
pub mod rules;
pub mod utils;
pub mod fixer;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Types
// ============================================================================

#[derive(Deserialize)]
pub struct LintConfig {
    pub local_only: bool,
    pub rules: Option<Vec<String>>,
    pub fix: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LintIssue {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub path: String,
    pub line: Option<u32>,
    pub fix: Option<Value>,
}

#[derive(Serialize, Debug)]
pub struct LintStats {
    pub total_requests: u32,
    pub total_tests: u32,
    pub total_folders: u32,
    pub errors: u32,
    pub warnings: u32,
    pub infos: u32,
}

#[derive(Serialize, Debug)]
pub struct LintResult {
    pub score: u32,
    pub issues: Vec<LintIssue>,
    pub stats: LintStats,
}

// ============================================================================
// Moteur Principal
// ============================================================================

pub fn run_linter(collection: &Value, config: &LintConfig) -> LintResult {
    let mut issues = Vec::new();
    
    // Appliquer les règles
    let enabled_rules = config.rules.as_ref();
    
    // Testing rules
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"test-http-status-mandatory".to_string()) {
        issues.extend(rules::testing::test_http_status_mandatory::check(collection));
    }
    
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"test-description-with-uri".to_string()) {
        issues.extend(rules::testing::test_description_with_uri::check(collection));
    }
    
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"test-response-time-mandatory".to_string()) {
        issues.extend(rules::testing::test_response_time_mandatory::check(collection));
    }
    
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"test-body-content-validation".to_string()) {
        issues.extend(rules::testing::test_body_content_validation::check(collection));
    }
    
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"test-schema-validation-recommended".to_string()) {
        issues.extend(rules::testing::test_schema_validation_recommended::check(collection));
    }
    
    // Structure rules
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"request-naming-convention".to_string()) {
        issues.extend(rules::structure::request_naming_convention::check(collection));
    }
    
    // Performance rules
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"response-time-threshold".to_string()) {
        issues.extend(rules::performance::response_time_threshold::check(collection));
    }
    
    // Best practices rules
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"environment-variables-usage".to_string()) {
        issues.extend(rules::best_practices::environment_variables_usage::check(collection));
    }
    
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"test-coverage-minimum".to_string()) {
        issues.extend(rules::best_practices::test_coverage_minimum::check(collection));
    }
    
    // Documentation rules
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"collection-overview-template".to_string()) {
        issues.extend(rules::documentation::collection_overview_template::check(collection));
    }
    
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"request-examples-required".to_string()) {
        issues.extend(rules::documentation::request_examples_required::check(collection));
    }
    
    // Security rules
    if enabled_rules.is_none() || enabled_rules.unwrap().contains(&"hardcoded-secrets".to_string()) {
        issues.extend(rules::security::hardcoded_secrets::check(collection));
    }
    
    // Calculer les stats
    let stats = calculate_stats(collection, &issues);
    
    // Calculer le score
    let score = calculate_score(&issues, &stats);
    
    LintResult {
        score,
        issues,
        stats,
    }
}

fn calculate_stats(collection: &Value, issues: &[LintIssue]) -> LintStats {
    let total_requests = count_requests(collection);
    let total_tests = count_tests(collection);
    let total_folders = count_folders(collection);
    
    let errors = issues.iter().filter(|i| i.severity == "error").count() as u32;
    let warnings = issues.iter().filter(|i| i.severity == "warning").count() as u32;
    let infos = issues.iter().filter(|i| i.severity == "info").count() as u32;
    
    LintStats {
        total_requests,
        total_tests,
        total_folders,
        errors,
        warnings,
        infos,
    }
}

fn count_requests(value: &Value) -> u32 {
    let mut count = 0;
    if let Some(items) = value["item"].as_array() {
        for item in items {
            if item.get("request").is_some() {
                count += 1;
            }
            count += count_requests(item);
        }
    }
    count
}

fn count_tests(value: &Value) -> u32 {
    let mut count = 0;
    if let Some(events) = value["event"].as_array() {
        for event in events {
            if event["listen"] == "test" {
                count += 1;
            }
        }
    }
    if let Some(items) = value["item"].as_array() {
        for item in items {
            count += count_tests(item);
        }
    }
    count
}

fn count_folders(value: &Value) -> u32 {
    let mut count = 0;
    if let Some(items) = value["item"].as_array() {
        for item in items {
            if item.get("request").is_none() && item.get("item").is_some() {
                count += 1;
            }
            count += count_folders(item);
        }
    }
    count
}

fn calculate_score(issues: &[LintIssue], stats: &LintStats) -> u32 {
    let base_score = 100.0;
    
    // Compter les issues par sévérité
    let errors = issues.iter().filter(|i| i.severity == "error").count() as f64;
    let warnings = issues.iter().filter(|i| i.severity == "warning").count() as f64;
    let infos = issues.iter().filter(|i| i.severity == "info").count() as f64;
    
    // Calculer le score basé sur le pourcentage de requêtes avec des problèmes
    // Au lieu de pénaliser par nombre absolu, on pénalise par ratio
    let total_requests = stats.total_requests.max(1) as f64; // Éviter division par zéro
    
    // Pourcentage de requêtes affectées par chaque type de problème
    let error_ratio = (errors / total_requests).min(1.0); // Max 100%
    let warning_ratio = (warnings / total_requests).min(1.0);
    let info_ratio = (infos / total_requests).min(1.0);
    
    // Pénalités basées sur le ratio (pas le nombre absolu)
    // Si 100% des requêtes ont une erreur = -15%
    // Si 50% des requêtes ont une erreur = -7.5%
    let error_penalty = error_ratio * 15.0;
    let warning_penalty = warning_ratio * 8.0;
    let info_penalty = info_ratio * 3.0;
    
    let mut score = base_score - error_penalty - warning_penalty - info_penalty;
    
    // Bonus: +5% si 0 erreurs ET ≤2 warnings (comme dans le projet source)
    if errors == 0.0 && warnings <= 2.0 {
        score += 5.0;
    }
    
    // Limiter entre 0 et 100
    score.max(0.0).min(100.0) as u32
}

// ============================================================================
// WASM Bindings
// ============================================================================

#[wasm_bindgen]
pub fn lint(collection_json: &str, config_json: &str) -> Result<String, JsValue> {
    let collection: Value = serde_json::from_str(collection_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse collection: {}", e)))?;
    
    let config: LintConfig = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;
    
    let result = run_linter(&collection, &config);
    
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Applique les corrections automatiques et retourne la collection corrigée + le nombre de fixes appliqués
#[wasm_bindgen]
pub fn lint_and_fix(collection_json: &str, config_json: &str) -> Result<String, JsValue> {
    let mut collection: Value = serde_json::from_str(collection_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse collection: {}", e)))?;
    
    let config: LintConfig = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;
    
    // 1. Lancer le linter pour obtenir les issues
    let result = run_linter(&collection, &config);
    
    // 2. Appliquer les corrections
    let fixes_applied = fixer::apply_fixes(&mut collection, &result.issues);
    
    // 3. Re-lancer le linter sur la collection corrigée
    let new_result = run_linter(&collection, &config);
    
    // 4. Retourner la collection corrigée + les stats
    let response = serde_json::json!({
        "fixed_collection": collection,
        "fixes_applied": fixes_applied,
        "before": {
            "score": result.score,
            "issues": result.issues.len(),
        },
        "after": {
            "score": new_result.score,
            "issues": new_result.issues.len(),
        },
        "remaining_issues": new_result.issues,
    });
    
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_collection() {
        let collection = serde_json::json!({
            "info": { "name": "Test" },
            "item": []
        });
        let config = LintConfig {
            local_only: true,
            rules: Some(vec![]), // Désactiver toutes les règles pour ce test
            fix: None,
        };
        let result = run_linter(&collection, &config);
        assert_eq!(result.score, 100);
    }
}
