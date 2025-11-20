# @postman-linter/linter-wasm

WebAssembly wrapper for the Linterman core engine. Works in both browser and Node.js environments.

## 📦 Installation

```bash
npm install @postman-linter/linter-wasm
```

## 🌐 Usage

### In Node.js

```typescript
import { lintCollection, lintAndFixCollection } from '@postman-linter/linter-wasm';

// Lint a collection
const result = await lintCollection(collectionJson);
console.log(`Score: ${result.score}/100`);
console.log(`Issues found: ${result.issues.length}`);

// Lint and auto-fix
const fixedResult = await lintAndFixCollection(collectionJson);
console.log(`Fixed collection:`, fixedResult.collection);
```

### In Browser

```html
<script type="module">
  import { lintCollection } from '@postman-linter/linter-wasm';
  
  const result = await lintCollection(myCollection);
  console.log('Linting result:', result);
</script>
```

## 🔧 API

### `lintCollection(collection: object): Promise<LintResult>`

Analyzes a Postman collection and returns linting results.

**Returns:**
```typescript
{
  score: number;           // Quality score (0-100)
  issues: Issue[];         // Array of detected issues
  summary: {
    total: number;
    critical: number;
    warning: number;
    info: number;
  }
}
```

### `lintAndFixCollection(collection: object): Promise<FixResult>`

Analyzes and automatically fixes issues when possible.

**Returns:**
```typescript
{
  collection: object;      // Fixed collection
  score: number;
  issues: Issue[];
  fixedCount: number;      // Number of issues fixed
}
```

## 🚀 Performance

The WASM module provides near-native performance for linting operations, making it suitable for:
- Real-time linting in IDEs and editors
- CI/CD pipelines
- Browser-based tools
- Large collection analysis

## 📄 License

MIT - See [LICENSE](../../LICENSE) for details.
