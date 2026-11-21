# Lampcord Rust rewrite (GPUI)

This directory contains the in-progress Rust rewrite of Lampcord on the `rewrite/rust-gpui` branch.

## Workspace layout

- `crates/lampcord-model` — Discord protocol/data models (`Snowflake`, users, guilds, channels, messages, gateway payload types) and protocol tests.
- `crates/lampcord-core` — Shared session/protocol logic:
  - auth/login request/response and error classification
  - captcha parsing/retry helpers
  - REST config/request/response primitives and `RestClient` (captcha retry hook, unauthorized-token reporting, gateway URL fetch)
  - gateway planning/session/read-state/queue utilities and event dispatcher
  - `GatewayClient` (identify/resume, periodic heartbeats, READY/session tracking, dispatch handling)
  - `StateCache` (guild/channel/message/read-state tracking updated from gateway events)
  - token/account store (`TokenStore`)
- `crates/lampcord-transport` — Concrete transports:
  - `ReqwestRestTransport` (HTTP via reqwest/rustls)
  - `TungsteniteGatewayTransport` (WebSocket via tokio-tungstenite/rustls)
- `apps/lampcord` — GPUI desktop shell (early vertical slice).

## Current status (implemented so far)

### 1) End-to-end vertical slice + reconnect hardening
- `lampcord` starts a backend worker (Tokio runtime on a background thread) and a GPUI window.
- Token/bootstrap:
  - pass `--token <DISCORD_TOKEN>` or `--token=<DISCORD_TOKEN>`, or set `LAMPCORD_TOKEN` / `DISCORD_TOKEN`
  - if no token is provided, the app attempts to use the first saved account token from the local token store
- Connects to Discord via REST (`/gateway`) + WebSocket gateway using `lampcord-core`/`lampcord-transport`.
- Gateway client supports streaming events and periodic heartbeats (`run_stream`), and the backend runs a reconnect supervisor loop with backoff/resume-aware behavior for disconnect/reconnect/invalid-session cases (auth failures stop reconnecting).
- UI shows connection/status and current user (after READY), guilds, channels, and messages for the selected channel.

### 2) App/runtime architecture + interactive UI
- Backend/UI boundary in `apps/lampcord/src/backend.rs` + `main.rs`:
  - commands: connect, select guild/channel, fetch messages (with optional `before`), send message, persist token, logout, shutdown
  - snapshots/events published back to the UI thread; periodic UI refresh notifies GPUI
- Interactive controls:
  - clickable guild and channel selection
  - "Load older" message pagination control (fetches with `before` cursor)
  - basic composer (focus window, type, `Enter` to send, `Esc` to clear, `Shift+Enter` newline)

### 3) REST fetch/send path + pagination basics
- Typed REST helpers in `lampcord-core::api` for fetching and sending channel messages.
- Backend fetches recent channel messages via REST and supports older-page fetch/merge; snapshots merge fetched history with live gateway/cache messages.
- Send path implemented end-to-end (UI composer -> backend command -> REST create message -> local/snapshot update).

### 4) Token persistence/session flow (partial) + workflow/checks
- `TokenStore` load/save to JSON in config dir:
  - `$XDG_CONFIG_HOME/lampcord/accounts.json`, else `~/.config/lampcord/accounts.json`, else `./lampcord-accounts.json`
- On READY, active token/account is persisted (best-effort) and reused on next launch if no explicit token is provided.
- Rust rewrite CI workflow added: `.github/workflows/rust-rewrite.yml`
- Manual smoke checklist added: `rust/SMOKE_TEST.md`
- Basic runtime logging for key gateway events/errors is emitted to stderr (`[lampcord] ...`).

## Running the GPUI app

From the repository root:

```bash
cd rust
cargo run -p lampcord -- --token "<DISCORD_TOKEN>"
```

Or with an environment variable:

```bash
cd rust
LAMPCORD_TOKEN="<DISCORD_TOKEN>" cargo run -p lampcord
```

Notes:
- GPUI requires a working desktop graphics environment (Wayland/X11 on Linux as configured in this workspace).
- Using a user token may violate Discord's Terms of Service; use at your own risk and only with accounts/tokens you are authorized to use.
- This is an early rewrite slice: expect missing features, incomplete error handling, and protocol/UI gaps.

## Known gaps / next work

- Richer text input/composer (IME, cursor/selection, paste, attachments), message rendering/virtualization, and UX polish
- Full reconnect/session restore across app restarts, explicit account switching/logout UI, and secure token storage
- Broader REST + gateway coverage (pagination UX refinements, edits/deletes/reactions, members/presence/typing/threads, read-state acking)
- Improved rate-limit/captcha/auth error handling and diagnostics
- Voice, media/embeds, notifications, settings, and feature parity with the existing client
- More automated integration tests, stricter clippy/fmt enforcement, and packaging/CI expansion

## Useful checks

```bash
cd rust
cargo fmt --all -- --check
cargo check -p lampcord
cargo test -p lampcord-core -p lampcord-model
```

See `rust/SMOKE_TEST.md` for a manual validation checklist.

## Running the GPUI app

From the repository root:

```bash
cd rust
cargo run -p lampcord -- --token "<DISCORD_TOKEN>"
```

Or with an environment variable:

```bash
cd rust
LAMPCORD_TOKEN="<DISCORD_TOKEN>" cargo run -p lampcord
```

Notes:
- GPUI requires a working desktop graphics environment (Wayland/X11 on Linux as configured in this workspace).
- Using a user token may violate Discord's Terms of Service; use at your own risk and only with accounts/tokens you are authorized to use.
- This is an early rewrite slice: expect missing features, incomplete error handling, and protocol/UI gaps.

## Known gaps / next work

- Interactive UI actions (guild/channel selection, message composer/send, account switch/logout)
- Reconnect/backoff loop with session resume across disconnects/restarts
- Broader REST coverage (history pagination, edits/deletes/reactions, profiles/settings, etc.)
- More complete gateway dispatch/cache handling and UI virtualization/rendering
- Voice, media/embeds, notifications, richer formatting, and feature parity with the existing client
- Improved logging/diagnostics, integration tests, and packaging

## Useful checks

```bash
cd rust
cargo check -p lampcord
cargo test -p lampcord-core -p lampcord-model
```
