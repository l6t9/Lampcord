# Requiem visual parity for mobile

Desktop is now the Rust/Skia Requiem client in `rust/`. Android and iOS stay native
Kotlin/Compose, but their shell now uses Requiem's desktop token palette and metrics.

## Source of truth

- `rust/crates/requiem-core/src/ui/themes.css`
- `rust/crates/requiem-core/src/ui/tokens.css`
- `rust/crates/requiem-core/src/features/*/components/*.module.css`

The Compose mirror lives in `shared/src/commonMain/kotlin/me/lampu/lampcord/shared/ui/theme/Color.kt`
as `RequiemColors` and `RequiemMetrics`.

## Material 3 Expressive theme

A new palette style, `REQUIEM_EXPRESSIVE`, is now the default for fresh installs. It uses
MaterialKolor's `Expressive` palette with Requiem blurple as the seed, then maps the
resulting `ColorScheme` onto Requiem's desktop surfaces:

- guild rail / bottom nav: `#121214`
- channel sidebar / member list: `#1A1A1E`
- chat surface: `#222327`
- raised surfaces: `#242429` / `#2B2D31`
- brand: `#5865F2`
- primary text: `#F2F3F5`

It is selectable under Appearance → Color palette → Visual Style. On the first launch after
this update, installs that still used the old default `Tonal Spot` palette are migrated once;
custom palettes are left alone.

## Restyled mobile shell

The first parity pass covers the surfaces that define the Discord/Requiem silhouette:

- `LoginScreen`: deep purple auth background, centered 480dp column, 240dp QR card.
- `GuildRail` / `RegularGuildItem`: 72dp rail, white selected pill, muted unread pill.
- `Sidebar`: Requiem rail and sidebar backgrounds behind guild/channel lists.
- `GuildChannelList`: 48dp server header using the sidebar token.
- `ChannelHeader`: 48dp header using the chat surface token.
- `ChatArea`: chat surface token behind the message list.
- `MemberList`: Requiem member panel surface.
- `MobileBaseplate`: Requiem-colored bottom navigation bar.

## Scope

This is a visual pass, not a Skia/TSX port. Requiem's Rust UI is not embedded in mobile;
native IME, accessibility, lifecycle, notifications, back handling, and platform media
stay with Compose/Android/iOS.
