# Requiem TSX UI: Brief for Implementation

As of 2026-10-27. Written for an implementer who has not seen the earlier discussion. Facts marked **verified** were checked in the repo on this date. Anything marked **unverified** needs checking before it is relied on.

## 1. Purpose and status

We are changing how Requiem's view code is written. Today, feature views are Rust builder code. We want people to write UI as TSX that looks and reads like normal React, with type checking, and have Rust run it.

Status: **planning complete, no code written.** The first deliverable is a spike (Phase 1 below).

This is not a React runtime. There is no JavaScript engine, no `react-reconciler`, and no webview. TSX is parsed at build time, lowered to a small IR, and the IR is executed by Rust against the existing `requiem-ui` engine.

## 2. What exists today (verified)

- **Workspace layout.** Rust workspace under `crates/`. Relevant crates:
  - `requiem-ui`: the engine. Description in `crates/requiem-ui/Cargo.toml`: "winit + Skia + taffy + CSS theming". Layout is **taffy**, not a custom engine. Rendering is **skia-safe**.
  - `requiem-core`: app logic, stores, and feature views (`src/features/*/components/*.rs`). About 111k lines total in `src/`; about 49k lines are in `features/*/components/`.
  - `requiem`: the binary. About 18k lines.
- **Reactivity is already in the engine.** `crates/requiem-ui/src/signal.rs` provides `Signal`, `signal()`, `create_effect`, and an owner/flush model. `Effect` handles keep reactive scopes alive.
- **Node builder.** `crates/requiem-ui/src/node.rs` defines `Node` with a builder API: `class`, `classes`, `style`, `bind_class`, `bind_style`, `bind_text`, `bind_image`, `child`, `children`, `wants_focus`, `focus_scope`, `observe_size`, `observe_rect`, `on_context_menu`, and `reactive`. `Node::reactive(render)` re-runs `render` when its signals change.
- **Control flow helpers.** `crates/requiem-ui/src/control.rs`:
  - `show(tree, parent, cond, render)` mounts and unmounts a subtree when `cond` changes.
  - `for_each(tree, parent, items, key, render)` mounts new keys and unmounts stale ones, then reorders. **Known limitation:** it does not re-render an existing item when that item's data changes. Rows must bind each changing field to its own signal.
- **Stores are signal-based.** `crates/requiem-core/src/state/mod.rs` defines `AppState`, a plain struct of stores (`users`, `guilds`, `channels`, `messages`, `members`, and others). `message_store.rs` holds signals such as `by_channel: Signal<HashMap<…>>` and per-channel revision signals. **Caveat:** `by_channel` is one whole-map signal, so reading it subscribes to every channel. The global `state` facade must read through narrower accessors.
- **Class names are generated at build time.** `requiem-ui/src/lib.rs` includes `OUT_DIR/css_classes.rs` and `OUT_DIR/bundled.css`.
- **Memory baseline.** About 350 MB RSS for an account in about 200 servers. CPU and GPU are at most about 10% utilization. Most of the 350 MB is expected to be data, not UI. This has not been broken down by category.

## 3. The dialect: React surface, different execution

People write normal React-looking TSX. The execution model differs, and the compiler hides most of the difference. These are the rules.

### Matches React

- JSX, props, `useState`, `if` and ternaries in JSX, `.map` with `key`, event handlers, and plain `const` locals.
- Reading `state.*` inside JSX updates the UI.
- Handlers may close over state and props.

### Differs from React, and how it is handled

1. **Components run once.** Top-level derived locals such as `const total = a * 2` would go stale if computed once. The compiler wraps each derived local that depends on signals in a memo, so authors still write plain `const`. This is the most important rule for the compiler to get right.
2. **Reads are tracked automatically.** JSX and derived values need no dependency arrays. `useEffect(fn, deps)` may be accepted, but tracked reads decide when it runs. A lint should flag a `deps` array that disagrees with what the effect reads.
3. **Writes go through actions.** Assigning to `state.*` is a compile error. Mutations are calls to whitelisted Rust actions. This keeps gateway, REST, and state logic in Rust.
4. **`key` is required on `.map`.** React only warns. Here a missing key is a compile error.
5. **The subset is finite.** Not supported: `async`/`await`, `try`/`catch`, `for` and `while` loops, `new Date()`, `Object.*`, and other arbitrary JS. Each produces a compile error with a source span and a suggested alternative.

Not part of the surface: `React.memo`, `useMemo`, `useCallback`. The runtime makes them unnecessary. `useRef` maps to a signal or a plain value. `useContext` maps to the global `state` object or a typed per-feature context passed from Rust.

### The global `state` object

- A typed, read-only facade over the existing Rust stores, backed by their signals.
- Reads must be granular. `state.messages.in(channelId)` should track only that channel's revision signal. `state.users.get(id)` should track only that user. Reading a whole-map signal from TSX is not acceptable for list rows.
- Each store needs a designed read API. This is Rust work and is the largest cost in the plan.
- Writes are not exposed. Actions are exposed, for example `actions.sendMessage(...)`.
- TypeScript declarations for `state` and the actions are generated from the Rust types. Candidate tools: `ts-rs` or `specta` (**unverified** for this codebase). The generated declarations let `tsc` check property access, so a typo such as `state.guild.nmae` fails at build time.

## 4. Architecture

### Pipeline

1. **Parse** TSX with **Oxc** (`oxc_parser`, `oxc_semantic`). Oxc is preferred over SWC: its own benchmarks show about 3x to 4x faster parsing and about 1.3x to 1.4x lower peak memory than SWC. Those numbers come from the Oxc project's own benchmark repo and have not been independently verified.
2. **Resolve** identifiers with the semantic pass. Each identifier is a prop, a signal slot, a module constant, a `state` access, or an error.
3. **Lower** to the IR. Enforce the subset here. Map each IR node back to a source span.
4. **Interpret** the IR in `requiem-tsx`, calling the existing `Node` builder, `Signal`, `show`, and `for_each`. Nothing new is needed in `requiem-ui` for the basic cases.
5. **Type-check** UI sources with `tsc --noEmit` in CI. This uses the real TypeScript compiler. **tsrs** (Cloudflare's Rust port of the TypeScript compiler) is an option later, once it is stable. It is experimental as of this writing.

### IR shape (not binding; refine in the spike)

- **Template:** a static tree of elements and text.
- **Slots:** one per `useState` in a component, assigned at compile time. Hooks inside `if`, loops, or nested functions are compile errors.
- **Bindings:** text, class, and style expressions over the subset. Map to `bind_text`, `bind_class`, and `bind_style` where those work, and to `Node::reactive` otherwise.
- **Regions:** `if` maps to `show`, and `.map` maps to `for_each`. Each region carries its own sub-template.
- **Handlers:** each event handler is a list of slot writes and action calls. It is not a JS closure.

### Interpreter or codegen

Start with interpretation. It has one backend, allows hot reload with no rebuild, and its CPU cost is small compared with the 10% headroom. Generating Rust at build time is a later option, only if profiling shows the interpreter costs too much. Do not build both backends at the start.

### Build and release

- Development: read `.tsx` files from disk, lower them, and re-run on save.
- Release: lower at build time in `build.rs` or a build step, and embed the compiled IR in a compact binary form. The parser is not shipped in the release binary.

## 5. Phases

Each phase has a gate. Do not start the next phase until the gate passes or the gap is reported.

### Phase 0: baseline and budget

Before writing code, record the current build's numbers:

- RSS at idle, and with a large channel open.
- Frame time while scrolling a long message list.
- Update cost for one incoming message, and for a presence burst.

Use the existing automation port (see section 6). Save the numbers in this file or a sibling file.

**Gate:** numbers recorded. The budget for the new path is RSS within about +1 to 2 MB of baseline, no frame-time regression, and no change to the layout or paint output.

### Phase 1: spike

Create the `requiem-tsx` crate with:

- Oxc parsing and identifier resolution.
- Lowering for static elements, `useState`, `if`, keyed `.map`, and text, class, and style bindings.
- An interpreter that builds `Node`s through the existing API.

Port one existing component from `requiem-core/src/features/*/components/`. Choose one that uses state, a conditional, and a list. Report the candidate list and why the chosen one was picked. The thread browser (`requiem-core/src/features/threads/components/thread_browser.rs`) is a candidate, but its size and complexity are unverified.

**Gate:** the ported component produces the same `dump_tree` output as the Rust version, its screenshot matches, and Phase 0 budgets hold.

### Phase 2: subset v1 and diagnostics

- Write the subset spec as a short document. Every rule in section 3 must be covered.
- Add span-based error messages for every rejected construct.
- Run `tsc --noEmit` on the UI sources in CI.
- Add a lint for `useEffect` dependency arrays.

**Gate:** a set of negative test files, one per rejected construct, each produces the expected error.

### Phase 3: second component

Port a feature with more event handling and nested lists. Record each construct that falls outside the subset, and whether it needs an extension or a rewrite of the source. Extensions to the subset should be rare and deliberate.

**Gate:** the component passes the same checks as Phase 1, and the list of out-of-subset constructs is recorded.

### Phase 4: global state facade

- Design the read API for each store the components use, with per-key signals.
- Generate TypeScript declarations for `state` and the actions.
- Verify granularity: when one message arrives, count how many rows re-run. The count must be the rows that display that message, not every row.

**Gate:** the granularity check passes, and `tsc` catches a deliberate typo in a `state` access.

### Phase 5: hot reload

Watch `.tsx` files, re-lower changed files, and re-bind the affected feature. Static template structure is replaced and dynamic bindings are re-bound. Changes to Rust code still require a normal rebuild.

**Gate:** editing a class name or a text literal updates the running app without restart.

### Phase 6: migration by feature

Old Rust views and new TSX components coexist behind the same `Node` boundary. Migrate one feature at a time. Delete the Rust version only after its tree dump and screenshots match.

## 6. Verification

- **Tree dump.** Compare the `dump_tree` output of the Rust and TSX versions of a component.
- **Screenshots.** Compare the automation port's `screenshot` output.
- **Automation port.** Run the debug build with `UI_AUTOMATION_PORT=9232` and drive it over TCP JSON (`click`, `dump_tree`, `screenshot`). Verify in the running app, not only in unit tests.
- **Memory and frame time.** Measure against the Phase 0 numbers, using the same account and the same steps.
- **Test instance.** The user logs in to the test instance. Do not kill a running `requiem.exe` without asking.

## 7. Known engine gotchas

From the project notes. Check them before relying on any of these behaviors:

- **Occluded windows stop timers.** `set_timeout` and `request_animation_frame` do not fire while the window is hidden. Bring the test window to the foreground before judging animation.
- **`bind_style` did not update inside a rail section** in an earlier experiment, while `bind_class` and `Node::reactive` did. Confirm this before the lowering emits `bind_style` in a rail.
- **CSS transitions cover only opacity, colors, and transform.** Height animation must be driven from Rust, for example with a `Node::reactive` section reading a signal.
- **The `overflow` shorthand was missing** and has been added to the cascade. Unknown CSS properties are dropped silently, so check every style the lowering emits.
- **Absolute children position against their parent node**, not the nearest positioned ancestor.
- **`for_each` does not update existing rows** (section 2).

## 8. Out of scope

- A JavaScript runtime of any kind. No `rquickjs`, `deno_core`, or V8.
- A webview.
- `react-reconciler`.
- Full React semantics: concurrent rendering, Suspense, transitions, and the rest.
- Any change to the gateway, auth, REST, or persistence code.
- Replacing `requiem-ui`'s layout (taffy) or renderer (skia-safe).

## 9. Open questions

1. Confirm the dialect rules in section 3, especially rule 1 (memoized derived locals) and rule 3 (no writes to `state`).
2. Which component should Phase 1 port? Candidates need to be checked for size and complexity.
3. Are `ts-rs` or `specta` acceptable for generating declarations, or is a different approach preferred?
4. Should `useEffect` be in the subset at all? The recommendation is no until a concrete case needs it.

## 10. Reporting

When an implementer finishes a phase, report:

- The gate result, with the exact output of the tree dump comparison, screenshot result, and budget numbers.
- Every construct that was rejected by the subset, and every case that needed a workaround.
- Anything in this brief that turned out to be wrong, with evidence.
- Any claim marked unverified above that was checked, and the result.
