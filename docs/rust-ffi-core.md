# Rust Core + FFI Plan (Option A)

Status: draft  
Date: 2026-11-21  
Scope: Keep Kotlin/Compose UI, move shared Lampcord logic to a reusable Rust core exposed via FFI.

## Goal

Deliver Kotlin/Compose parity on mobile and desktop by reusing the existing Compose UI while replacing shared business/protocol logic with Rust.

The result is a **hybrid app**:

- **UI**: Kotlin/Compose Multiplatform (Android/iOS/Desktop UI shells as needed)
- **Core**: Rust (`lampcord-model`, `lampcord-core`, `lampcord-transport`, new FFI facade)
- **Bridge**: generated bindings (UniFFI target) + thin Kotlin/Swift platform adapters

## Why this direction

- The Kotlin app already has broad UI/feature surface in `shared/src/commonMain`.
- The Rust rewrite already has core protocol pieces (gateway, REST, state cache, auth/session primitives) but GPUI desktop UI is not a mobile path.
- Option A preserves mobile UI investment while converging protocol/state logic in Rust.

## Non-goals (initially)

- Rewriting Compose UI in Rust/GPUI for mobile.
- Exposing every internal Rust struct over FFI.
- Big-bang replacement of all Kotlin shared stores at once.
- Full voice/media parity in the first FFI milestone.

## Workspace layout

Existing:

- `rust/crates/lampcord-model` - protocol/data models
- `rust/crates/lampcord-core` - auth/gateway/rest/state/session logic
- `rust/crates/lampcord-transport` - reqwest/tungstenite transports
- `rust/apps/lampcord` - GPUI desktop app (native Rust consumer)

Planned:

- `rust/crates/lampcord-ffi` - UI-agnostic FFI facade; stable API for Kotlin/Swift/desktop JVM bindings
- `shared/rust-bridge` or equivalent KMP module - Kotlin expect/actual wrappers, Flow adapters, lifecycle management
- platform packaging glue - Android AAR/JNI libs, iOS XCFramework, desktop JVM native libs

## Architecture

```text
Compose Multiplatform UI
        │
        │ Kotlin/Swift bindings
        ▼
lampcord-ffi (facade, FFI-safe DTOs, commands, events, snapshots)
        │
        ▼
lampcord-core (gateway/rest/auth/state/cache/session)
        │
        ├── lampcord-model
        └── lampcord-transport
```

Native Rust consumers (GPUI app) may call `lampcord-core` directly or through the same facade.

## FFI boundary rules

1. **No UI framework types** cross FFI (no GPUI, Compose, UIKit, Android types).
2. **No internal locks/channels/raw serde_json** in public FFI API unless explicitly wrapped.
3. **FFI DTOs are versioned and explicit**. Internal Rust models can evolve behind adapters.
4. **Rust owns async runtime**. Kotlin/Swift receives callbacks/event streams/snapshots.
5. **Platform capabilities are injected** through traits/callbacks: secure storage, file picker/open, notifications, audio devices where needed.
6. **Errors are structured**: stable error code/kind + message + optional details.

## Facade API shape (initial)

The first facade should support a desktop/mobile-proof vertical slice:

- Client lifecycle: create/start/shutdown
- Auth: token connect, password login, MFA submit, remote auth start/stop, logout
- Navigation/state: select guild/channel, snapshot/query current state
- Messages: fetch recent/older, send/edit/delete/reply as available
- Events: subscribe to state changes / gateway events / errors

Illustrative Rust facade (not final):

```rust
pub struct LampcordClient { /* runtime + core state */ }

pub struct ClientConfig {
    pub api_base: String,
    pub gateway_url: Option<String>,
    pub user_agent: Option<String>,
    pub storage_path: Option<String>,
}

pub enum ClientCommand {
    ConnectToken { token: String },
    SelectGuild { guild_id: Option<u64> },
    SelectChannel { channel_id: Option<u64> },
    FetchMessages { channel_id: u64, before: Option<u64> },
    SendMessage { channel_id: u64, content: String },
    Logout,
    Shutdown,
}

pub struct ClientSnapshot { /* FFI-safe snapshot */ }
pub enum ClientEvent { /* snapshot updated, auth required, error, ... */ }
```

UniFFI bindings should expose idiomatic Kotlin/Swift wrappers over this facade.

## Async and event model

- Rust creates and owns a Tokio runtime in `lampcord-ffi`.
- UI calls are non-blocking commands or short queries.
- Core emits events through:
  - callback interface for UniFFI initial implementation, and/or
  - internal queue polled by bindings, wrapped as Kotlin `Flow` / Swift `AsyncSequence`.
- Snapshots should be **coalesced** to avoid flooding UI on bursty gateway dispatches.

## Data mapping strategy

- Use `u64`/string IDs at FFI boundary for Snowflakes (clear, easy across Kotlin/Swift).
- Convert internal `lampcord_model` types to FFI DTOs in facade layer.
- Keep JSON/debug payloads out of normal UI path; allow optional raw event debug API behind feature flag.
- Large collections: paginate and expose incremental updates where possible (messages, member lists).

## Token and secure storage

Current Rust app stores accounts in JSON config. That is acceptable for desktop dev but not sufficient for mobile parity.

Plan:

- Define a `SecretStore`/`AccountStore` trait in core/ffi.
- Desktop implementation: config file or OS keychain-backed store.
- Android: Keystore/EncryptedSharedPreferences-backed adapter via Kotlin.
- iOS: Keychain-backed adapter via Swift.
- FFI client accepts platform store callbacks/handles during initialization.

## Migration phases

### Phase 0 - Stabilize current Rust slice
- Land/split uncommitted WIP (member list, gateway/UI updates).
- Add tests for gateway dispatch/member list/state cache behavior.
- Keep GPUI app working as native reference consumer.

### Phase 1 - FFI vertical slice
- Add `lampcord-ffi` crate with minimal command/snapshot/event API.
- Implement token connect, guild/channel/message flow, logout.
- Generate Kotlin bindings for JVM desktop first; prove Compose can consume it.

### Phase 2 - Mobile packaging
- Android: cargo-ndk + Gradle integration, AAR/native lib packaging.
- iOS: XCFramework build, Swift bindings, KMP integration.
- CI jobs for bindgen and target builds.

### Phase 3 - Replace Kotlin shared internals
- Auth/session/account store
- Gateway manager + state cache
- REST API calls used by Compose UI
- Read state/presence/member list/typing
- Notifications/media/voice (later, larger)

Each slice keeps Compose UI mostly unchanged by adapting to existing Kotlin view models/state where practical.

## Immediate implementation tasks

- [ ] Add `rust/crates/lampcord-ffi` facade crate to workspace.
- [ ] Define FFI-safe snapshot/event/command DTOs.
- [ ] Wrap existing `lampcord-core` gateway/rest/state/token store behind facade.
- [ ] Add desktop JVM binding prototype (UniFFI).
- [ ] Add a small Compose desktop proof-of-concept consuming Rust core behind feature flag.
- [ ] Document platform storage and lifecycle contracts.

## Risks

- **FFI churn**: mitigate with narrow facade and DTO adapters.
- **Async lifecycle bugs**: explicit shutdown, weak callback refs, runtime ownership tests.
- **Build/packaging complexity**: automate bindgen and platform builds in CI early.
- **Parity gap**: migrate by vertical slices, not full replacement.
- **Mobile background/network behavior**: platform adapters for lifecycle/connectivity/push.
