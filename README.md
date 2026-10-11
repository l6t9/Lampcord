# Lampcord

Lampcord is a lightweight Discord client built with Compose Multiplatform for Mobile and Desktop. It aims for a native, fast, and customizable Discord experience without the bloat and heavy memory footprint of Electron or WebViews.

[![Latest release](https://img.shields.io/github/v/release/l6t9/Lampcord)](https://github.com/l6t9/Lampcord/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-join-5865F2?logo=discord&logoColor=white)](https://discord.gg/uHXJJzxSD8)

## Inspiration

- **[Aliucord](https://github.com/Aliucord/Aliucord)**: Lampcord inherits a lot of my work on Aliucord and my [plugins](https://github.com/l6t9/AliucordPlugins) for it.
- **[Nucleus](https://github.com/NucleusFramework/Nucleus)**: Provides proper support for Wayland.

---

> [!Warning]
> **Lampcord is in BETA**
>
> This is still in development and may not be ready for day-to-day use
>

# Features roadmap
## Authentication
  * [x] Password & token login
  * [x] Remote QR code login
  * [x] Multi-account switcher
## Messaging and chat
  * [x] Real-time chat & gateway sync
  * [x] Markdown parsing & code highlighting
  * [x] Custom emojis, reactions & stickers
  * [x] Message replies & edit / delete
  * [x] Message pinning & thread views
  * [x] Local message deletion logger
  * [ ] Slash commands & application options
## Voice and media
  * [x] Native DAVE voice protocol (DAVE v1)
  * [x] Voice channels & in-call controls
  * [x] Image / video viewer & audio player
  * [x] Custom status & activity / rich presence
  * [ ] Video calling & screen share rendering
## Notifications
  * [x] Android system notifications & conversation bubbles
  * [x] Desktop system notifications & toasts
  * [x] Background FCM push notification sync
## Customization and themes
  * [x] Custom Material 3 Expressive (M3E) themes & Matugen color sync
  * [x] Custom font selection (Inter, Maple Mono, System)
  * [x] Custom client profiles, UserBG & UserPFP
  * [x] Client-side free Nitro emojis
## Guild and channel management
  * [ ] Server settings (roles, channels, emoji management)
  * [ ] Stage & forum channel creation and moderation

---

<a id="download-now"></a>
# Download

Two channels are published on the [releases page](https://github.com/l6t9/Lampcord/releases): tagged **stable** builds under `v<version>` tags, and rolling **nightly** builds under the `nightly` tag.

| Platform | Stable | Nightly | Artifact |
| --- | --- | --- | --- |
| **Android** (9+) | [Download APK](https://github.com/l6t9/Lampcord/releases/latest) | [Download APK](https://github.com/l6t9/Lampcord/releases/tag/nightly) | `lampcord-android-<version>.apk` |
| **Windows** (10+, x64) | [Download ZIP](https://github.com/l6t9/Lampcord/releases/latest) | [ZIP / EXE installer](https://github.com/l6t9/Lampcord/releases/tag/nightly) | `lampcord-desktop-windows-x64-<version>.zip` |
| **macOS** (unsigned) | [Download DMG](https://github.com/l6t9/Lampcord/releases/latest) | [Download DMG](https://github.com/l6t9/Lampcord/releases/tag/nightly) | `lampcord-desktop-macos-<version>.dmg` |
| **Linux** (x86_64) | [Download AppImage](https://github.com/l6t9/Lampcord/releases/latest) | [AppImage / DEB / RPM / Arch](https://github.com/l6t9/Lampcord/releases/tag/nightly) | `lampcord-desktop-linux-x64-<version>.AppImage` |
| **iOS** (unsigned) | [Download IPA](https://github.com/l6t9/Lampcord/releases/latest) | [Download IPA](https://github.com/l6t9/Lampcord/releases/tag/nightly) | `lampcord-ios-<version>.ipa` |

---

# Building from source

## Prerequisites

- **JDK 17+**
- **Android SDK** (API 34+)
- **CMake & NDK** (for native voice/DAVE components)

### Desktop (Linux / macOS / Windows)

```bash
# Run Desktop App
./gradlew :desktopApp:run

# Package Linux AppImage
./gradlew :desktopApp:createAppImageLocal

# Package macOS DMG (macOS only)
./gradlew :desktopApp:packageReleaseDmg

# Package Windows Zip
./gradlew :desktopApp:createDistributable
```

The macOS DMG is unsigned, like the iOS IPA: there is no `codesign` identity or notarisation
in the build, so macOS will need `xattr -d com.apple.quarantine` after first download.

### Android

```bash
# For assembling the debug APK
./gradlew :androidApp:assembleDebug

# For assembling the release APK
./gradlew :androidApp:assembleRelease
```

---

## Support

Lampcord is free and open-source under GPL-3.0. If you like it, you can support development via [GitHub Sponsors](https://github.com/sponsors/l6t9).

---

## Important disclaimer

Lampcord is a third-party open-source client and is **not** affiliated with, endorsed by, or sponsored by Discord Inc. Lampcord may violate Discord's Terms of Service. Use it at your own risk.

---

## License

Lampcord is released under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).
