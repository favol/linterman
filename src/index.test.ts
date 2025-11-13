import { describe, it, expect, beforeAll } from 'vitest';
import { initWasm, lint, lintSync, getAvailableRules, getRuleMetadata, isWasmInitialized } from './index';

describe('linter-wasm', () => {
  beforeAll(async () => {
    await initWasm();
  });

  it('should initialize WASM', () => {
    expect(isWasmInitialized()).toBe(true);
  });

  it('should return available rules', () => {
    const rules = getAvailableRules();
    expect(rules).toContain('test-http-status-mandatory');
    expect(rules).toContain('hardcoded-secrets');
  });

  it('should return rule metadata', () => {
    const metadata = getRuleMetadata('test-http-status-mandatory');
    expect(metadata).toBeDefined();
    expect(metadata?.id).toBe('test-http-status-mandatory');
    expect(metadata?.category).toBe('testing');
    expect(metadata?.severity).toBe('error');
  });

  it('should lint an empty collection', async () => {
    const collection = {
      info: { name: 'Test Collection' },
      item: [],
    };

    const result = await lint(collection);
    
    expect(result).toBeDefined();
    expect(result.score).toBe(100);
    expect(result.issues).toHaveLength(0);
    expect(result.stats.total_requests).toBe(0);
  });

  it('should detect missing HTTP status test', async () => {
    const collection = {
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

    const result = await lint(collection);
    
    expect(result.score).toBeLessThan(100);
    expect(result.issues.length).toBeGreaterThan(0);
    expect(result.issues[0].rule_id).toBe('test-http-status-mandatory');
    expect(result.issues[0].severity).toBe('error');
    expect(result.stats.errors).toBeGreaterThan(0);
  });

  it('should detect hardcoded API key', async () => {
    const collection = {
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

    const result = await lint(collection);
    
    const secretIssue = result.issues.find(i => i.rule_id === 'hardcoded-secrets');
    expect(secretIssue).toBeDefined();
    expect(secretIssue?.severity).toBe('error');
  });

  it('should work with lintSync', () => {
    const collection = {
      info: { name: 'Test Collection' },
      item: [],
    };

    const result = lintSync(collection);
    
    expect(result).toBeDefined();
    expect(result.score).toBe(100);
  });

  it('should filter rules when specified', async () => {
    const collection = {
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

    // Seulement la règle hardcoded-secrets
    const result = await lint(collection, {
      rules: ['hardcoded-secrets'],
    });
    
    // Ne devrait détecter que les secrets, pas le test HTTP manquant
    const hasSecretIssue = result.issues.some(i => i.rule_id === 'hardcoded-secrets');
    const hasHttpTestIssue = result.issues.some(i => i.rule_id === 'test-http-status-mandatory');
    
    expect(hasSecretIssue).toBe(true);
    expect(hasHttpTestIssue).toBe(false);
  });
});
