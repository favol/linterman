import type { LintConfig, LintResult } from '@linterman/shared-types';
import { LintConfigSchema, LintResultSchema } from '@linterman/shared-types';

// ============================================================================
// Types
// ============================================================================

export type { LintConfig, LintResult, LintIssue, LintStats } from '@linterman/shared-types';

interface WasmModule {
  lint(collection_json: string, config_json: string): string;
  lint_and_fix(collection_json: string, config_json: string): string;
}

// ============================================================================
// WASM Loader
// ============================================================================

let wasmModule: WasmModule | null = null;
let wasmInitPromise: Promise<WasmModule> | null = null;

/**
 * Initialise le module WASM
 * Doit être appelé avant d'utiliser lint()
 */
export async function initWasm(): Promise<void> {
  if (wasmModule) {
    return; // Déjà initialisé
  }

  if (wasmInitPromise) {
    await wasmInitPromise; // Initialisation en cours
    return;
  }

  wasmInitPromise = (async () => {
    try {
      // Import dynamique du module WASM
      // wasm-pack génère un module qui s'initialise automatiquement
      const wasm = await import('../wasm/postman_linter_core.js');
      
      // Le module est déjà initialisé par wasm-bindgen
      return wasm as unknown as WasmModule;
    } catch (error) {
      wasmInitPromise = null;
      throw new Error(`Failed to initialize WASM module: ${error}`);
    }
  })();

  wasmModule = await wasmInitPromise;
}

/**
 * Vérifie si le WASM est initialisé
 */
export function isWasmInitialized(): boolean {
  return wasmModule !== null;
}

// ============================================================================
// API Principale
// ============================================================================

/**
 * Analyse une collection Postman avec le linter
 * 
 * @param collection - Collection Postman (objet JSON)
 * @param config - Configuration du linter (optionnel)
 * @returns Résultat de l'analyse avec score, issues et stats
 * 
 * @example
 * ```typescript
 * import { initWasm, lint } from '@linterman/linter-wasm';
 * 
 * await initWasm();
 * 
 * const collection = {
 *   info: { name: "My API" },
 *   item: [...]
 * };
 * 
 * const result = await lint(collection, {
 *   local_only: true,
 *   rules: ['test-http-status-mandatory', 'hardcoded-secrets']
 * });
 * 
 * console.log(`Score: ${result.score}/100`);
 * console.log(`Issues: ${result.issues.length}`);
 * ```
 */
export async function lint(
  collection: unknown,
  config: Partial<LintConfig> = {}
): Promise<LintResult> {
  // Initialiser WASM si nécessaire
  if (!wasmModule) {
    await initWasm();
  }

  if (!wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }

  // Valider et normaliser la config
  const validatedConfig = LintConfigSchema.parse({
    local_only: true,
    ...config,
  });

  try {
    // Convertir en JSON strings
    const collectionJson = JSON.stringify(collection);
    const configJson = JSON.stringify(validatedConfig);

    // Appeler le WASM
    const resultJson = wasmModule.lint(collectionJson, configJson);

    // Parser et valider le résultat
    const result = JSON.parse(resultJson);
    return LintResultSchema.parse(result);
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Linting failed: ${error.message}`);
    }
    throw error;
  }
}

/**
 * Analyse une collection Postman de manière synchrone (Node.js uniquement)
 * 
 * ⚠️ Cette fonction nécessite que initWasm() ait été appelé au préalable
 * 
 * @param collection - Collection Postman (objet JSON)
 * @param config - Configuration du linter (optionnel)
 * @returns Résultat de l'analyse
 */
export function lintSync(
  collection: unknown,
  config: Partial<LintConfig> = {}
): LintResult {
  if (!wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }

  const validatedConfig = LintConfigSchema.parse({
    local_only: true,
    ...config,
  });

  try {
    const collectionJson = JSON.stringify(collection);
    const configJson = JSON.stringify(validatedConfig);
    const resultJson = wasmModule.lint(collectionJson, configJson);
    const result = JSON.parse(resultJson);
    return LintResultSchema.parse(result);
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Linting failed: ${error.message}`);
    }
    throw error;
  }
}

/**
 * Analyse et corrige automatiquement une collection Postman
 * 
 * @param collection - Collection Postman (objet JSON)
 * @param config - Configuration du linter (optionnel)
 * @returns Résultat avec collection corrigée et statistiques
 * 
 * @example
 * ```typescript
 * import { initWasm, lintAndFix } from '@linterman/linter-wasm';
 * 
 * await initWasm();
 * 
 * const result = await lintAndFix(collection);
 * console.log(`Fixes applied: ${result.fixes_applied}`);
 * console.log(`Score: ${result.before.score}% → ${result.after.score}%`);
 * ```
 */
export async function lintAndFix(
  collection: unknown,
  config: Partial<LintConfig> = {}
): Promise<any> {
  // Initialiser WASM si nécessaire
  if (!wasmModule) {
    await initWasm();
  }

  if (!wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }

  const validatedConfig = LintConfigSchema.parse({
    local_only: true,
    fix: true,
    ...config,
  });

  try {
    const collectionJson = JSON.stringify(collection);
    const configJson = JSON.stringify(validatedConfig);
    const resultJson = wasmModule.lint_and_fix(collectionJson, configJson);
    return JSON.parse(resultJson);
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Auto-fix failed: ${error.message}`);
    }
    throw error;
  }
}

/**
 * Version synchrone de lintAndFix (Node.js uniquement)
 */
export function lintAndFixSync(
  collection: unknown,
  config: Partial<LintConfig> = {}
): any {
  if (!wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }

  const validatedConfig = LintConfigSchema.parse({
    local_only: true,
    fix: true,
    ...config,
  });

  try {
    const collectionJson = JSON.stringify(collection);
    const configJson = JSON.stringify(validatedConfig);
    const resultJson = wasmModule.lint_and_fix(collectionJson, configJson);
    return JSON.parse(resultJson);
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Auto-fix failed: ${error.message}`);
    }
    throw error;
  }
}

// ============================================================================
// Helpers
// ============================================================================

/**
 * Obtient la liste des règles disponibles
 */
export function getAvailableRules(): string[] {
  return [
    'test-http-status-mandatory',
    'hardcoded-secrets',
  ];
}

/**
 * Obtient les métadonnées d'une règle
 */
export function getRuleMetadata(ruleId: string): {
  id: string;
  name: string;
  category: string;
  severity: string;
  description: string;
} | null {
  const rules: Record<string, any> = {
    'test-http-status-mandatory': {
      id: 'test-http-status-mandatory',
      name: 'Test HTTP Status Mandatory',
      category: 'testing',
      severity: 'error',
      description: 'Vérifie que chaque requête teste le code de statut HTTP',
    },
    'hardcoded-secrets': {
      id: 'hardcoded-secrets',
      name: 'Hardcoded Secrets',
      category: 'security',
      severity: 'error',
      description: 'Détecte les secrets hardcodés (API keys, tokens, passwords)',
    },
  };

  return rules[ruleId] || null;
}

/**
 * Version du package
 */
export const VERSION = '1.0.0';
