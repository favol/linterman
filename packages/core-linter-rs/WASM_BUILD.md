# Build WASM - Documentation

## Problème résolu

Le WASM doit être compilé avec `wasm-pack` (pas juste `cargo build`) pour générer les bindings JavaScript nécessaires.

## Emplacements des fichiers WASM

Le fichier WASM doit être copié vers **3 emplacements** :
1. `../linter-wasm/postman_linter_core_bg.wasm`
2. `../linter-wasm/wasm/postman_linter_core_bg.wasm`
3. `../linter-wasm/wasm/linter_bg.wasm`

## Comment compiler le WASM

### En développement local
```bash
cd packages/core-linter-rs
./build-wasm.sh
```

### En production
Le script de déploiement (`deployment/scripts/deploy.sh`) exécute automatiquement `build-wasm.sh`.

## Commandes manuelles (si nécessaire)

```bash
# 1. Compiler avec wasm-pack
cd packages/core-linter-rs
wasm-pack build --target nodejs --out-dir pkg-node --release

# 2. Copier vers tous les emplacements
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/postman_linter_core_bg.wasm
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/wasm/postman_linter_core_bg.wasm
cp pkg-node/postman_linter_core_bg.wasm ../linter-wasm/wasm/linter_bg.wasm

# 3. Redémarrer le backend
cd ../backend
pnpm dev
```

## ⚠️ Important

- **NE PAS** utiliser `cargo build --target wasm32-unknown-unknown` seul
- **TOUJOURS** utiliser `wasm-pack build` pour générer les bindings JavaScript
- **TOUJOURS** copier vers les 3 emplacements
