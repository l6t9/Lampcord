# Lampcord
Lampcord is a lightweight Discord client built with Compose Multiplatform for Android and Desktop (Linux & Windows).

It is designed to deliver a native, fast, and customizable Discord experience without the bloat and heavy memory footprint of Electron or WebViews.

## Inspiration
- **[Aliucord](https://github.com/Aliucord/Aliucord)**: Lampcord inherits a lot of my work on Aliucord, and my [plugins](https://github.com/l6t9/AliucordPlugins) for it.
- **[Nucleus](https://github.com/NucleusFramework/Nucleus)**: Used to provide proper support for Wayland.

# Screenshots
<img width="1264" height="2780" alt="lampcord-p1" src="https://github.com/user-attachments/assets/8963d016-ec49-4273-9ab3-f92f7ec6f50a" />
<img width="2876" height="1796" alt="lampcord-p" src="https://github.com/user-attachments/assets/a35ecc79-a978-4b14-83a3-0197821b5ce9" />
<img width="1264" height="2780" alt="lampcord-s1" src="https://github.com/user-attachments/assets/6c62d1fe-36c9-4a6d-acf8-aa604603648e" />
<img width="2870" height="1796" alt="lampcord-s" src="https://github.com/user-attachments/assets/59aa0f6d-0636-41bc-9935-00a9a3ba84d3" />
<img width="1264" height="2780" alt="lampcord-1" src="https://github.com/user-attachments/assets/405d73cf-f0d1-4610-b545-bedcc6803bc1" />
<img width="2874" height="1796" alt="lampcord" src="https://github.com/user-attachments/assets/de2a75e1-2e5a-40e6-a0b2-00bac33d9ae9" />

---

>[!Warning]
>**Lampcord is in BETA**
>
>The client is still in developement and many issues that we're currently fixing.
>
>So don't delete your current client yet. Unless you're sure about it.

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

<h1><a id="download-now"></a>Download Today !</h1>

<h2>Stable Release</h2>

<table>
  <tr>
    <th align="center">Obtainium</th>
  </tr>
  <tr>
    <td align="center">
      <a href="https://apps.obtainium.imranr.dev/redirect?r=obtainium://add/https://github.com/l6t9/Lampcord/">
        <img src="https://github.com/ImranR98/Obtainium/blob/main/assets/graphics/badge_obtainium.png" alt="Download from Obtainium" height="50">
      </a>
    </td>
  </tr>
  <tr>
    <th align="center" colspan="2">GitHub</th>
  </tr>
  <tr>
    <td align="center" colspan="2">
      <a href="https://github.com/l6t9/Lampcord/releases/latest/">
        <img src="https://github.com/machiav3lli/oandbackupx/blob/034b226cea5c1b30eb4f6a6f313e4dadcbb0ece4/badge_github.png" alt="Download from GitHub" height="75">
      </a>
    </td>
  </tr>
</table>

<h2>Nightly Build</h2>

<table>
  <thead>
    <tr>
      <th align="center">GitHub</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="center">
        <a href="https://github.com/l6t9/Lampcord/releases/tag/nightly">
          <img src="https://github.com/machiav3lli/oandbackupx/blob/034b226cea5c1b30eb4f6a6f313e4dadcbb0ece4/badge_github.png" alt="Download from GitHub" height="75">
        </a>
      </td>
    </tr>
  </tbody>
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

## Important disclaimer

Lampcord is a third-party open-source client and is **not** affiliated with, endorsed by, or sponsored by Discord Inc. Lampcord DOES use third-party clients may violate Discord's Terms of Service. Use it at your own risk.

---

## License

Lampcord is released under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).

# Thank you for considering out Lampcord :)
