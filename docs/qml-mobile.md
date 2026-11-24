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

## What is in the first slice

- Rust `lampcord-qml` is now both the desktop binary and a library exposing
  `lampcord_qml_main()` for the Qt Android activity.
- `MobileMain.qml`: full-screen root, backend polling, navigation restore,
  Android back-key handling.
- `MobileBaseplate.qml`: Compose-like start drawer (guild rail + channel list),
  centre chat surface, member drawer, and five-tab bottom navigation.
- Reuses the existing QML login, connecting, chat, composer, emoji, pins,
  account and member components.

Deliberately still stubbed in this slice: Search, Mentions, You settings tree,
voice calls, GIFs/stickers, captcha solving, notifications, secure token
storage, and deep links. These need their own Kotlin→Rust/QML port slices.
