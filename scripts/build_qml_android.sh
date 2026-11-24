#!/usr/bin/env bash
# Build the Rust QML client as an Android APK using the same cxx-qt module as
# the desktop app. Requires Qt for Android and cargo-ndk.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QT_ANDROID_PREFIX="${QT_ANDROID_PREFIX:-$HOME/Qt/6.7.3/android_arm64_v8a}"
QT_HOST_PREFIX="${QT_HOST_PREFIX:-$HOME/Qt/6.7.3/gcc_64}"
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_SDK_ROOT/ndk/27.2.12479018}"
JAVA_HOME="${JAVA_HOME:-/home/lampu/temp_jdk/jdk-21.0.6+7}"
ABI="${ABI:-arm64-v8a}"
BUILD_DIR="${BUILD_DIR:-$REPO_ROOT/rust/target/android-$ABI-qml}"
# Qt 6.7's androiddeployqt always picks the highest installed Android platform,
# but its bundled AGP 7.4.1 cannot read the API 37+ SDK metadata. Give Qt a
# narrow SDK view so packaging uses API 34 / build-tools 34.0.0.
QT_SDK="$REPO_ROOT/rust/target/android-sdk-qt"

case "$ABI" in
    arm64-v8a) RUST_TARGET="aarch64-linux-android" ;;
    x86_64) RUST_TARGET="x86_64-linux-android" ;;
    *) echo "Unsupported ABI: $ABI" >&2; exit 2 ;;
esac

for path in "$QT_ANDROID_PREFIX" "$QT_HOST_PREFIX" "$ANDROID_NDK_HOME" "$JAVA_HOME"; do
    if [ ! -d "$path" ]; then
        echo "Missing path: $path" >&2
        echo "Set QT_ANDROID_PREFIX / QT_HOST_PREFIX / ANDROID_NDK_HOME / JAVA_HOME." >&2
        exit 2
    fi
done

mkdir -p "$QT_SDK/platforms" "$QT_SDK/build-tools" "$QT_SDK/platform-tools"
ln -sfn "$ANDROID_SDK_ROOT/platforms/android-34" "$QT_SDK/platforms/android-34"
ln -sfn "$ANDROID_SDK_ROOT/build-tools/34.0.0" "$QT_SDK/build-tools/34.0.0"
ln -sfn "$ANDROID_SDK_ROOT/platform-tools" "$QT_SDK/platform-tools"

export ANDROID_HOME="$QT_SDK"
export ANDROID_SDK_ROOT="$QT_SDK"
export ANDROID_NDK_HOME
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export JAVA_HOME
# cxx-qt's build script must compile against the Android Qt headers, not the
# host Qt headers that happen to be earlier in PATH for the desktop build.
export QMAKE="$QT_ANDROID_PREFIX/bin/qmake"
# Qt's Android package here was built without zstd support, but host RCC uses
# zstd compression by default. Generate zlib-compressed resources instead.
export CXX_QT_AUTORCC_OPTIONS="--no-zstd"
export PATH="$JAVA_HOME/bin:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$QT_HOST_PREFIX/bin:$PATH"

echo "==> Building Rust QML library for $RUST_TARGET"
(
    cd "$REPO_ROOT/rust"
    cargo ndk \
        --target "$ABI" \
        --platform 27 \
        build --release \
        -p lampcord-qml \
        --lib
)

RUST_LIB="$REPO_ROOT/rust/target/$RUST_TARGET/release/liblampcord_qml.a"
if [ ! -f "$RUST_LIB" ]; then
    echo "Rust library not found at $RUST_LIB" >&2
    exit 1
fi

echo "==> Configuring Qt Android app"
cmake -S "$REPO_ROOT/rust/crates/lampcord-qml/android" -B "$BUILD_DIR" -G Ninja \
    -DCMAKE_TOOLCHAIN_FILE="$QT_ANDROID_PREFIX/lib/cmake/Qt6/qt.toolchain.cmake" \
    -DQT_CHAINLOAD_TOOLCHAIN_FILE="$ANDROID_NDK_HOME/build/cmake/android.toolchain.cmake" \
    -DANDROID_ABI="$ABI" \
    -DANDROID_PLATFORM=android-27 \
    -DANDROID_SDK_ROOT="$ANDROID_SDK_ROOT" \
    -DANDROID_NDK_ROOT="$ANDROID_NDK_HOME" \
    -DQT_HOST_PATH="$QT_HOST_PREFIX" \
    -DCMAKE_BUILD_TYPE=Release \
    -DLAMPCORD_RUST_LIB="$RUST_LIB"

echo "==> Building APK"
cmake --build "$BUILD_DIR"

echo
echo "APK output is under:"
echo "  $BUILD_DIR"
find "$BUILD_DIR" -name '*.apk' -type f -print
