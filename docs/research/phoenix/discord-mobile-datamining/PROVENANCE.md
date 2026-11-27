# Discord mobile datamining snapshot

Curated files from `/home/lampu/Downloads/discord-mobile-datamining-main`.

## Source

- Upstream snapshot: `Wumpus-Central/discord-mobile-datamining`
- Commit: `e529e88e24f712b9cb06a66f22c8429756f54fb2`
- Commit message: `356.7 (356207) runtime`
- Commit date: `2026-11-22T03:18:09Z`
- Local full dump: `/home/lampu/Downloads/discord-mobile-datamining-main` (206 MB, 20,217 files)

## Version match

This snapshot matches the research APK exactly:

| Source | Version |
|---|---|
| APK | `com.discord.phoenix`, `versionCode=356207`, `versionName=356.7 - Alpha` |
| Datamining snapshot | commit message `356.7 (356207) runtime` |
| `discord_app/utils/native/ClientInfoUtils.tsx:12-15` | hard-codes `35620700000000` |

APK bundle string-table offsets also tie the snapshot directly to `assets/index.android.bundle`, including:

```text
modules/app_startup/native/NativeAppStartup.tsx
modules/cache/CacheStore.tsx
modules/gateway/GatewaySocket.tsx
modules/gateway/dispatchSocketMessage.tsx
modules/user_settings/phoenix_launch/native/PhoenixRolloutManager.tsx
```

## What these files are

They are **recovered/decompiled Hermes JavaScript/TSX source**, not official Discord internal source. Evidence:

- Real module paths, API names, and log strings survive.
- Local variables/closures are mangled, e.g. `discord_app/index.native.tsx:13-14`.
- Some function bodies are elided into `function() { ... }` / `/* F... */`.
- Some constructs are not valid-looking source, e.g. `discord_app/modules/libdiscore/libdiscoreExperiments.tsx:166-168`.
- There are no `.rs`, `.wasm`, or `.so` files. The Rust/Wasm implementation is only in the APK.

Treat identifiers and control flow as directional. Use this as behavioral evidence, not as code to copy into Lampcord.

## Scope of the preserved copy

This directory contains a curated 40-file subset (~1.1 MB) focused on:

- startup and cache hydration
- per-account database/cache storage
- gateway connect/resume/dispatch
- Phoenix launch/native bridge
- libdiscore/Flux bridge
- native markdown bridge

The full 206 MB tree was intentionally not copied into the workspace.

## Handling

Private educational research copy. Do not redistribute.
