#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust"

rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

cargo build --release -p lampcord-ffi --target aarch64-apple-ios
cargo build --release -p lampcord-ffi --target aarch64-apple-ios-sim
cargo build --release -p lampcord-ffi --target x86_64-apple-ios

mkdir -p target/ios-device target/ios-sim
cp target/aarch64-apple-ios/release/liblampcord_ffi.a target/ios-device/liblampcord_ffi.a
lipo -create \
  target/aarch64-apple-ios-sim/release/liblampcord_ffi.a \
  target/x86_64-apple-ios/release/liblampcord_ffi.a \
  -output target/ios-sim/liblampcord_ffi.a

echo "iOS static libraries: $ROOT/rust/target/ios-device and $ROOT/rust/target/ios-sim"
