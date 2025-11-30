import { describe, it, expect, beforeAll } from 'vitest';
import { initWasm, lint, lintSync, isWasmInitialized } from '../src/index';

describe('linter-wasm (tests de base)', () => {
	beforeAll(async () => {
		await initWasm();
	});

	it('initialise correctement le module WASM', () => {
		expect(isWasmInitialized()).toBe(true);
	});

	it('lint retourne un résultat cohérent pour une collection vide', async () => {
		const collection = {
			info: { name: 'Test Collection' },
			item: [],
		};

		const result = await lint(collection);
		
		expect(result).toBeDefined();
		expect(typeof result.score).toBe('number');
		expect(Array.isArray(result.issues)).toBe(true);
	});

	it('lintSync fonctionne sur une collection vide', () => {
		const collection = {
			info: { name: 'Test Collection' },
			item: [],
		};

		const result = lintSync(collection);
		
		expect(result).toBeDefined();
		expect(typeof result.score).toBe('number');
		expect(Array.isArray(result.issues ?? [])).toBe(true);
	});
});
