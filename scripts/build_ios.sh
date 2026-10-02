#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/.."

SCHEME="iosApp"
PROJECT="iosApp/iosApp.xcodeproj"
BUNDLE_ID="me.lampu.lampcord"
BUILD_DIR="iosApp/build"
DERIVED_DATA_DIR="iosApp/DerivedData"
DIST_DIR="dist"

mode="ipa"
run_after_build=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ipa|--release) mode="ipa" ;;
    --sim|--simulator) mode="sim" ;;
    --device) mode="device" ;;
    --run) run_after_build=1 ;;
    --clean) rm -rf "$BUILD_DIR" "$DERIVED_DATA_DIR" "$PROJECT/xcuserdata" "$PROJECT/project.xcworkspace/xcuserdata" ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

if ! command -v xcodebuild > /dev/null 2>&1; then
  if [[ "${CI:-}" == "true" ]]; then
    echo "xcodebuild is required to build the iOS app and is not available on this runner" >&2
    exit 1
  fi
  echo "xcodebuild is not available; skipping the iOS build." >&2
  exit 0
fi

version="$(sed -n 's/^appVersion=//p' gradle.properties)"
if [[ ! "$version" =~ ^([0-9]+\.[0-9]+\.[0-9]+)(-a([0-9]+))?(-nightly\.([0-9]+))?$ ]]; then
  echo "appVersion is not a version I can put in a bundle: $version" >&2
  exit 1
fi
MARKETING_VERSION="${BASH_REMATCH[1]}"
CURRENT_PROJECT_VERSION="${BASH_REMATCH[3]:-${BASH_REMATCH[5]:-1}}"

COMMON_SETTINGS=(
  -project "$PROJECT"
  -scheme "$SCHEME"
  -configuration "$([ "$mode" = "ipa" ] && echo Release || echo Debug)"
  -derivedDataPath "$DERIVED_DATA_DIR"
  MARKETING_VERSION="$MARKETING_VERSION"
  CURRENT_PROJECT_VERSION="$CURRENT_PROJECT_VERSION"
  PRODUCT_BUNDLE_IDENTIFIER="$BUNDLE_ID"
)

UNSIGNED_SETTINGS=(
  CODE_SIGNING_ALLOWED=NO
  CODE_SIGNING_REQUIRED=NO
  CODE_SIGN_IDENTITY=""
  AD_HOC_CODE_SIGNING_ALLOWED=YES
  ENABLE_DEBUG_DYLIB=NO
)

mkdir -p "$DIST_DIR"

case "$mode" in
  ipa)
    archive_dir="$BUILD_DIR/Build/Products/Release-iphoneos/Lampcord.xcarchive"
    xcodebuild archive "${COMMON_SETTINGS[@]}" "${UNSIGNED_SETTINGS[@]}" \
      -archivePath "$archive_dir"

    app="$archive_dir/Products/Applications/Lampcord.app"
    [[ -d "$app" ]] || { echo "xcodebuild did not produce an .app" >&2; exit 1; }

    plist_value() { /usr/libexec/PlistBuddy -c "Print :$1" "$app/Info.plist"; }
    [[ "$(plist_value DTPlatformName)" == "iphoneos" ]] || { echo "The archive is not a device build" >&2; exit 1; }
    [[ "$(plist_value CFBundleIdentifier)" == "$BUNDLE_ID" ]] || { echo "The archive has the wrong bundle identifier" >&2; exit 1; }
    [[ "$(plist_value CFBundleShortVersionString)" == "$MARKETING_VERSION" ]] || { echo "The archive has the wrong marketing version" >&2; exit 1; }
    [[ "$(plist_value CFBundleVersion)" == "$CURRENT_PROJECT_VERSION" ]] || { echo "The archive has the wrong build number" >&2; exit 1; }
    lipo -verify_arch arm64 "$app/Lampcord"
    xcrun vtool -show-build "$app/Lampcord" | grep -q 'platform IOS' \
      || { echo "The binary is not built for iOS" >&2; exit 1; }

    staging="$(mktemp -d)"
    mkdir -p "$staging/Payload"
    cp -R "$app" "$staging/Payload/Lampcord.app"
    ipa="$DIST_DIR/Lampcord.ipa"
    (cd "$staging" && zip -qr "$OLDPWD/$ipa" Payload)
    unzip -tq "$ipa"
    echo "Wrote $ipa"
    ;;

  sim)
    destination="dist/Lampcord.app"
    xcodebuild build "${COMMON_SETTINGS[@]}" "${UNSIGNED_SETTINGS[@]}" \
      -destination 'generic/platform=iOS Simulator'

    built="$DERIVED_DATA_DIR/Build/Products/Debug-iphonesimulator/Lampcord.app"
    [[ -d "$built" ]] || { echo "xcodebuild did not produce a simulator build" >&2; exit 1; }
    rm -rf "$destination"
    cp -R "$built" "$destination"
    echo "Wrote $destination"

    if [[ "$run_after_build" == "1" ]]; then
      xcrun simctl boot "iPhone 15" 2> /dev/null || true
      xcrun simctl install booted "$destination"
      xcrun simctl launch booted "$BUNDLE_ID"
    fi
    ;;

  device)
    xcodebuild build "${COMMON_SETTINGS[@]}" "${UNSIGNED_SETTINGS[@]}" \
      -destination 'generic/platform=iOS'
    echo "Device build complete in $DERIVED_DATA_DIR"
    ;;
esac
