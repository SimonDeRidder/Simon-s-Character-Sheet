#!/bin/sh

RUSTFLAGS="--cfg erase_components" wasm-pack build -m no-install --no-typescript -t web --dev -d wasm --out-name wasm --no-pack
