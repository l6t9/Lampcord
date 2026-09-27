# Lampcord

**Lampcord** is a lightweight, high-performance, open-source custom Discord client built with Compose Multiplatform for Android and Desktop (Linux & Windows).

It is designed to deliver a native, fast, and highly customizable Discord experience without the bloat and heavy memory footprint of Electron or WebViews.

---

## Purpose & Inspiration

Lampcord was born out of a desire for a smooth, battery-friendly, responsive Discord client on mobile and desktop platforms.

- **Inspired by Aliucord**: Lampcord inherits design philosophy and client-side modding concepts from [Aliucord](https://github.com/Aliucord/Aliucord), bringing modern Material 3 / Expressive UI, deep customization, and client-side enhancements into a native multiplatform app.
- **Compose Multiplatform**: Powered by Kotlin Multiplatform and Jetpack Compose, sharing core gateway logic, entity state management, and UI across Android and Desktop.

---

## Roadmap & Features

### Core Features

- [x] **Authentication**
  - [x] Password & Token Login
  - [x] Remote QR Code Login
  - [x] Multi-account Switcher
- [x] **Messaging & Chat**
  - [x] Real-time Chat & Gateway Sync
  - [x] Markdown Parsing & Code Highlighting
  - [x] Custom Emojis, Reactions & Stickers
  - [x] Message Replies & Edit / Delete
  - [x] Message Pinning & Thread Views
  - [x] Local Message Deletion Logger
- [x] **Voice & Media**
  - [x] Native DAVE Voice Protocol (DAVE v1)
  - [x] Voice Channels & In-call Controls
  - [x] Image / Video Viewer & Audio Player
  - [x] Custom Status & Activity / Rich Presence
- [x] **Notifications**
  - [x] Android System Notifications & Conversation Bubbles
  - [x] Desktop System Notifications & Toasts
  - [x] Background FCM Push Notification Sync
- [x] **Customization & Themes**
  - [x] Custom Material 3 Expressive (M3E) Themes & Matugen Color Sync
  - [x] Custom Font Selection (Inter, Maple Mono, System)
  - [x] Custom Client Profiles, UserBG & UserPFP
  - [x] Client-side Free Nitro Emojis

### Gaps & Planned Enhancements

- [ ] **Video Calling & Screen Share** *(Voice calls supported; video stream rendering planned)*
- [ ] **Slash Commands & Autocomplete** *(Basic command picker implemented; application options in progress)*
- [ ] **Guild Management** *(Server settings: roles, channels, emoji management partially implemented)*
- [ ] **Stage & Forum Channels** *(Basic viewing supported; creation and moderation controls in progress)*

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
