# Performance notes

These are the performance rules the app follows, plus the known remaining work.

## Dispatchers

Shared stores run on `Dispatchers.Default`. UI-only collectors stay on `Dispatchers.Main`.
Gateway events are handled off the main thread. Read acknowledgements are debounced before
the REST request and the local preference write.

## Message list

- Only the active channel's messages are published; updates to other cached channels do not re-emit the list.
- Unread count and first unread are derived state, not recomputed on every emission.
- Scroll and autoload effects read the current message list instead of capturing a stale one.
- Blocked authors are a `Set<String>`, built once per relationship change.
- Rows do not use `zIndex`, per-row `graphicsLayer`, or `animateItem()`.
- Message body scans are remembered by content and embed data.

## Images

- Coil has a 256 MB disk cache under the app storage directory and a 32 MB memory cache.
- Requests do not override Coil's cache key, so size and data are part of the key.
- Small images skip the crossfade.
- **Size-bucketed CDN URLs**: All avatar and guild icon URLs now go through `CdnUrls.getUserAvatarUrl`, `CdnUrls.getGuildIconUrl`, etc., which normalize sizes to standard buckets (16, 32, 64, 128, 256, 512, 1024). This reduces cache fragmentation by ensuring the same logical image always uses the same URL.
- Twemoji resource reads off the composition thread.
- Remaining: blackhole prefetch for the next page of avatars and attachments (deferred).

## State lookup

- `UserStore` exposes `memberState(guildId, userId)` and `userState(userId)` for per-row member and user observation.
- `PresenceStore` exposes `presenceState(userId)` for per-row presence observation.
- `GuildStore` exposes per-guild unread and mention state plus a private-DM mention total.
- `EntityStore` exposes `channelsForGuild(guildId)`; the sidebar observes only the selected guild's channels.
- Channel updates that only change the last message id do not re-emit the sidebar list.
- Channel rows observe only their own read state instead of the whole read-state map.
- Gateway DM updates go through one channel update instead of two.
- Remaining: update guild unread and mention totals incrementally instead of rescanning every cached channel on each message.

## Database

The message log has a composite `(channelId, timestamp)` index and a 90-day retention pass.
Queries and JSON decoding run on IO or Default.

Remaining: transactional batch inserts, offline message cache, and a storage settings page.

## Desktop

The window backend is Nucleus 3.0 with system decorations. Hyprland uses `undecorated = true`.
Window size and placement are persisted. Remaining: native window cache trimming and a bundled
JDK 25 runtime so Nucleus can generate and ship a Project Leyden AOT cache for faster startup.

## Next

These are the larger changes that need their own PRs:

1. Blackhole prefetch for the next page of avatars and attachments.
2. Offline message cache served from the database with a stale-data badge.
3. Registry-driven settings with deep search that scrolls to and highlights the matching row.
4. A storage page with image and database usage, plus a confirmation flow for reset.
5. A typed error and empty-state kit with retry actions.
6. Live tray state, close-to-tray, and a desktop shortcut layer.
7. `commonTest` plus CI gates, ktlint, Android lint, and split verification from packaging.
8. A Macrobenchmark module for startup and message-list scroll.
