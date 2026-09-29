# Lampcord

**Lampcord** is a lightweight, high-performance, open-source custom Discord client built with Compose Multiplatform for Android and Desktop (Linux & Windows).

It is designed to deliver a native, fast, and highly customizable Discord experience without the bloat and heavy memory footprint of Electron or WebViews.

---

## Inspiration

- **[Aliucord](https://github.com/Aliucord/Aliucord)**: Lampcord inherits a lot of my work on Aliucord, and my [plugins](https://github.com/l6t9/AliucordPlugins) for it.
- **[Nucleus](https://github.com/NucleusFramework/Nucleus)**: is used to provide proper support for Wayland.

---

## Features & Roadmap

* **Authentication**
  * [x] Password & token login
  * [x] Remote QR code login
  * [x] Multi-account switcher
* **Messaging & Chat**
  * [x] Real-time chat & gateway sync
  * [x] Markdown parsing & code highlighting
  * [x] Custom emojis, reactions & stickers
  * [x] Message replies & edit / delete
  * [x] Message pinning & thread views
  * [x] Local message deletion logger
  * [ ] Slash commands & application options
* **Voice & Media**
  * [x] Native DAVE voice protocol (DAVE v1)
  * [x] Voice channels & in-call controls
  * [x] Image / video viewer & audio player
  * [x] Custom status & activity / rich presence
  * [ ] Video calling & screen share rendering
* **Notifications**
  * [x] Android system notifications & conversation bubbles
  * [x] Desktop system notifications & toasts
  * [x] Background FCM push notification sync
* **Customization & Themes**
  * [x] Custom Material 3 Expressive (M3E) themes & Matugen color sync
  * [x] Custom font selection (Inter, Maple Mono, System)
  * [x] Custom client profiles, UserBG & UserPFP
  * [x] Client-side free Nitro emojis
* **Guild & Channel Management**
  * [ ] Server settings (roles, channels, emoji management)
  * [ ] Stage & forum channel creation and moderation

---

## Building from Source

### Prerequisites

- **JDK 17+**
- **Android SDK** (API 34+)
- **CMake & NDK** (for native voice/DAVE components)

### Desktop (Linux / Windows)

```bash
# Run Desktop App
./gradlew :desktopApp:run

# Package Linux AppImage
./gradlew :desktopApp:createAppImageLocal

# Package Windows Zip
./gradlew :desktopApp:createDistributable
```

### Android

```bash
# Assemble Debug APK
./gradlew :androidApp:assembleDebug

# Assemble Release APK
./gradlew :androidApp:assembleRelease
```

---

## Disclaimer & Notice

Lampcord is a third-party open-source client and is **not** affiliated with, endorsed by, or sponsored by Discord Inc. Using third-party clients may violate Discord's Terms of Service. Use at your own risk.

---

## License

Lampcord is released under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).
