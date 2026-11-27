# Lampcord

Lampcord is a lightweight Discord client with a Kotlin Multiplatform mobile app for Android/iOS and a Rust/Skia Requiem desktop client. It aims for a native, fast, and customizable Discord experience without Electron or a webview-rendered UI.

<div style="display: flex; gap: 0.5rem; align-items: center;">
  <a href="https://github.com/l6t9/Lampcord/releases">
    <img src="https://img.shields.io/github/v/release/l6t9/Lampcord?style=for-the-badge&labelColor=0d1117" alt="Latest release" />
  </a>
  <a href="https://github.com/l6t9/Lampcord/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/l6t9/lampcord?style=for-the-badge&labelColor=0d1117" alt="License" />
  </a>
  <a href="https://github.com/l6t9/Lampcord/releases">
    <img src="https://img.shields.io/github/downloads/l6t9/Lampcord/total?style=for-the-badge&labelColor=0d1117" alt="Downloads" />
  </a>
  <a href="https://discord.gg/uHXJJzxSD8">
    <img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord" />
  </a>
</div>

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
  * [x] Video calling & screen share rendering
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

<div align="center">

<h1><a id="download-now"></a>Download now</h1>

<table>
  <tr>
    <td align="center" valign="top" width="33%">
      <h2>Android</h2>
      <p>Android 9+</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/stable/lampcord-android.apk">
          <img alt="Download Stable APK" src="assets/badges/platform/android-stable.svg" />
        </a>
      </p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/nightly-latest/lampcord-android-nightly.apk">
          <img alt="Download Nightly APK" src="assets/badges/platform/android-nightly.svg" />
        </a>
      </p>
    </td>
    <td align="center" valign="top" width="33%">
      <h2>iOS</h2>
      <p>iPhone &amp; iPad · iOS 17+ (unsigned)</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/stable/lampcord-ios.ipa">
          <img alt="Download Stable IPA" src="assets/badges/platform/ios-ipa.svg" />
        </a>
      </p>
      <p><sub>AltStore · SideStore · TrollStore · Sideloadly</sub></p>
    </td>
    <td align="center" valign="top" width="33%">
      <h2>Desktop</h2>
      <p>Windows / Linux · Rust/Skia Requiem client</p>
      <p>Build from <code>rust/</code> with <code>cargo build --release -p requiem</code>.</p>
      <p><sub>The old Compose/JVM desktop packages are retired.</sub></p>
    </td>
  </tr>
  <tr>
    <td align="center" valign="top" colspan="3">
      <h2>All Releases</h2>
      <p>All versioned mobile builds &amp; packages</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases">
          <img alt="View GitHub Releases" src="https://img.shields.io/badge/GitHub-View_Releases-181717?style=for-the-badge&logo=github&logoColor=white" />
        </a>
      </p>
    </td>
  </tr>
</table>

</div>

---

# Building from source

## Prerequisites

- **JDK 17+**
- **Android SDK** (API 34+)
- **CMake & NDK** (for native voice/DAVE components)

### Desktop (Requiem client)

Desktop is the Rust/Skia client in `rust/`. The old Compose/JVM desktop app and its
Gradle tasks have been removed.

```bash
cd rust
cargo build --release -p requiem
./target/release/requiem
```

Build a Linux AppImage after the release binary exists:

```bash
cd ..
scripts/package_appimage.sh
```

The script reads `rust/target/release/requiem` and writes `dist/Lampcord-Requiem.AppImage`.
Pass `--binary` if Cargo used a custom `CARGO_TARGET_DIR`. Packaging artifacts are written
to `dist/`. Voice on desktop is handled by Requiem's Rust voice stack; `native/voice`
remains for Android.

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

## Inspiration

- **[Aliucord](https://github.com/Aliucord/Aliucord)**: Lampcord inherits a lot of my work on Aliucord and my [plugins](https://github.com/l6t9/AliucordPlugins) for it.
- **[Nucleus](https://github.com/NucleusFramework/Nucleus)**: Provides proper support for Wayland.

---

## Important disclaimer

Lampcord is a third-party open-source client and is **not** affiliated with, endorsed by, or sponsored by Discord Inc. Lampcord may violate Discord's Terms of Service. Use it at your own risk.

---

## License

Lampcord is released under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).
