# Lampcord Rust core

Rust core for Lampcord on `rewrite/rust-core`. Goal: stable, UI-agnostic engine integrated with Kotlin Multiplatform/Compose via FFI.

The GPUI desktop experiment is preserved on `archive/rust-gpui-desktop`.

## Workspace

- `crates/lampcord-model` — Discord protocol/data models.
- `crates/lampcord-core` — Auth/gateway/REST/state/session/cache logic.
- `crates/lampcord-transport` — Reqwest REST + tungstenite gateway transports.
- `crates/lampcord-ffi` — UI-agnostic FFI facade (commands, snapshots, events, UniFFI).

## FFI surface (current)

- **Config**: API base, gateway URL override, user agent, storage path.
- **Commands**: `ConnectToken`, `ConnectStoredToken`, `SelectGuild/Channel`, `FetchMessages`, `SendMessage`, `Logout`, `Shutdown`.
- **State**: pollable `ClientSnapshot` (seq, connection, token, user, guilds/channels/messages, error) + drained `ClientEvent`s.
- **Errors**: `FfiClientError` (UniFFI flat error) for command failures.
- **Persistence**: simple token-store file persistence via `storage_path`.

## Checks

```bash
cd rust
cargo fmt --all -- --check
cargo test -p lampcord-model -p lampcord-core -p lampcord-transport -p lampcord-ffi
cargo build -p lampcord-ffi --features uniffi
```

## Kotlin/JVM PoC

- Generate bindings: `scripts/generate_kotlin_bindings.sh`
- Desktop flag: `-Dlampcord.rustCore=true` (PoC bridge/injection/parity overlay; Kotlin gateway skipped in `RustCoreMode`).
