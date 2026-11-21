# Rust vs Kotlin Multiplatform for Lampcord - Honest Review

Status: draft  
Date: 2026-11-21  
Context: Lampcord is currently a Compose Multiplatform app (Android/Desktop/iOS targets) with a Rust/GPUI rewrite in progress on `rewrite/rust-gpui`. The user asked for an honest assessment of what Rust can do better than KMP, especially for performance, and how to reach that in the rewrite.

## Executive summary

**Rust is a better long-term fit for Lampcord's core engine and performance-critical desktop client, but not a free win.** 

- **Rust wins** when you need predictable performance, low memory overhead, native systems/media integration, small packaging, and a single-language core that can be shared across platforms via FFI.
- **Kotlin Multiplatform wins** when you need fast UI iteration, mature mobile/desktop UI tooling, a large ecosystem, and the ability to ship features quickly with existing Compose code.
- **The best path for Lampcord** is the one already chosen: **Option A** - keep Kotlin/Compose UI where it is strong (especially mobile), move shared core/protocol/state/media logic to Rust, and use GPUI as a native Rust desktop frontend where it makes sense.

The Rust rewrite will only become "better than KMP" if we intentionally exploit Rust's advantages: zero-copy/low-allocation data flow, explicit state ownership, background work isolation, native media pipelines, careful UI virtualization, and measured performance budgets. A naive Rust port can still be slow.

## Where Rust can be better

### 1) Predictable performance and memory control

**Rust advantage**
- No JVM/GC pauses on desktop. Memory lifetimes and allocations are explicit.
- Smaller idle overhead is possible: no JVM heap, no JIT warmup, fewer runtime services.
- Better control over hot paths: message parsing, state cache updates, image decode/resize, indexing/search, compression/crypto.
- Easier to enforce allocation budgets in tight loops and avoid accidental copies.

**KMP reality**
- Kotlin/JVM desktop can be fast enough, but GC pauses, object churn, and JVM memory footprint are common pain points for chat clients.
- Compose Desktop/Skiko is good, but large lists/images/animations can expose allocation pressure.

**How Lampcord reaches it**
- Keep `Snapshot`/state updates immutable or diff-based; avoid cloning whole app state every render.
- Use virtualized lists everywhere (`gpui::list`/`uniform_list`) and keep row render cheap.
- Cache decoded images with explicit size limits (memory + disk) and decode at display size.
- Batch/coalesce gateway events before notifying UI.
- Add frame-time/RSS/allocation instrumentation and performance budgets in CI/smoke tests.

### 2) Native systems and media integration

**Rust advantage**
- Strong fit for Discord gateway/voice/media: WebSockets, HTTP, crypto, audio codecs, opus, E2EE/DAVE, file I/O, native sockets.
- Easier to embed native libraries and control threading/latency.
- Can use platform APIs directly without JVM/JNI overhead in the hot path.

**KMP reality**
- KMP can do this, but often through JNI/desktop JVM interop, platform-specific native libs, or wrappers. It works, but the boundary can be awkward and costly.
- Desktop voice/media stacks often become "native code hidden behind Kotlin".

**How Lampcord reaches it**
- Put protocol, cache, media pipeline, and crypto in Rust crates.
- Keep UI as a thin consumer of Rust events/snapshots.
- For voice: isolate capture/encode/decode/playback in dedicated Rust services with clear latency metrics.

### 3) Packaging and deployment

**Rust advantage**
- Native binaries, no bundled JVM runtime required.
- Smaller downloads/updates are possible.
- Faster startup potential (no JVM boot/JIT warmup).
- Simpler process model for background services/CLI/tools.

**KMP reality**
- Compose Desktop can package with a runtime, but artifacts are larger and startup/memory can be heavier.
- Nucleus helps desktop integration, but does not remove JVM/runtime costs.

**How Lampcord reaches it**
- Static/native-friendly dependency choices where practical.
- Split features into crates; keep optional heavy deps behind feature flags.
- Measure cold start, idle RSS, and installer size as release gates.

### 4) Concurrency model

**Rust advantage**
- Tokio/async + channels + explicit ownership make it possible to build a clear backend pipeline.
- Can move CPU-heavy work off UI thread predictably.
- No accidental shared mutable state across threads if designed well.

**KMP reality**
- Coroutines/Flow are excellent ergonomically. But shared mutable state and object allocation patterns can still bite.
- JVM threading is fine; the problem is usually not "can't do concurrency", but overhead and discipline.

**How Lampcord reaches it**
- Keep one authoritative core state owner; UI receives snapshots/diffs/events.
- Never block UI render on network/disk/image decode.
- Use bounded queues and backpressure for gateway/event floods.

### 5) Webviews and native embedding (the Nucleus/wry example)

**Rust advantage**
- Rust has first-class options for native webviews/windows: `wry`, `tao`, `webkitgtk`/WebView2/WKWebView bindings.
- For specific embedded web content (OAuth flows, captcha/help pages, rich embedded content, or trusted remote views), Rust can embed a webview directly in the app process/windowing stack.

**KMP/Nucleus reality**
- The current Kotlin desktop stack benefits from Nucleus desktop integration. Depending on the exact integration, desktop wrappers often rely on system webviews (wry/webview stack) or embedded browser components. This is not inherently bad - it's pragmatic.
- But the webview boundary is another runtime/layer to manage, and behavior can differ by platform/webview engine.

**Honest take**
- "Implement wry directly in here" is possible in Rust, but **not trivial in GPUI today**. GPUI has its own rendering/windowing model; embedding a native webview requires careful platform/window handle integration and may be limited by GPUI's current capabilities.
- If we need webviews soon, the pragmatic options are:
  1. Open system browser/external auth window (simple, reliable).
  2. Use a separate wry/tao window for OAuth/captcha (practical hybrid).
  3. Invest in GPUI webview embedding only when needed and supported.

**How Lampcord reaches it**
- Don't make webview embedding a blocker for core chat parity.
- Abstract "external web content" behind a capability: `open_external_url`, `open_auth_session`, `embedded_webview` (optional).
- Use embedded webview only for cases where in-app UX is clearly better.

### 6) Security and reliability

**Rust advantage**
- Memory safety by default reduces classes of bugs in native code.
- Strong type system helps model protocol/state invariants.
- Easier to audit critical crypto/network parsing paths without GC/JNI boundary complexity.

**KMP reality**
- Kotlin is also memory-safe on JVM. Many bugs are logic bugs, not memory safety bugs.
- The risky parts are usually native dependencies anyway (voice, crypto, webview, media).

**How Lampcord reaches it**
- Keep unsafe code isolated and audited.
- Fuzz/fixture-test protocol parsing and gateway/state reducers.
- Use typed DTOs at boundaries; avoid stringly-typed JSON in UI.

## Where Kotlin Multiplatform is still better (honest)

### 1) UI development speed and ecosystem

- **Compose Multiplatform is mature for product UI.** Layout, animation, accessibility, previews, text input, theming, and component ecosystem are far ahead of current GPUI ergonomics for a Discord-like app.
- **GPUI is fast but young.** You get control and performance potential, but you pay in missing components, tooling, docs, and edge cases.
- Building rich chat UX (markdown, emoji picker, embeds, media viewer, context menus, IME, accessibility) will likely be slower in GPUI initially.

### 2) Existing code and feature parity

- Lampcord already has a large Kotlin codebase with many features implemented. Rewriting means re-solving solved problems.
- KMP gives shared mobile UI today. Rust+GPUI does not give mobile UI parity.

### 3) Libraries and integrations

- Kotlin has mature libraries for HTTP, serialization, DB, image loading, notifications, OAuth, etc.
- Rust has equivalents for many, but UI-adjacent integrations can be rougher, especially with GPUI.

### 4) Developer onboarding

- Kotlin/Compose is easier for most app contributors than Rust + custom UI framework + async runtime + FFI.

## What "better than KMP" should mean for Lampcord

Don't aim for "Rust because Rust". Aim for measurable wins:

| Metric | Target direction | Why it matters |
|---|---:|---|
| Idle CPU | lower | Chat app should be near-idle when not animating |
| Idle RSS | lower | Big pain point for Electron/JVM clients |
| Scroll/frame time | more stable | Large channels, image-heavy chats |
| Cold start | faster | Desktop UX |
| Crash/memory bug class | lower | Native core safety |
| Install/update size | smaller | Distribution |
| Voice/media latency | lower/more stable | Real-time features |
| Feature velocity | acceptable | Must not stall product |

## How to reach Rust's advantages in this rewrite

### A. Architecture rules

1. **Core owns truth; UI renders projections.**
   - `lampcord-core`/`lampcord-ffi`: authoritative state, network, cache, persistence.
   - UI gets snapshots/diffs/events, not raw mutable internals.
2. **No heavy work on UI thread.**
   - JSON parse, image decode, DB, crypto, compression, search indexing off UI thread.
3. **Coalesce updates.**
   - Gateway bursts -> batched state updates -> at most one UI notify per frame/window.
4. **Stable data identities.**
   - Message/channel/guild IDs stable; avoid rebuilding everything for small changes.

### B. Performance playbook

- **Measure first:** frame interval, render count, snapshot clone cost, HTTP bytes/reqs, RSS, image cache hits.
- **Virtualize:** messages, member lists, guild/channel lists if large.
- **Image discipline:** request sized CDN assets, decode to display size, LRU memory cache, disk cache, placeholders.
- **Avoid clones:** `Arc<Snapshot>` or diff patches; split snapshot into domain slices.
- **Bound memory:** cap message cache per channel, evict old channels/media, track bytes.
- **Background prefetch carefully:** only near viewport; cancel on scroll.

### C. Feature strategy

- **Use Rust where it shines:**
  - gateway, REST, caching, sync, DB, search, notifications plumbing, media/voice, crypto, file handling.
- **Use Compose/KMP where it shines:**
  - mobile UI, rapid product iteration, complex screen flows until GPUI parity exists.
- **GPUI desktop:**
  - invest where native perf/control matters; accept slower UI iteration early.

### D. Webview strategy

- Default: external browser for OAuth/unknown links.
- Optional: dedicated wry/tao auth window for flows needing embedded UX.
- Future: evaluate GPUI-native webview embedding only when there is a concrete need and platform support.

## Risks / honest caveats

- **GPUI maturity risk:** missing UI features, accessibility gaps, text/IME edge cases, fewer libraries.
- **Rewrite tax:** parity with 494-file Kotlin shared codebase is large.
- **FFI complexity:** Kotlin/Swift bindings, async lifecycle, memory/object mapping, debugging across boundary.
- **Performance is not automatic:** Rust can still be slow with bad architecture.
- **Mobile parity:** Rust core helps, but Rust UI does not replace Compose mobile soon.

## Recommendation

1. **Keep Option A.** Rust core + Kotlin/Compose UI for mobile; GPUI desktop as native Rust frontend.
2. **Make perf measurable now.** Baseline before big UI work.
3. **Exploit Rust in the core first.** Gateway/state/cache/media/DB are where Rust gives clear wins.
4. **Be pragmatic about webviews.** Use system browser or separate wry window when needed; don't block on GPUI webview embedding.
5. **Judge success by metrics and user experience, not language purity.**
