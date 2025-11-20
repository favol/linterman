import type { LintConfig, LintResult } from '@postman-linter/shared-types';
export type { LintConfig, LintResult, LintIssue, LintStats } from '@postman-linter/shared-types';
/**
 * Initialise le module WASM
 * Doit être appelé avant d'utiliser lint()
 */
export declare function initWasm(): Promise<void>;
/**
 * Vérifie si le WASM est initialisé
 */
export declare function isWasmInitialized(): boolean;
/**
 * Analyse une collection Postman avec le linter
 *
 * @param collection - Collection Postman (objet JSON)
 * @param config - Configuration du linter (optionnel)
 * @returns Résultat de l'analyse avec score, issues et stats
 *
 * @example
 * ```typescript
 * import { initWasm, lint } from '@postman-linter/linter-wasm';
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
export declare function lint(collection: unknown, config?: Partial<LintConfig>): Promise<LintResult>;
/**
 * Analyse une collection Postman de manière synchrone (Node.js uniquement)
 *
 * ⚠️ Cette fonction nécessite que initWasm() ait été appelé au préalable
 *
 * @param collection - Collection Postman (objet JSON)
 * @param config - Configuration du linter (optionnel)
 * @returns Résultat de l'analyse
 */
export declare function lintSync(collection: unknown, config?: Partial<LintConfig>): LintResult;
/**
 * Analyse et corrige automatiquement une collection Postman
 *
 * @param collection - Collection Postman (objet JSON)
 * @param config - Configuration du linter (optionnel)
 * @returns Résultat avec collection corrigée et statistiques
 *
 * @example
 * ```typescript
 * import { initWasm, lintAndFix } from '@postman-linter/linter-wasm';
 *
 * await initWasm();
 *
 * const result = await lintAndFix(collection);
 * console.log(`Fixes applied: ${result.fixes_applied}`);
 * console.log(`Score: ${result.before.score}% → ${result.after.score}%`);
 * ```
 */
export declare function lintAndFix(collection: unknown, config?: Partial<LintConfig>): Promise<any>;
/**
 * Version synchrone de lintAndFix (Node.js uniquement)
 */
export declare function lintAndFixSync(collection: unknown, config?: Partial<LintConfig>): any;
/**
 * Obtient la liste des règles disponibles
 */
export declare function getAvailableRules(): string[];
/**
 * Obtient les métadonnées d'une règle
 */
export declare function getRuleMetadata(ruleId: string): {
    id: string;
    name: string;
    category: string;
    severity: string;
    description: string;
} | null;
/**
 * Version du package
 */
export declare const VERSION = "1.0.0";
//# sourceMappingURL=index.d.ts.map