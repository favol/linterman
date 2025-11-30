# @linterman/linter-wasm

Wrapper TypeScript pour le moteur de linting Rust/WASM de collections Postman.

## Installation

```bash
npm install @linterman/linter-wasm
# ou
pnpm add @linterman/linter-wasm
```

## Usage

### Node.js

```typescript
import { initWasm, lint } from '@linterman/linter-wasm';

// Initialiser le WASM (une seule fois)
await initWasm();

// Analyser une collection
const collection = {
  info: { name: "My API" },
  item: [
    {
      name: "Get Users",
      request: {
        url: "https://api.example.com/users",
        method: "GET"
      }
    }
  ]
};

const result = await lint(collection, {
  local_only: true,
  rules: ['test-http-status-mandatory', 'hardcoded-secrets']
});

console.log(`Score: ${result.score}/100`);
console.log(`Errors: ${result.stats.errors}`);
console.log(`Issues found: ${result.issues.length}`);

result.issues.forEach(issue => {
  console.log(`[${issue.severity}] ${issue.message}`);
});
```

### Browser

```typescript
import { initWasm, lint } from '@linterman/linter-wasm';

async function analyzeCollection(collection: any) {
  // Initialiser le WASM
  await initWasm();
  
  // Analyser
  const result = await lint(collection);
  
  return result;
}
```

### Nuxt 3 / Vue 3

```vue
<script setup lang="ts">
import { initWasm, lint } from '@linterman/linter-wasm';
import { ref } from 'vue';

const result = ref(null);
const loading = ref(false);

async function analyzeCollection(collection: any) {
  loading.value = true;
  
  try {
    await initWasm();
    result.value = await lint(collection);
  } catch (error) {
    console.error('Linting failed:', error);
  } finally {
    loading.value = false;
  }
}
</script>
```

## API

### `initWasm(): Promise<void>`

Initialise le module WASM. Doit être appelé avant d'utiliser `lint()`.

### `lint(collection, config?): Promise<LintResult>`

Analyse une collection Postman.

**Paramètres:**
- `collection`: Collection Postman (objet JSON)
- `config` (optionnel): Configuration du linter
  - `local_only`: boolean (défaut: true)
  - `rules`: string[] (optionnel, toutes les règles par défaut)
  - `fix`: boolean (défaut: false)

**Retour:**
```typescript
{
  score: number,        // Score 0-100
  issues: LintIssue[],  // Liste des problèmes détectés
  stats: {
    total_requests: number,
    total_tests: number,
    total_folders: number,
    errors: number,
    warnings: number,
    infos: number
  }
}
```

### `lintSync(collection, config?): LintResult`

Version synchrone (Node.js uniquement). Nécessite que `initWasm()` ait été appelé.

### `getAvailableRules(): string[]`

Retourne la liste des règles disponibles.

### `getRuleMetadata(ruleId): RuleMetadata | null`

Retourne les métadonnées d'une règle.

### `isWasmInitialized(): boolean`

Vérifie si le WASM est initialisé.

## Règles Disponibles

### Testing
- **test-http-status-mandatory** (error): Vérifie que chaque requête teste le code de statut HTTP

### Security
- **hardcoded-secrets** (error): Détecte les secrets hardcodés (API keys, tokens, passwords)

## Types

Le package exporte tous les types de `@linterman/shared-types`:

```typescript
import type { 
  LintConfig, 
  LintResult, 
  LintIssue, 
  LintStats 
} from '@linterman/linter-wasm';
```

## Performance

- **Taille WASM**: ~966KB (non optimisé)
- **Temps de chargement**: ~100ms
- **Temps d'analyse**: <10ms pour une collection de 50 requêtes

## Développement & tests locaux (monorepo)

Ce package est développé dans le monorepo `lintermanSAAS` et consomme le moteur Rust/WASM déjà packagé.

### Prérequis

Depuis la racine du monorepo :

```bash
pnpm install
```

### Build du package

Depuis la racine :

```bash
pnpm build --filter @linterman/linter-wasm
```

ou directement dans le package :

```bash
cd packages/linter-wasm
pnpm build            # tsc → dist/
```

Les artefacts WASM (`wasm/`) et `dist/` sont commités pour faciliter la CI/CD.

### Tests Vitest (local uniquement)

Depuis la racine du monorepo :

```bash
pnpm wasm:test
```

ou directement dans le package :

```bash
cd packages/linter-wasm
pnpm test             # vitest run
```

Ces tests "smoke" valident :

- l'initialisation du module WASM (`initWasm`),
- un appel simple à `lint` sur une collection vide,
- un appel simple à `lintSync`.

La configuration des tests se trouve dans `vitest.config.ts` (environnement `node`, fichiers `tests-vitest/**/*.vitest.ts`).

Ils sont pensés pour le développement local et **ne sont pas exécutés dans la CI GitHub** actuelle.

## License

MIT
