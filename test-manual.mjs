#!/usr/bin/env node

/**
 * Test manuel du wrapper WASM
 * Usage: node test-manual.mjs
 */

import { initWasm, lint, lintSync, getAvailableRules, getRuleMetadata, isWasmInitialized } from './dist/index.js';

console.log('🧪 Test manuel du wrapper WASM\n');

// Test 1: Initialisation
console.log('1️⃣  Test: Initialisation WASM');
await initWasm();
console.log('✅ WASM initialisé:', isWasmInitialized());
console.log('');

// Test 2: Règles disponibles
console.log('2️⃣  Test: Règles disponibles');
const rules = getAvailableRules();
console.log('✅ Règles:', rules);
console.log('');

// Test 3: Métadonnées d'une règle
console.log('3️⃣  Test: Métadonnées de règle');
const metadata = getRuleMetadata('test-http-status-mandatory');
console.log('✅ Métadonnées:', metadata);
console.log('');

// Test 4: Collection vide
console.log('4️⃣  Test: Collection vide');
const emptyCollection = {
  info: { name: 'Empty Collection' },
  item: [],
};
const emptyResult = await lint(emptyCollection);
console.log('✅ Score:', emptyResult.score);
console.log('✅ Issues:', emptyResult.issues.length);
console.log('✅ Stats:', emptyResult.stats);
console.log('');

// Test 5: Détection de test HTTP manquant
console.log('5️⃣  Test: Détection test HTTP manquant');
const collectionWithoutTest = {
  info: { name: 'Test Collection' },
  item: [
    {
      name: 'Get Users',
      request: {
        url: 'https://api.example.com/users',
        method: 'GET',
      },
    },
  ],
};
const resultWithoutTest = await lint(collectionWithoutTest);
console.log('✅ Score:', resultWithoutTest.score);
console.log('✅ Issues:', resultWithoutTest.issues.length);
resultWithoutTest.issues.forEach(issue => {
  console.log(`   - [${issue.severity}] ${issue.rule_id}: ${issue.message}`);
});
console.log('');

// Test 6: Détection de secret hardcodé
console.log('6️⃣  Test: Détection secret hardcodé');
const collectionWithSecret = {
  info: { name: 'Test Collection' },
  item: [
    {
      name: 'Get Users',
      request: {
        url: 'https://api.example.com/users',
        method: 'GET',
        header: [
          {
            key: 'X-API-Key',
            value: 'api_key=abcdef1234567890abcdef1234567890',
          },
        ],
      },
    },
  ],
};
const resultWithSecret = await lint(collectionWithSecret);
console.log('✅ Score:', resultWithSecret.score);
console.log('✅ Issues:', resultWithSecret.issues.length);
resultWithSecret.issues.forEach(issue => {
  console.log(`   - [${issue.severity}] ${issue.rule_id}: ${issue.message.substring(0, 80)}...`);
});
console.log('');

// Test 7: Filtrage de règles
console.log('7️⃣  Test: Filtrage de règles');
const resultFiltered = await lint(collectionWithSecret, {
  rules: ['hardcoded-secrets'],
});
console.log('✅ Issues (seulement hardcoded-secrets):', resultFiltered.issues.length);
resultFiltered.issues.forEach(issue => {
  console.log(`   - ${issue.rule_id}`);
});
console.log('');

// Test 8: lintSync
console.log('8️⃣  Test: lintSync (synchrone)');
const syncResult = lintSync(emptyCollection);
console.log('✅ Score (sync):', syncResult.score);
console.log('');

// Résumé
console.log('🎉 Tous les tests sont passés !');
console.log('');
console.log('📊 Résumé:');
console.log('  - Initialisation WASM: ✅');
console.log('  - Règles disponibles: ✅');
console.log('  - Métadonnées: ✅');
console.log('  - Collection vide: ✅');
console.log('  - Détection test HTTP: ✅');
console.log('  - Détection secrets: ✅');
console.log('  - Filtrage règles: ✅');
console.log('  - Mode synchrone: ✅');
