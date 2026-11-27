# TSX UI: progress and how to continue

Status as of 2026-10-27. Companion to `TSX_UI_BRIEF.md` (the original plan). Nothing here is committed yet; commit before deleting any Rust UI code.

## What this is

UI written as React-like TSX, parsed with Oxc, lowered to an IR and interpreted in Rust against the `requiem-ui` engine. No JS runtime, no webview. Components run once; derived values are signal-synced memos; reads are tracked by the engine's signals and effects, so rows update in place.

Crate: `crates/requiem-tsx`. App glue: `crates/requiem-core/src/ui_tsx.rs`. Feature TSX sits next to the Rust it replaces (`features/<feature>/components/*.tsx`).

## Flags (both default ON since 2026-10-28; set to `0` to fall back to the Rust UI)

| Variable | Effect |
| --- | --- |
| `REQUIEM_TSX_ROWS=0` | Opt out of the TSX thread browser. On by default: thread browser (header, search, list, rows, empty state) rendered from TSX |
| `REQUIEM_TSX_MESSAGES=0` | Opt out of the TSX message list. On by default: message list rendered from TSX (rows still built by the Rust row builders) |
| `UI_AUTOMATION_PORT=9232` | My test instance; `click`, `scroll`, `dump_tree`, `screenshot` over TCP JSON |

## Done

### Language and tooling (`requiem-tsx`)
- Elements: `div`, `Text`, `Button`, `Stack`, `ScrollView`, `Chevron`, per-icon components (`ThreadIcon`, ...), `<>` fragments, `&&` and `?:` regions, `.map` with required `key`, component composition across files, host components from Rust (`@requiem/app`).
- Props: function props and closures, `onClick` (with `shiftKey`), `onContextMenu` (`x`, `y`), CSS modules (`cls.rowText`), `style` strings, `className` strings.
- Typed boundaries, all generated from Rust and checked by `tsc`:
  - `@requiem/types` from specta, with branded ids (`ChannelId`, `UserId`, ...); floats come out as `number`.
  - `state.x.y(...)` queries, `actions.x(...)` from the `ui_actions!` enum, `t("key", { args })` from the Fluent catalogue (key and argument names/kinds checked at compile time and in TS).
- Compile-time diagnostics with spans; no comments and no filler CSS in TSX or Rust (see memory `feedback_no_comments_no_fake_css`).
- Declarations are split per module under `crates/requiem-core/types/` (`ui.d.ts`, `app.d.ts`, `types.d.ts`, `state.d.ts`, `actions.d.ts`, `i18n.d.ts`, `jsx.d.ts`, `react.d.ts`, `css/*.module.css.d.ts`), with `crates/requiem-core/tsconfig.json`. A test fails if they drift.
  Regenerate: `REQUIEM_UPDATE_TYPES=1 cargo test -p requiem-core --features sound --lib ui_tsx` (and `-p requiem-tsx --test spike` for the fixture).

### Engine additions
- `ScrollView` windowing: `itemHeight={n | "field"}`, `itemGap`, `initialOffset`, `onScroll`, `onNearEnd`; exactly one `.map` child; only rows near the viewport are mounted, spacers stand in for the rest.
- `ScrollView` chat mode: `position={{pinned,key,offset,height}}`, `onPosition`, `onMeasure`, `pinThreshold`. Rows are wrapped in keyed `flex-shrink: 0` divs that are direct children of the scroll node (the engine anchors on direct children). Window comes from the pinned or anchored position plus item heights.
- `Node::on_mount` hook, and `WidgetTree.defer_mount_hooks` (see gotchas).
- `control::track_locale()` so translations follow locale changes.

### Features
- Thread browser: fully TSX behind `REQUIEM_TSX_ROWS` (`ThreadBrowser`, `ThreadBrowserHeader`, `ThreadBrowserList`, `ThreadBrowserEntry`, `ThreadBrowserEmpty`, `ThreadBrowserRow`, `ThreadBrowserRowSubtext`). Verified live. The popout and modal wrappers and the channel-header button stay Rust.
- Message list, stage 1: `MessageList.tsx` and `MessageEntry.tsx` with Rust host components `MessageRowHost` (wraps the existing Rust row builders), `MessageDayDivider`, `MessageHistoryHeader`. Verified live on a real channel.
- Rust glue for messages lives in `features/messages/components/message_list/tsx_list.rs` (a child module of `message_list.rs` so it can reuse its private builders); its tests are in `tsx_list_tests.rs`.

### Architecture (the part to remember)
- One app-wide TSX instance: `ui_tsx::shared(...)` builds it once; one `UiAction` enum; queries = `ui_state::queries` + `thread_browser::browser_queries` + `message_list::tsx_queries`; host components = base (`Avatar`, `AuthorName`, `SearchBar`, `AssetImage`) + `message_list::tsx_components`. Add a feature by adding its sources to `THREAD_SOURCES` in `ui_tsx.rs`, its actions to `UiAction`, and routing in `dispatch`.
- Host components that must rebuild when data changes use the `rebuilding(props, version, build)` helper in `tsx_list.rs`: an effect recomputes a version and rebuilds the child only when it changes.
- Message list data flow: query `messages.rows` (key, height, day, unread, startsGroup) and `messages.scroll`; actions `messageListScrolled`, `messageRowsMeasured`; measured entry heights live in `ENTRY_HEIGHTS`; the list node is kept across outer `build()` reruns with `retained`/`Node::reuse` keyed by channel id.

## Gotchas (each cost real time)
- **Scratch trees.** The shell builds the message area in a scratch tree and `detach`es it into a `Node`; detaching drops effects. Anything TSX placed via scratch-tree `tree.mount` needs `section_tree.defer_mount_hooks = true` (set in `crates/requiem/src/shell.rs`). Symptom if forgotten: the TSX renders once and never updates (the message list sat on its loading skeleton forever).
- **ScrollView is a flex row.** Children stretch to the viewport height and shrink; painting skips children of a boxes that is out of view, so rows vanish once scrolled past the box height. Windowed lists force `flex-direction: column` and `flex-shrink: 0`. The Rust thread list has this bug too (left alone on purpose).
- **Text widgets ignore padding**; put padding on a wrapping div (the archived-row dot).
- **Clippy `absolute_paths` and `unused_qualifications` are denied**; `cargo build` with default features also catches the `video` cfg paths that `--features sound` clippy does not.
- **Windows tooling:** run cargo from the PowerShell tool (Git Bash's `link.exe` shadows MSVC's). FFMPEG_DIR is in memory `requiem_build_and_verify`. Bash heredocs mangle backslashes and apostrophes; write Python or Rust test files with the Write tool and run them by path.
- **Processes:** the user keeps their own instance; stop only my own pid, never `Stop-Process -Name requiem`.
- Core lib tests need `--features sound`.

## Closed since the first version (2026-10-28)
- Jump to message, jump to present and follow-own-message: `tsx_list::navigation` is an effect attached to the list's `main` div (via `on_mount`) that runs `follow_own_message`, `resolve_pending_jump` and `resolve_pending_present`.
- `ScrollView` chat mode now pushes `position` prop changes into the widget (`ScrollViewWidget::set_chat_position`, effect in `mount_scroll`); before, the prop was read once at build and a retained list never moved its real anchor. Positions the widget itself emitted are skipped (last 16 kept) so scrolling does not fight its echo.
- Avatar click: the outer build mounts the profile popout in the flagged path too (tracks `profile_popout.open`, popout user, image tick, window resize).
- Pending uploads: host components `MessagePendingNotices` (per message) and `MessagePendingBottom` (tail row); the tail height is now measured.
- Engine, found on the way: the app loop replaced its animation schedule with only the regions painted in a partial repaint, which froze animated emojis until a full repaint (`app.rs`, `carried_regions`). Fixed.
- Startup rebuild storms: `global_tick` is throttled to one bump per 60ms (`bump_tick` in `boot/events.rs`); guild rail and folder icons update in place through `live_icon` in `guild_rail.rs` (the rail section no longer reads the tick; its scratch tree needs `defer_mount_hooks`); the user panel uses the `settled` helper in `shell.rs`.

## Known gaps in the message list (stage 1)
- The Rust list builds heavy new rows in batches of 8 (`NEW_ROWS_PER_REBUILD`); the TSX list builds the whole window at once. Possible stutter on large scrolls (not measured yet).
- Row-level reuse: a changed row is unmounted and rebuilt (hover and focus state are lost).
- The history header is hosted through Rust (`MessageHistoryHeader`).

## Next steps, in order
1. Commit the work (it is all uncommitted); suggested chunks: `requiem-tsx` crate, engine changes, i18n catalogue, thread browser TSX, message list TSX.
2. Close the message-list gaps above, then have the user soak-test it (scroll, send, reactions, replies, hover toolbar, context menu).
3. The TSX thread browser and message list are now the default (opt-out `=0`). After a soak period, delete the thread browser's Rust path (`row`, `sections`, `search_results`, header and empty state in `thread_browser.rs`).
4. Message list stage 2: replace Rust row parts with TSX one at a time (leading avatar, header with name/tag/timestamp, reply preview), shrinking `MessageRowHost`. Markdown content, attachments, embeds and reactions stay host components until TSX has an inline rich-text primitive (it has no spans by design).
5. Stage 3: list chrome (day dividers, unread line, history header, jump-to-present, new-messages bar, typing clearance) as TSX.
6. Stage 4: composer, toolbar, editing and reply.
7. Measurements the brief asked for and we have not done: RSS, frame time and popout-open time, TSX path vs Rust path on the same channel.
8. Still open from the brief: style objects (`style={{}}`), component children/slots, dynamic `className` on `Text` and `Button`, `PopOver`/`Modal`/`Menu` components, an inline rich-text primitive, the Phase 0 baseline.

## Test and verify commands
```
cargo test -p requiem-tsx
cargo test -p requiem-core --features sound --lib
cargo clippy -p requiem-core -p requiem-tsx -p requiem-ui --features requiem-core/sound --all-targets
cargo build -p requiem            # default features; catches the video cfg paths
cd crates/requiem-core && npx --yes -p typescript@5.9.3 tsc -p tsconfig.json
```
Live check: build, then start `target/debug/requiem.exe` with `UI_AUTOMATION_PORT=9232` (TSX is the default; add `REQUIEM_TSX_ROWS=0 REQUIEM_TSX_MESSAGES=0` to compare with the Rust UI) from PowerShell, open a channel, and drive it over the automation port (`scripts` I used live in the session scratchpad, not in the repo: a 30-line Python TCP-JSON client).
