# @postman-linter/cli

Command-line interface for Linterman - A powerful linter for Postman collections.

## 🚀 Installation

```bash
npm install -g @postman-linter/cli
```

## 📖 Usage

```bash
# Lint a Postman collection
postman-lint my-collection.json

# Lint and auto-fix issues
postman-lint my-collection.json --fix

# Output results in JSON format
postman-lint my-collection.json --format json
```

## 🔧 Options

- `--fix` - Automatically fix issues when possible
- `--format <type>` - Output format (text, json, html)
- `--config <path>` - Path to configuration file
- `--help` - Show help

## 📄 License

MIT - See [LICENSE](../../LICENSE) for details.
