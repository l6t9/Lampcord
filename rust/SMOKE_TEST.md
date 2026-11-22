# Rust rewrite manual smoke test checklist

Use this checklist to validate the current `lampcord` vertical slice.

## Prerequisites
- Rust toolchain compatible with `rust/` workspace (see `rust-version` in `rust/Cargo.toml`)
- Desktop session with graphics support required by GPUI (Wayland/X11 on Linux)
- A Discord token you are authorized to use (via `--token` / `--token=<TOKEN>` or `LAMPCORD_TOKEN` / `DISCORD_TOKEN`)

## Build/check
- [ ] `cd rust && cargo check -p lampcord`
- [ ] `cd rust && cargo test -p lampcord-model -p lampcord-core`

## Launch and connect
- [ ] Start app: `cd rust && cargo run -p lampcord -- --token "<TOKEN>"`
- [ ] Status progresses to connected/ready (or shows a clear error)
- [ ] On READY, current user is shown and token/account is persisted to config store

## Navigation and data
- [ ] Guild list populates (or DMs/channels as available)
- [ ] Clicking a guild updates channel list
- [ ] Clicking a channel selects it and loads recent messages
- [ ] "Load older" fetches and prepends older messages when available
- [ ] Incoming gateway messages appear in the selected channel view

## Sending
- [ ] Focus the window, type in composer, press Enter to send plain-text message
- [ ] Sent message appears in message list (or error is shown on failure)
- [ ] Esc clears composer, Shift+Enter inserts newline (basic behavior)

## Reconnect/resilience (best effort)
- [ ] Temporary disconnect/reconnect request shows reconnecting status and recovers
- [ ] Invalid token/auth failure shows auth error and does not loop indefinitely

## Persistence
- [ ] Restart app without explicit token and confirm saved token is reused (if previously saved)
- [ ] Config file exists in expected location (`$XDG_CONFIG_HOME/lampcord/accounts.json` or `~/.config/lampcord/accounts.json`)

## Notes
Record environment (OS, compositor/display server, Rust version), observed failures, logs (`[lampcord] ...` on stderr), and screenshots if UI issues occur.
