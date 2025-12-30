# 🔍 Linterman

Open-source linter for Postman collections - Analyze and improve your API collections with 12+ quality rules.

> **🌐 SaaS Platform**: Try the full-featured web app at [linterman.fr](https://linterman.fr) with cloud history, dashboard, and PDF exports.

---

## 📦 Packages

This monorepo contains three open-source packages:

- **[core-linter-rs](./packages/core-linter-rs)** - Rust-based linting engine (12+ rules)
- **[linter-wasm](./packages/linter-wasm)** - WebAssembly wrapper for browser and Node.js
- **[cli](./packages/cli)** - Command-line interface (coming soon to npm)

---

## 🚀 Quick Start

### CLI Usage (Local Build)

```bash
# Clone the repository
git clone https://github.com/favol/linterman.git
cd linterman

# Build the CLI
cd packages/core-linter-rs
cargo build --release --bin postman-linter

# Run analysis
./target/release/postman-linter path/to/collection.json

# With specific rules
./target/release/postman-linter --rules test-http-status-mandatory,hardcoded-secrets collection.json

# From stdin
cat collection.json | ./target/release/postman-linter

# With config file (exported from SaaS)
./target/release/postman-linter --config linterman-rules-config.json collection.json
```

### WASM Usage (Browser/Node.js)

```bash
npm install @linterman/linter-wasm
```

```javascript
import { initWasm, lint } from '@linterman/linter-wasm';

await initWasm();
const result = await lint(collectionJson, { local_only: true });

console.log(`Score: ${result.score}%`);
console.log(`Issues: ${result.issues.length}`);
```

---

## 📋 Available Rules

### 🔴 ERROR Rules (Critical)
- `test-http-status-mandatory` - HTTP status tests required
- `test-description-with-uri` - Test descriptions must include URIs
- `collection-overview-template` - Collection must follow documentation template
- `request-examples-required` - Response examples required
- `documentation-completeness` - Complete documentation required

### ⚠️ WARNING Rules (Recommended)
- `test-response-time-mandatory` - Response time tests recommended
- `test-body-content-validation` - Body content validation recommended
- `request-naming-convention` - Follow naming conventions
- `response-time-threshold` - Response time thresholds
- `environment-variables-usage` - Use environment variables
- `test-coverage-minimum` - Minimum test coverage (80%)
- `hardcoded-secrets` - Detect hardcoded secrets (API keys, tokens, passwords)

---

## 🛠️ CLI Options

```bash
postman-linter [OPTIONS] [COLLECTION_FILE]

Options:
  --config <FILE>    Load rules configuration from JSON file
  --rules <RULES>    Comma-separated list of rule IDs to enable
  --help             Show help message

Examples:
  postman-linter collection.json
  postman-linter --rules test-http-status-mandatory,hardcoded-secrets collection.json
  postman-linter --config linterman-rules-config.json collection.json
  cat collection.json | postman-linter
```

---

## 📊 Output Format

The CLI outputs JSON with the following structure:

```json
{
  "score": 42,
  "issues": [
    {
      "rule_id": "test-http-status-mandatory",
      "severity": "error",
      "message": "Request \"Users List\" is missing HTTP status test",
      "path": "/item[0]/item[0]",
      "line": null,
      "fix": {
        "type": "add_test",
        "suggested_code": "pm.test(\"Status code is 200\", ...)"
      }
    }
  ],
  "stats": {
    "total_requests": 8,
    "total_tests": 6,
    "total_folders": 3,
    "errors": 23,
    "warnings": 25,
    "infos": 0
  }
}
```

---

## 🌐 SaaS Platform

For a complete experience with additional features:

- **Cloud History** - Save and track analysis history
- **Dashboard** - Visualize score evolution and KPIs
- **PDF Export** - Generate professional reports
- **Auto-Fix** - Automatic correction for 5+ rules
- **Custom Templates** - Customize documentation templates
- **Team Collaboration** - Share configurations and results

👉 **[Try it now at linterman.fr](https://linterman.fr)**

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/favol/linterman.git
cd linterman/packages/core-linter-rs
cargo build --release

# Run tests
cargo test
```

---

## 📄 License

MIT - See [LICENSE](./LICENSE) for details.

---

**Made with ❤️ by the Linterman team**

*For issues and feature requests: [GitHub Issues](https://github.com/favol/linterman/issues)*
