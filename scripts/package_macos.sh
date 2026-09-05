#!/usr/bin/env bash
# package_macos.sh — build, bundle, sign, and package eDirStat for macOS.
#
# Channels:
#   (default) / --devid  Developer ID → notarize → zip    (itch.io / direct download, unsandboxed)
#   --skip-notarize      Developer ID, sign only          (quick local iteration)
#   --ad-hoc / --unsigned Ad-hoc sign (-), skip notary    (dev / CI without secrets)
#   --appstore           App Store / TestFlight → .pkg    (sandboxed, signed for Transporter)
#
# Examples:
#   ./scripts/package_macos.sh                          # production itch.io release build
#   ./scripts/package_macos.sh --skip-notarize          # local signed test (skip notarize wait)
#   ./scripts/package_macos.sh --ad-hoc                 # local dev or CI build (no secrets needed)
#   ./scripts/package_macos.sh --appstore --build 3     # Mac App Store upload (.pkg)
#
# One-time prereqs:
#   xcode-select --install
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   xcrun notarytool store-credentials "notary" --apple-id ... --team-id ... --password ...
#   App Store mode also expects: sandbox.entitlements + the Mac App Store
#   provisioning profile (default path: appstore.provisionprofile)

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$REPO_ROOT" ]]; then
    SCRIPT_PATH="$(realpath "$0" 2>/dev/null || readlink -f "$0" 2>/dev/null || echo "$0")"
    REPO_ROOT="$(cd "$(dirname "$SCRIPT_PATH")/.." && pwd -P)"
fi
cd "$REPO_ROOT"

# ---------- Config (override via environment) ----------
APP_NAME="eDirStat"
BINARY_NAME="edirstat"
BUNDLE_ID="com.edirstat.app"
TEAM_ID="B2QGXRL5VZ"
ICON_SOURCE="crates/edirstat/assets/img/icon_512x.png"   # largest master available
CATEGORY="public.app-category.utilities"
TARGET="aarch64-apple-darwin"
NOTARY_PROFILE="${NOTARY_PROFILE:-notary}"
APPSTORE_PROFILE="${APPSTORE_PROFILE:-appstore.provisionprofile}"
BUILD_NUMBER="${BUILD_NUMBER:-1}"

MODE="devid"
SKIP_NOTARIZE=0
AD_HOC=0
ENTITLEMENTS="${ENTITLEMENTS:-}"
NO_DEFAULT_FEATURES="${NO_DEFAULT_FEATURES:-0}"

# App Store requires deployment target >= 12.0 for arm64-only builds,
# and rustc reads this at link time
export MACOSX_DEPLOYMENT_TARGET="12.0"
MIN_MACOS="$MACOSX_DEPLOYMENT_TARGET"

# ---------- Args ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)               TARGET="$2"; shift 2 ;;
    --skip-notarize)        SKIP_NOTARIZE=1; shift ;;
    --ad-hoc)               AD_HOC=1; SKIP_NOTARIZE=1; shift ;;
    --unsigned)             AD_HOC=1; SKIP_NOTARIZE=1; shift ;;
    --appstore)             MODE="appstore"; shift ;;
    --devid)                MODE="devid"; shift ;;
    --build)                BUILD_NUMBER="$2"; shift 2 ;;
    --entitlements)         ENTITLEMENTS="$2"; shift 2 ;;
    --no-default-features|--no-online|--offline) NO_DEFAULT_FEATURES=1; shift ;;
    --online)               NO_DEFAULT_FEATURES=0; shift ;;
    -h|--help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --target <triple>     Target triple (default: aarch64-apple-darwin)"
      echo "  --devid               Developer ID channel (itch.io/direct, unsandboxed, notarized) [default]"
      echo "  --appstore            Mac App Store channel (sandboxed, signed .pkg, no default features)"
      echo "  --ad-hoc, --unsigned  Ad-hoc sign (-), skip notarization (for local dev / CI)"
      echo "  --skip-notarize       Sign with Developer ID, skip notarytool"
      echo "  --build <num>         App Store CFBundleVersion build number (default: 1)"
      echo "  --entitlements <file> Override entitlements plist"
      echo "  --no-default-features, --no-online"
      echo "                        Build without default features (omits GitHub update check)"
      echo "  -h, --help            Show this help message"
      exit 0
      ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
done

# ---------- Select identities ----------
find_identity() {  # $1 = grep pattern, $2 = policy flags
  security find-identity -v $2 2>/dev/null | grep "$1" | head -1 | sed -E 's/.*"(.*)"/\1/' || true
}

if [[ "$AD_HOC" -eq 1 ]]; then
  CODESIGN_IDENTITY="-"
  echo "==> Mode:             ad-hoc development (unnotarized)"
  echo "==> Signing identity: ad-hoc (-)"
elif [[ "$MODE" == "appstore" ]]; then
  if [[ -z "$ENTITLEMENTS" ]]; then
    if [[ -f "sandbox.entitlements" ]]; then
      ENTITLEMENTS="sandbox.entitlements"
    elif [[ -f "entitlements.plist" ]]; then
      ENTITLEMENTS="entitlements.plist"
    else
      ENTITLEMENTS="sandbox.entitlements"
      cat > "$ENTITLEMENTS" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.application-identifier</key>
    <string>${TEAM_ID}.${BUNDLE_ID}</string>
</dict>
</plist>
EOF
    fi
  fi
  CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-$(find_identity "Apple Distribution" "-p codesigning")}"
  INSTALLER_IDENTITY="${INSTALLER_IDENTITY:-$(find_identity -i "Mac Installer Distribution\|3rd Party Mac Developer Installer" "")}"
  [[ -n "$CODESIGN_IDENTITY" ]]  || { echo "ERROR: no 'Apple Distribution' identity." >&2; exit 1; }
  [[ -n "$INSTALLER_IDENTITY" ]] || { echo "ERROR: no installer identity (Mac Installer Distribution)." >&2; exit 1; }
  [[ -f "$APPSTORE_PROFILE" ]]   || { echo "ERROR: provisioning profile not found at $APPSTORE_PROFILE" >&2; exit 1; }
  echo "==> Mode:             Mac App Store (.pkg)"
  echo "==> App identity:     $CODESIGN_IDENTITY"
  echo "==> Installer ident:  $INSTALLER_IDENTITY"
  echo "==> Entitlements:     $ENTITLEMENTS"
else
  # Default: Developer ID (unsandboxed for itch.io / direct download)
  CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-$(find_identity "Developer ID Application" "-p codesigning")}"
  [[ -n "$CODESIGN_IDENTITY" ]] || {
    echo "ERROR: no 'Developer ID Application' identity found in keychain." >&2
    echo "       Pass --ad-hoc for local development / CI builds without certificates." >&2
    exit 1
  }
  echo "==> Mode:             Developer ID / itch.io (unsandboxed)"
  echo "==> Signing identity: $CODESIGN_IDENTITY"
fi

echo "==> Target:           $TARGET (min macOS $MIN_MACOS)"

# ---------- Build ----------
CARGO_BUILD_ARGS=(-p "$BINARY_NAME")
if [[ "$MODE" == "appstore" ]]; then
  echo "==> Configuring build for Mac App Store (sandboxed)"
  export EDIRSTAT_MACOS_APPSTORE=1
  export EDIRSTAT_APP_SANDBOX=1
  NO_DEFAULT_FEATURES=1
fi

if [[ "$NO_DEFAULT_FEATURES" -eq 1 ]]; then
  echo "==> Building with --no-default-features (online features disabled)"
  CARGO_BUILD_ARGS+=(--no-default-features)
fi

cargo build --release --target "$TARGET" "${CARGO_BUILD_ARGS[@]}"

VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
echo "==> Version: $VERSION (build $BUILD_NUMBER)"

# ---------- Assemble .app bundle ----------
APP_DIR="staging/$APP_NAME.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cp "target/$TARGET/release/$BINARY_NAME" "$APP_DIR/Contents/MacOS/$BINARY_NAME"

if command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
  echo "==> Compiling icon (full retina ladder)"
  ICONSET="$APP_NAME.iconset"
  rm -rf "$ICONSET"; mkdir -p "$ICONSET"
  sips -z 16 16     "$ICON_SOURCE" --out "$ICONSET/icon_16x16.png"      >/dev/null
  sips -z 32 32     "$ICON_SOURCE" --out "$ICONSET/icon_16x16@2x.png"   >/dev/null
  sips -z 32 32     "$ICON_SOURCE" --out "$ICONSET/icon_32x32.png"      >/dev/null
  sips -z 64 64     "$ICON_SOURCE" --out "$ICONSET/icon_32x32@2x.png"   >/dev/null
  sips -z 128 128   "$ICON_SOURCE" --out "$ICONSET/icon_128x128.png"    >/dev/null
  sips -z 256 256   "$ICON_SOURCE" --out "$ICONSET/icon_128x128@2x.png" >/dev/null
  sips -z 256 256   "$ICON_SOURCE" --out "$ICONSET/icon_256x256.png"    >/dev/null
  sips -z 512 512   "$ICON_SOURCE" --out "$ICONSET/icon_256x256@2x.png" >/dev/null
  sips -z 512 512   "$ICON_SOURCE" --out "$ICONSET/icon_512x512.png"    >/dev/null
  sips -z 1024 1024 "$ICON_SOURCE" --out "$ICONSET/icon_512x512@2x.png" >/dev/null
  iconutil -c icns "$ICONSET" --output "$APP_DIR/Contents/Resources/icon.icns"
  rm -rf "$ICONSET"
fi

# App Store mode: embed the provisioning profile BEFORE signing
if [[ "$MODE" == "appstore" ]]; then
  cp "$APPSTORE_PROFILE" "$APP_DIR/Contents/embedded.provisionprofile"
fi

cat > "$APP_DIR/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>          <string>$BINARY_NAME</string>
    <key>CFBundleIconFile</key>            <string>icon.icns</string>
    <key>CFBundleIdentifier</key>          <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>                <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>         <string>APPL</string>
    <key>CFBundleShortVersionString</key>  <string>$VERSION</string>
    <key>CFBundleVersion</key>             <string>$BUILD_NUMBER</string>
    <key>LSMinimumSystemVersion</key>      <string>$MIN_MACOS</string>
    <key>LSApplicationCategoryType</key>   <string>$CATEGORY</string>
    <key>ITSAppUsesNonExemptEncryption</key> <false/>
</dict>
</plist>
EOF

# ---------- Sign (inside-out: executable, then bundle) ----------
if [[ "$AD_HOC" -eq 1 ]]; then
  echo "==> codesign (ad-hoc)"
  codesign --force --options runtime ${ENTITLEMENTS:+--entitlements "$ENTITLEMENTS"} \
    --sign - "$APP_DIR/Contents/MacOS/$BINARY_NAME"
  codesign --force --options runtime ${ENTITLEMENTS:+--entitlements "$ENTITLEMENTS"} \
    --sign - "$APP_DIR"
else
  echo "==> codesign${ENTITLEMENTS:+ (entitlements: $ENTITLEMENTS)}"
  codesign --force --options runtime --timestamp ${ENTITLEMENTS:+--entitlements "$ENTITLEMENTS"} \
    --sign "$CODESIGN_IDENTITY" \
    "$APP_DIR/Contents/MacOS/$BINARY_NAME"
  codesign --force --options runtime --timestamp ${ENTITLEMENTS:+--entitlements "$ENTITLEMENTS"} \
    --sign "$CODESIGN_IDENTITY" \
    "$APP_DIR"
fi

# ---------- App Store channel: package and stop ----------
if [[ "$MODE" == "appstore" ]]; then
  echo "==> verifying entitlements include application-identifier"
  codesign -d --entitlements :- "$APP_DIR" | grep -q "com.apple.application-identifier" \
    || { echo "ERROR: entitlements missing com.apple.application-identifier (TestFlight 90886)." >&2; exit 1; }

  PKG="${BINARY_NAME}-appstore-${VERSION}-${BUILD_NUMBER}.pkg"
  echo "==> productbuild ($INSTALLER_IDENTITY)"
  productbuild --component "$APP_DIR" /Applications \
    --sign "$INSTALLER_IDENTITY" \
    "$PKG"
  pkgutil --check-signature "$PKG"
  echo "==> Done: $PKG  → drag into Transporter (build $BUILD_NUMBER must be unique per upload)"
  exit 0
fi

# ---------- Developer ID & Ad-Hoc channels ----------
OUT_ZIP="${BINARY_NAME}-macos-${TARGET%%-*}-${VERSION}.zip"

if [[ "$AD_HOC" -eq 1 ]]; then
  ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"
  echo "==> Done (ad-hoc, for development/CI): $OUT_ZIP"
  exit 0
fi

if [[ "$SKIP_NOTARIZE" -eq 1 ]]; then
  echo "==> Skipping notarization (--skip-notarize)"
  ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"
  echo "==> Done (signed, NOT notarized): $OUT_ZIP"
  exit 0
fi

# ---------- Notarize ----------
echo "==> notarytool submit (profile: $NOTARY_PROFILE)"
rm -f submission.zip
ditto -c -k --keepParent "$APP_DIR" submission.zip
xcrun notarytool submit submission.zip --keychain-profile "$NOTARY_PROFILE" --wait
rm -f submission.zip

# ---------- Staple + final package ----------
echo "==> stapling ticket"
xcrun stapler staple "$APP_DIR"

ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"

# ---------- Verify ----------
echo "==> Gatekeeper check:"
spctl -a -vvv "$APP_DIR"

echo "==> Done: $OUT_ZIP"
