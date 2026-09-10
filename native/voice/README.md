# Audio voice

Android and desktop use Discord voice gateway v8, UDP/RTP, Opus (48 kHz stereo), and libdave protocol v1 for server channels and one-to-one/group DM calls. Video, screen sharing, Stage channels, and iOS voice are not implemented. Existing iOS targets and all four Android ABIs remain available.

The native library is built from source. `CMakeLists.txt` pins and SHA-256 verifies the libdave/MLSpp/BoringSSL source bundle from KyokoBot/libdave-jvm and Opus 1.5.2. No JVM wrapper or prebuilt crypto library from that project is used. Dependency notices ship in `shared/src/commonMain/resources/voice-licenses`.

## Build and checks

Desktop needs a JDK, CMake 3.26+, and a C++17 toolchain. Android uses NDK 27.2.12479018 and SDK CMake 4.1.2. Native builds are capped at two jobs; bound Gradle separately:

```sh
./gradlew :shared:desktopTest :androidApp:assembleDebug --no-daemon --no-parallel --max-workers=2 \
  '-Dorg.gradle.jvmargs=-Xmx2048m -XX:MaxMetaspaceSize=512m -XX:ActiveProcessorCount=2 -XX:+UseSerialGC' \
  -Pkotlin.compiler.execution.strategy=in-process
cmake --build build/voice-desktop --config Release --target voice_check --parallel 2
build/voice-desktop/voice_check
```

On Visual Studio generators the last executable is `build/voice-desktop/Release/voice_check.exe`. For a phone-only development APK, `-PvoiceAbis=arm64-v8a` skips the other native architectures without changing release defaults.

`voice_check` uses Discord's test external sender to form a real MLS group, exchange encrypted Opus, add a third member, execute a rekey, and reject tampered/replayed/plaintext frames. The desktop test checks the JNI boundary, AES-GCM against Java's implementation, both transport modes, RTP extensions/padding/replay handling, and jitter/sequence wrapping. Neither test records a microphone or contacts Discord. They do not replace a live interoperability call.

## Using calls

Open a server voice channel and choose **Join voice**, or use the phone button in a DM. Incoming DM calls offer **Answer** and **Decline**. The persistent call bar provides mute, deafen, disconnect, duration, encryption state, and an epoch verification code. Compare that raw hexadecimal MLS code between Lampcord participants over a separate trusted channel; it is not Discord's formatted numeric verification display.

Android requests microphone and, on Android 12+, nearby-device permission. An ongoing microphone foreground service keeps an accepted call alive with the screen off and provides notification mute/hang-up actions. Wired/Bluetooth communication devices take precedence over the earpiece unless speaker is selected. Incoming-call notifications require a running main-gateway connection and notification permission; this does not add killed-app incoming-call delivery through FCM.

Desktop uses the system default Java Sound capture/playback devices. A headset is recommended: desktop echo cancellation and device selection are not implemented. Android uses available platform echo cancellation/noise suppression. Audio is never saved to disk.

No audio is sent before DAVE is ready, and unsupported/downgraded encryption ends the call. The only unencrypted Opus payload allowed inside the authenticated RTP transport is Discord's fixed three-byte silence marker. MLS identities are ephemeral per call; ordinary rekeys retain libdave's bounded old receive keys for in-flight packets, while full MLS resets discard them. WebSocket resume keeps the transport nonce counter and MLS state; a new voice server/session gets fresh transport and DAVE state.

Before release, verify a live call against an official Discord client: both directions of audio, third-member join/leave and code changes, mute/deafen, DM ringing/answer/decline, a reconnect, logout, and Android screen-off/wired/Bluetooth routing. Windows native builds also require verification on a Windows runner.
