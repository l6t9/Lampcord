# QML mobile port (Android first)

The Rust QML app is the single codebase for desktop and mobile. Android and iOS
load `MobileMain.qml`; desktop keeps `Main.qml`. The mobile shell is a port of the
Compose/Phoenix navigation model: one three-state panel controller, Material 3
dynamic colors from the Rust palette, M3 expressive motion, a bottom bar that
hides while chatting, and a floating composer.

## Toolchain

Desktop and Android both use Qt 6.12.0 so `RectangularShadow` and the rest of the
QML stay identical across targets.

Installed locally:

- Host Qt: `/home/lampu/Qt/6.12.0/gcc_64`
- Android Qt: `/home/lampu/Qt/6.12.0/android_arm64_v8a`
- Android SDK: `/home/lampu/Android/Sdk`
- NDK: `/home/lampu/Android/Sdk/ndk/27.2.12479018`
- JDK: `/home/lampu/temp_jdk/jdk-21.0.6+7`
- `cargo-ndk`

Qt Android resources are generated with `--no-zstd` because that Qt build does
not ship the zstd resource plugin. The Rust `staticlib` is linked with
`WHOLE_ARCHIVE`, so the generated Qt resource initializers survive native
linking.

## Build and install

Signing uses the normal `androiddeployqt` flow with the existing Lampcord
keystore. Put the values in `local.properties` (never in git):

```properties
qt.keystore=/absolute/path/to/lampcord.jks
qt.keystore.password=...
qt.keystore.key.alias=...
qt.keystore.key.password=...
```

Then:

```bash
./scripts/build_qml_android.sh
adb install -r \
  rust/target/android-arm64-v8a-qml/android-build/build/outputs/apk/release/android-build-release-signed.apk
adb shell monkey -p me.lampu.lampcord.qml -c android.intent.category.LAUNCHER 1
```

Overrides are available through `QT_ANDROID_PREFIX`, `QT_HOST_PREFIX` and
`ABI`. The app id is `me.lampu.lampcord.qml`.

## Preview the mobile shell on desktop

```bash
LAMPCORD_QML_MOBILE=1 cargo run -p lampcord-qml
```

Set `QT_QPA_PLATFORM=offscreen` for a headless smoke test.

## Mobile shell

- `MobileMain.qml`: full-screen root, backend polling, navigation restore,
  status/navigation insets, login/connecting/chat routing, and double-back exit.
- `MobileBaseplate.qml`: the Phoenix-style three-panel controller. `panel` is
  `1` (guild/channel sidebar), `0` (current route) or `-1` (members). Panels are
  full-width and overlapping by default, tinted with the same M3E
  `surface`/`surfaceContainer` roles as Compose, and driven by a whole-shell
  horizontal drag, scrim taps, header actions and the bottom bar. The gesture
  activates after 12 dp, snaps at half the drawer width, commits a fling at
  240 dp/s, and settles in 250 ms open / 200 ms close.
- Bottom bar: `surface`, hidden while the centre chat is open, sliding back in
  when the sidebar opens or when a tab route is active. The active item uses the
  theme's `primary` colour, matching the flat Phoenix navigation treatment while
  keeping M3E colours.
- `ChatView.qml`: reuses the shared message list, reply/edit/delete actions and
  pins. On mobile the desktop header is suppressed in favour of the shell's top
  app bar, and the list scrolls behind the floating input.
- `Composer.qml`: floating M3E input — 16 dp side/bottom inset, 16 dp rounded
  `surfaceContainerHigh` field with a 1 dp subtle border, 40 dp add button,
  32 dp emoji action, 36 dp filled send button, a 56 dp fade scrim, and bottom
  safe-area padding.
- `MemberPanel.qml`: full-width overlapping member sheet with a back affordance.
- `MobileDms.qml`, `MobileSearch.qml`, `MobileMentions.qml`, `MobileYou.qml`:
  tab routes backed by the Rust snapshot/API.
- `LoginScreen.qml`: credentials, token, 2FA and QR login. The six-box
  `OtpInput.qml` mirrors Compose, auto-submits at six digits, and explicitly
  reopens the Android keyboard when a box is tapped.

Defaults that mirror the intended Phoenix shell are stored in the `navigation`
`Settings` category: `panelType = "overlapping"`,
`panelAnimation = "expressive"`, `reduceMotion = false`.

## Research data

The full Phoenix datamining dump under `/home/lampu/Downloads/` is read-only
research input. The curated copy in `docs/research/` is not committed; its
`PROVENANCE.md` says not to redistribute it.

## Still out of scope

Voice calls, reactions/threads/polls, GIFs/stickers, gateway-side message
search, conversation bubbles, secure token storage, content-URI attachments and
deep links need their own Kotlin→Rust/QML slices.
