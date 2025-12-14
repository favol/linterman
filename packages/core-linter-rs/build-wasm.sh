#!/bin/bash
# Script de build WASM pour le linter Postman
# Génère le WASM avec wasm-pack et le copie aux bons emplacements

set -e

echo "🔨 Compilation du WASM avec wasm-pack..."
wasm-pack build --target nodejs --out-dir pkg-node --release

echo "📦 Copie du WASM vers les emplacements requis..."
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/postman_linter_core_bg.wasm
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/wasm/postman_linter_core_bg.wasm
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/wasm/linter_bg.wasm

echo "✅ WASM compilé et copié avec succès !"
