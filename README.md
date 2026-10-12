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

<div align="center">

<h1><a id="download-now"></a>Download now</h1>

<table>
  <tr>
    <td align="center" valign="top" width="33%">
      <h2>Android</h2>
      <p>Android 9+</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/latest/download/lampcord-android.apk">
          <img alt="Download Stable APK" src="https://img.shields.io/badge/Stable_APK-Android-3DDC84?style=for-the-badge&logo=android&logoColor=white" />
        </a>
      </p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/nightly-latest/lampcord-android-nightly.apk">
          <img alt="Download Nightly APK" src="https://img.shields.io/badge/Nightly_APK-Android-2e8b57?style=for-the-badge&logo=android&logoColor=white" />
        </a>
      </p>
    </td>
    <td align="center" valign="top" width="33%">
      <h2>Windows</h2>
      <p>Windows 10+ · x64</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/latest/download/lampcord-desktop-windows-x64.zip">
          <img alt="Download Stable ZIP" src="https://img.shields.io/badge/Stable_ZIP-Windows-0078D4?style=for-the-badge&logo=windows&logoColor=white" />
        </a>
      </p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/nightly-latest/lampcord-desktop-windows-x64-nightly.zip">
          <img alt="Download Nightly ZIP" src="https://img.shields.io/badge/Nightly_ZIP-Windows-005a9e?style=for-the-badge&logo=windows&logoColor=white" />
        </a>
      </p>
    </td>
    <td align="center" valign="top" width="33%">
      <h2>Linux</h2>
      <p>x86_64 · AppImage or packages</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/latest/download/lampcord-desktop-linux-x64.AppImage">
          <img alt="Download Stable AppImage" src="https://img.shields.io/badge/Stable_AppImage-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" />
        </a>
      </p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/nightly-latest/lampcord-desktop-linux-x64-nightly.AppImage">
          <img alt="Download Nightly AppImage" src="https://img.shields.io/badge/Nightly_AppImage-Linux-d4a017?style=for-the-badge&logo=linux&logoColor=black" />
        </a>
      </p>
      <p>
        <a href="https://aur.archlinux.org/packages/lampcord-bin">
          <img alt="AUR Package" src="https://img.shields.io/badge/AUR_Package-lampcord--bin-1793D1?style=for-the-badge&logo=archlinux&logoColor=white" />
        </a>
      </p>
    </td>
  </tr>
  <tr>
    <td align="center" valign="top" width="33%">
      <h2>macOS</h2>
      <p>macOS 12+ · x64 / ARM (unsigned)</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/latest/download/lampcord-desktop-macos.dmg">
          <img alt="Download Stable DMG" src="https://img.shields.io/badge/Stable_DMG-macOS-000000?style=for-the-badge&logo=apple&logoColor=white" />
        </a>
      </p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/nightly-latest/lampcord-desktop-macos-nightly.dmg">
          <img alt="Download Nightly DMG" src="https://img.shields.io/badge/Nightly_DMG-macOS-333333?style=for-the-badge&logo=apple&logoColor=white" />
        </a>
      </p>
    </td>
    <td align="center" valign="top" width="33%">
      <h2>iOS</h2>
      <p>iPhone &amp; iPad · iOS 17+ (unsigned)</p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/latest/download/lampcord-ios.ipa">
          <img alt="Download Stable IPA" src="https://img.shields.io/badge/Stable_IPA-iOS-000000?style=for-the-badge&logo=apple&logoColor=white" />
        </a>
      </p>
      <p>
        <a href="https://github.com/l6t9/Lampcord/releases/download/nightly-latest/lampcord-ios-nightly.ipa">
          <img alt="Download Nightly IPA" src="https://img.shields.io/badge/Nightly_IPA-iOS-333333?style=for-the-badge&logo=apple&logoColor=white" />
        </a>
      </p>
      <p><sub>AltStore · SideStore · TrollStore · Sideloadly</sub></p>
    </td>
    <td align="center" valign="top" width="33%">
      <h2>All Releases</h2>
      <p>All versioned builds &amp; packages</p>
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
