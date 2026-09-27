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

Below is a breakdown of implemented features and current gaps relative to the official Discord client:

### Implemented Features

| Category | Feature | Status |
| :--- | :--- | :---: |
| **Authentication** | Password & Token Login | Completed |
| | Remote QR Code Login | Completed |
| | Multi-account Switcher | Completed |
| **Messaging** | Real-time Chat & Gateway Sync | Completed |
| | Markdown Parsing & Code Highlighting | Completed |
| | Custom Emojis, Reactions & Stickers | Completed |
| | Message Replies & Edit / Delete | Completed |
| | Message Pinning & Thread Views | Completed |
| | Message Local Deletion Logger | Completed |
| **Voice & Media** | Native DAVE Voice Protocol (DAVE v1) | Completed |
| | Voice Channels & In-call Controls | Completed |
| | Image / Video Viewer & Audio Player | Completed |
| | Custom Status & Activity / Rich Presence | Completed |
| **Notifications** | Android System Notifications & Bubbles | Completed |
| | Desktop System Notifications & Toasts | Completed |
| | Background FCM Push Notification Sync | Completed |
| **Customization** | Custom Material 3 Expressive (M3E) Themes | Completed |
| | Font Selection (Inter, Maple Mono, System) | Completed |
| | Custom Client Profiles & UserBG | Completed |
| | Client-side Free Nitro Emojis | Completed |

### Current Gaps / In Progress

| Feature | Notes |
| :--- | :--- |
| **Video Calling & Screen Share** | Voice call is fully implemented; video/screenshare stream rendering is planned |
| **Slash Commands & Autocomplete** | Basic command picker exists; complex application options in progress |
| **Guild Management** | Server settings (roles, channels, emoji management) partially implemented |
| **Stage & Forum Channels** | Basic viewing supported; creation/moderation controls in progress |
| **Store & Nitro Purchases** | Not planned (out of scope for custom open-source client) |

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
