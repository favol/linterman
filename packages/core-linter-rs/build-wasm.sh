#!/bin/bash
# Script de build WASM pour le linter Postman
# Génère le WASM avec wasm-pack et le copie aux bons emplacements

set -e

echo "🔨 Compilation du WASM pour Node.js (Backend/CLI)..."
wasm-pack build --target nodejs --out-dir pkg-node --release

echo "🔨 Compilation du WASM pour le Web (Frontend)..."
wasm-pack build --target web --out-dir pkg-web --release

echo "📦 Copie du WASM Node.js vers les emplacements requis..."
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/postman_linter_core_bg.wasm
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/wasm/postman_linter_core_bg.wasm
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/wasm/linter_bg.wasm

echo "📦 Préparation du package Web..."
mkdir -p ../frontend/public/wasm
cp pkg-web/postman_linter_core_bg.wasm ../frontend/public/wasm/
cp pkg-web/postman_linter_core.js ../frontend/public/wasm/

echo "✅ WASM compilé (Node + Web) et copié avec succès !"
