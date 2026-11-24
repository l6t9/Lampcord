# QML mobile port (Android first)

Status: **first vertical slice builds and launches on Android**. The desktop QML
app is unchanged; the Rust launcher selects `MobileMain.qml` on Android, or when
`LAMPCORD_QML_MOBILE` is set.

## Toolchain

The Qt open-source mirror available here tops out at Qt 6.7.3, while the system
desktop Qt is 6.12.0. `cxx-qt 0.10` supports all Qt 6 versions, so the mobile
build uses a self-contained Qt 6.7.3 host + Android install under `~/Qt` and
does not touch the desktop toolchain.

If you have Qt 6.12 for Android from the official installer, use that instead by
setting `QT_ANDROID_PREFIX` / `QT_HOST_PREFIX` when building.

Install with aqt:

```bash
python3 -m venv ~/aqt-venv
~/aqt-venv/bin/pip install -q aqtinstall
~/aqt-venv/bin/aqt install-qt linux desktop 6.7.3 linux_gcc_64 -O ~/Qt
~/aqt-venv/bin/aqt install-qt linux android 6.7.3 android_arm64_v8a -O ~/Qt
~/aqt-venv/bin/aqt install-qt linux android 6.7.3 android_x86_64 -O ~/Qt
```

Already available in this workspace:

- Android SDK: `/home/lampu/Android/Sdk`
- NDK: `/home/lampu/Android/Sdk/ndk/27.2.12479018`
- JDK: `/home/lampu/temp_jdk/jdk-21.0.6+7`
- `cargo-ndk`: installed via `cargo install cargo-ndk --locked`

## Build the APK

```bash
./scripts/build_qml_android.sh
```

Override the toolchain if needed:

```bash
QT_ANDROID_PREFIX=~/Qt/6.12.0/android_arm64_v8a \
QT_HOST_PREFIX=~/Qt/6.12.0/gcc_64 \
ABI=arm64-v8a \
./scripts/build_qml_android.sh
```

For the x86_64 emulator:

```bash
QT_ANDROID_PREFIX=~/Qt/6.7.3/android_x86_64 \
ABI=x86_64 \
./scripts/build_qml_android.sh
```

`androiddeployqt` emits an unsigned APK. For local testing, sign a copy with a
throwaway debug key:

```bash
JAVA_HOME=/home/lampu/temp_jdk/jdk-21.0.6+7
PATH="$JAVA_HOME/bin:$PATH"
KEYSTORE=~/.local/share/lampcord/qml-android-debug.jks
mkdir -p "$(dirname "$KEYSTORE")"
keytool -genkeypair -keystore "$KEYSTORE" \
  -storepass android -keypass android -alias androiddebugkey \
  -dname 'CN=Android Debug,O=Android,C=US' -validity 3650 \
  -keyalg RSA -keysize 2048
rust/target/android-sdk-qt/build-tools/34.0.0/apksigner sign \
  --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android \
  --v1-signing-enabled true --v2-signing-enabled true \
  --out /tmp/lampcord-qml-debug.apk \
  rust/target/android-arm64-v8a-qml/android-build/lampcord_qml.apk
adb install -r /tmp/lampcord-qml-debug.apk
adb shell monkey -p org.qtproject.example.lampcord_qml -c android.intent.category.LAUNCHER 1
```

The build script handles three Qt 6.7 quirks:

- It writes a standalone `settings.gradle` into Qt's generated Gradle project so
  Gradle does not walk up into the repository's Compose build. Its project name
  must be `android-build` for `androiddeployqt` to find the APK it generated.
- It exposes only API 34 and build-tools 34 to Qt, because `androiddeployqt`
  otherwise selects API 37+, which the bundled AGP 7.4.1 cannot parse.
- It links the Rust `staticlib` with `WHOLE_ARCHIVE`, so the QML cache and Qt
  resource initializers survive native linking, and generates
  zlib-compressed resources because this Qt Android build lacks zstd support.

The verified device run loaded `MobileMain.qml`, stayed running, and showed no
QML/resource warnings in logcat.

## Preview the mobile shell on desktop

```bash
cd rust
LAMPCORD_QML_MOBILE=1 cargo run -p lampcord-qml
```

## What works in the mobile UI

- Rust `lampcord-qml` is both the desktop binary and a library exposing
  `lampcord_qml_main()` for the Qt Android activity.
- `MobileMain.qml`: full-screen root, backend polling, navigation restore,
  safe-area margins and a double-back exit confirmation.
- `MobileBaseplate.qml`: a small navigation stack, per-tab back handling,
  configurable bottom tabs, start drawer (guild rail + channel list), chat
  surface, member drawer, pins and search actions.
- Chat: `ChatView.qml` hides the desktop header on mobile and reuses the
  message list, pins and composer. Long-press/right-click opens reply, edit and
  delete actions; the composer shows reply/edit banners and sends through the
  matching backend calls.
- `MobileDms.qml`: searchable list of open conversations.
- `MobileSearch.qml`: local finder over guilds, channels, DMs and loaded
  messages, with a persisted recent-search list. Gateway-side search is not
  exposed by the core yet.
- `MobileMentions.qml`: real mentions page backed by a new `mentions` section
  in the FFI snapshot; tapping a result opens its channel.
- `MobileYou.qml`: profile card, presence and custom-status controls,
  configurable bottom tabs and logout confirmation.
- `LoginScreen.qml`: credentials, token, MFA, QR login, SMS fallback and
  password reset. Captcha display is still not wired to the core.

Still out of scope for this slice: voice calls, reactions/threads/polls,
GIFs/stickers, notifications and conversation bubbles, secure token storage,
content-URI attachments from Android's file picker, and deep links. These need
their own Kotlin→Rust/QML port slices.
