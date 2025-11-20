# @postman-linter/core

High-performance Rust-based linting engine for Postman collections.

## 🦀 About

This is the core linting engine written in Rust for maximum performance. It analyzes Postman collections and applies a comprehensive set of rules to ensure quality and best practices.

## 🔍 Features

- **Fast**: Written in Rust for optimal performance
- **Comprehensive**: 20+ linting rules covering:
  - Request structure and naming
  - Authentication best practices
  - Environment variable usage
  - Documentation completeness
  - Script quality
- **Auto-fix**: Automatic fixing for many issues
- **Scoring**: Quality score calculation (0-100)

## 🛠️ Building

```bash
# Build the Rust library
cargo build --release

# Run tests
cargo test

# Compile to WebAssembly
wasm-pack build --target web
```

## 📦 Usage

This package is typically used through the WASM wrapper (`@postman-linter/linter-wasm`) or the CLI (`@postman-linter/cli`).

## 📄 License

MIT - See [LICENSE](../../LICENSE) for details.
