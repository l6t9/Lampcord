#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust"
cargo build -p lampcord-ffi --features uniffi
cargo run -p lampcord-ffi --features bindgen -- generate --library target/debug/liblampcord_ffi.so --language kotlin --out-dir ../shared/src/desktopMain/kotlin
