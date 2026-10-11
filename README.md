# Lampcord

Lampcord is a lightweight Discord client built with Compose Multiplatform for Android and Desktop (Linux & Windows), with macOS and iOS targets in the build as well. It aims for a native, fast, and customizable Discord experience without the bloat and heavy memory footprint of Electron or WebViews.

Come hang out and follow development on the [Lampcord Discord server](https://discord.gg/uHXJJzxSD8).

[![Latest release](https://img.shields.io/github/v/release/l6t9/Lampcord)](https://github.com/l6t9/Lampcord/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-join-5865F2?logo=discord&logoColor=white)](https://discord.gg/uHXJJzxSD8)

## Inspiration

- **[Aliucord](https://github.com/Aliucord/Aliucord)**: Lampcord inherits a lot of my work on Aliucord and my [plugins](https://github.com/l6t9/AliucordPlugins) for it.
- **[Nucleus](https://github.com/NucleusFramework/Nucleus)**: Provides proper support for Wayland.

# Screenshots
<img width="1264" height="2780" alt="lampcord-p1" src="https://github.com/user-attachments/assets/8963d016-ec49-4273-9ab3-f92f7ec6f50a" />
<img width="2876" height="1796" alt="lampcord-p" src="https://github.com/user-attachments/assets/a35ecc79-a978-4b14-83a3-0197821b5ce9" />
<img width="1264" height="2780" alt="lampcord-s1" src="https://github.com/user-attachments/assets/6c62d1fe-36c9-4a6d-acf8-aa604603648e" />
<img width="2870" height="1796" alt="lampcord-s" src="https://github.com/user-attachments/assets/59aa0f6d-0636-41bc-9935-00a9a3ba84d3" />
<img width="1264" height="2780" alt="lampcord-1" src="https://github.com/user-attachments/assets/405d73cf-f0d1-4610-b545-bedcc6803bc1" />
<img width="2874" height="1796" alt="lampcord" src="https://github.com/user-attachments/assets/de2a75e1-2e5a-40e6-a0b2-00bac33d9ae9" />

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

The macOS DMG and iOS IPA are unsigned, so they need `xattr -d com.apple.quarantine` and a sideloading tool respectively. Every stable release also ships a `SHA256SUMS` file.

For Android, you can let [Obtainium](https://apps.obtainium.imranr.dev/redirect?r=obtainium://add/https://github.com/l6t9/Lampcord/) handle updates straight from GitHub releases.

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

### Voice (DAVE)

Android and desktop use Discord's voice gateway v8 with UDP/RTP, Opus (48 kHz stereo), and the libdave protocol v1 for server channels and one-to-one/group DM calls. The native library is built from a pinned, SHA-256 verified source bundle rather than prebuilt binaries, and `voice_check` exercises a real MLS group negotiation without recording a microphone or contacting Discord. See [`native/voice/README.md`](native/voice/README.md) for the build, test, and live-verification details.

---

## Acknowledgements

Besides the projects listed under [Inspiration](#inspiration), Lampcord stands on the libdave/MLSpp/BoringSSL sources used for end-to-end encrypted voice and the [Opus](https://opus-codec.org/) codec that carries it. Thanks to everyone maintaining the Compose Multiplatform and Kotlin ecosystem it is built on.

---

## Support

Lampcord is free and open-source under the GPL. If it earns a place in your daily setup, you can support development via [GitHub Sponsors](https://github.com/sponsors/l6t9).

---

## Important disclaimer

Lampcord is a third-party open-source client and is **not** affiliated with, endorsed by, or sponsored by Discord Inc. Lampcord may violate Discord's Terms of Service. Use it at your own risk.

---

## License

Lampcord is released under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).
