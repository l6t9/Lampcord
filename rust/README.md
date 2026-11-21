# Lampcord Rust core

This directory contains the Rust core for Lampcord on the `rewrite/rust-core` branch. The goal is to build a stable, UI-agnostic engine that can be integrated with the Kotlin Multiplatform/Compose app via FFI.

The previous GPUI desktop experiment is preserved on `archive/rust-gpui-desktop`.

## Workspace layout

- `crates/lampcord-model` — Discord protocol/data models (`Snowflake`, users, guilds, channels, messages, gateway types).
- `crates/lampcord-core` — Shared session/protocol logic:
  - auth/login, captcha, remote auth, OAuth
  - REST config/client, gateway planning/session/queue, event dispatch
  - `GatewayClient` (identify/resume, heartbeats, READY/RESUMED/dispatch)
  - `StateCache`, read state, token store
- `crates/lampcord-transport` — Concrete transports:
  - `ReqwestRestTransport` (HTTP via reqwest/rustls)
  - `TungsteniteGatewayTransport` (WebSocket via tokio-tungstenite/rustls)
- `crates/lampcord-ffi` — UI-agnostic FFI facade for Kotlin/Swift/desktop bridges (commands, snapshots, events).

## Current focus

1. Harden core APIs and state/cache behavior.
2. Expand `lampcord-ffi` into a real integration slice (auth/session/gateway/messages).
3. Add UniFFI/Kotlin bridge proof-of-concept for JVM desktop.
4. Integrate with Kotlin Multiplatform UI behind feature flags.

## Useful checks

```bash
cd rust
cargo fmt --all -- --check
cargo test -p lampcord-model -p lampcord-core -p lampcord-ffi
cargo check -p lampcord-model -p lampcord-core -p lampcord-transport -p lampcord-ffi
```
