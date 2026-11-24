#!/usr/bin/env bash
# Build the Rust QML client as an Android APK using Qt's CMake toolchain.
# Requires Qt for Android 6.12, the matching host Qt, and cargo-ndk.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QT_ANDROID_PREFIX="${QT_ANDROID_PREFIX:-$HOME/Qt/6.12.0/android_arm64_v8a}"
QT_HOST_PREFIX="${QT_HOST_PREFIX:-$HOME/Qt/6.12.0/gcc_64}"
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_SDK_ROOT/ndk/27.2.12479018}"
JAVA_HOME="${JAVA_HOME:-/home/lampu/temp_jdk/jdk-21.0.6+7}"
ABI="${ABI:-arm64-v8a}"
BUILD_DIR="${BUILD_DIR:-$REPO_ROOT/rust/target/android-$ABI-qml}"

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

export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_SDK_ROOT
export ANDROID_NDK_HOME
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export JAVA_HOME
# cxx-qt's build script must compile against the Android Qt headers, not the
# host Qt headers that happen to be earlier in PATH for the desktop build.
export QMAKE="$QT_ANDROID_PREFIX/bin/qmake"
# The Android Qt package is built without zstd support, while host RCC defaults
# to zstd. Generate zlib-compressed resources so the generated code can link.
export CXX_QT_AUTORCC_OPTIONS="--no-zstd"
export PATH="$JAVA_HOME/bin:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$QT_HOST_PREFIX/bin:$PATH"

# Qt signs the release APK with these variables (see QT_ANDROID_SIGN_APK).
local_properties="$REPO_ROOT/local.properties"
if [ ! -f "$local_properties" ]; then
    echo "Missing $local_properties with release signing properties" >&2
    exit 2
fi
while IFS='=' read -r key value || [ -n "${key:-}" ]; do
    case "$key" in
        release.storeFile)
            case "$value" in
                /*) export QT_ANDROID_KEYSTORE_PATH="$value" ;;
                *) export QT_ANDROID_KEYSTORE_PATH="$REPO_ROOT/$value" ;;
            esac
            ;;
        release.keyAlias) export QT_ANDROID_KEYSTORE_ALIAS="$value" ;;
        release.storePassword) export QT_ANDROID_KEYSTORE_STORE_PASS="$value" ;;
        release.keyPassword) export QT_ANDROID_KEYSTORE_KEY_PASS="$value" ;;
    esac
done < "$local_properties"
for var in QT_ANDROID_KEYSTORE_PATH QT_ANDROID_KEYSTORE_ALIAS QT_ANDROID_KEYSTORE_STORE_PASS QT_ANDROID_KEYSTORE_KEY_PASS; do
    if [ -z "${!var:-}" ]; then
        echo "Missing $var in $local_properties" >&2
        exit 2
    fi
done

echo "==> Building Rust QML library for $RUST_TARGET"
(
    cd "$REPO_ROOT/rust"
    cargo ndk \
        --target "$ABI" \
        --platform 34 \
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
    -DANDROID_PLATFORM=android-34 \
    -DQT_HOST_PATH="$QT_HOST_PREFIX" \
    -DCMAKE_BUILD_TYPE=Release \
    -DLAMPCORD_RUST_LIB="$RUST_LIB"

echo "==> Building signed APK"
cmake --build "$BUILD_DIR"

echo
echo "Signed APK is under:"
echo "  $BUILD_DIR"
find "$BUILD_DIR" -name '*.apk' -type f -print
