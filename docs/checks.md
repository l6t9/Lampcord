# Checks

Run these before opening a pull request.

```bash
# Android assemble (also compiles shared/commonMain + androidMain)
./gradlew :androidApp:assembleDebug --no-configuration-cache

# iOS framework compile (macOS only)
./gradlew :shared:compileKotlinIosArm64

# Desktop (Requiem / Rust)
cd rust
cargo check -p requiem-tsx
cargo check -p requiem
cargo test -p requiem-tsx
cargo test -p requiem-core --features sound --lib ui_tsx
cargo build --release -p requiem

# Optional Linux AppImage from the release binary
cd ..
scripts/package_appimage.sh
```

## Notes

- The old Compose/JVM desktop app and `desktopTest` source set have been removed.
- The vendored Requiem tree is not rustfmt-clean under the pinned toolchain, so CI does not
  enforce `cargo fmt`; avoid reformatting unrelated upstream files.
- `native/voice` is still built for Android by `:androidApp`; do not remove it.
- iOS compilation is covered by the macOS CI job.
- There is no ktlint or Android lint gate yet. Do not reformat unrelated files in a
  behavior change.
