#!/usr/bin/env bash
# package_macos.sh — build, bundle, sign, and package eDirStat for macOS.
#
# Channels:
#   (default) / --devid  Developer ID → notarize → zip    (itch.io / direct download, unsandboxed)
#   --skip-notarize      Developer ID, sign only          (quick local iteration)
#   --ad-hoc / --unsigned Ad-hoc sign (-), skip notary    (dev / CI without secrets)
#   --appstore           App Store / TestFlight → .pkg    (sandboxed, signed for Transporter)
#   --validate [path]    Pre-flight validation on bundle/package (or test an existing target)
#
# Examples:
#   ./scripts/package_macos.sh                          # production itch.io release build
#   ./scripts/package_macos.sh --skip-notarize          # local signed test (skip notarize wait)
#   ./scripts/package_macos.sh --ad-hoc                 # local dev or CI build (no secrets needed)
#   ./scripts/package_macos.sh --appstore --build 3     # Mac App Store upload (.pkg)
#   ./scripts/package_macos.sh --appstore --validate    # build App Store .pkg and run validation
#   ./scripts/package_macos.sh --validate staging/eDirStat.app # validate existing .app bundle
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
VALIDATE=0
VALIDATE_TARGET=""

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
    --validate)
      VALIDATE=1
      if [[ $# -ge 2 && "$2" != --* ]]; then
        VALIDATE_TARGET="$2"
        shift 2
      else
        shift
      fi
      ;;
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
      echo "  --validate [path]     Run pre-flight validation checks (on built output or specified target)"
      echo "  -h, --help            Show this help message"
      exit 0
      ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
done

# ---------- Validation functions ----------
validate_bundle() {
  local target_app="$1"
  echo "==> [Validate] Inspecting application bundle: $target_app"

  if [[ ! -d "$target_app" ]]; then
    echo "ERROR: Bundle not found at $target_app" >&2
    return 1
  fi

  # 1. Info.plist structure & syntax
  local plist_path="$target_app/Contents/Info.plist"
  if [[ -f "$plist_path" ]]; then
    if command -v plutil >/dev/null 2>&1; then
      echo "  ✓ Checking Info.plist syntax (plutil)..."
      plutil -lint "$plist_path" >/dev/null || {
        echo "ERROR: Info.plist failed plutil lint check." >&2
        return 1
      }
    else
      echo "  ✓ Info.plist exists."
    fi
  else
    echo "ERROR: Info.plist is missing from $target_app" >&2
    return 1
  fi

  # 2. Main Mach-O binary check
  local bin_path="$target_app/Contents/MacOS/$BINARY_NAME"
  if [[ ! -f "$bin_path" ]]; then
    bin_path="$(find "$target_app/Contents/MacOS" -type f 2>/dev/null | head -1 || true)"
  fi
  if [[ -z "$bin_path" || ! -f "$bin_path" ]]; then
    echo "ERROR: Missing executable in $target_app/Contents/MacOS" >&2
    return 1
  fi
  if [[ ! -x "$bin_path" ]]; then
    echo "ERROR: Binary at $bin_path is not executable" >&2
    return 1
  fi
  if command -v file >/dev/null 2>&1; then
    echo "  ✓ Binary format: $(file -b "$bin_path")"
  fi

  # 3. Codesign deep & strict verification
  if command -v codesign >/dev/null 2>&1; then
    echo "  ✓ Verifying code signature (deep, strict)..."
    codesign --verify --deep --strict --verbose=2 "$target_app" 2>&1 | sed 's/^/    /' || {
      echo "ERROR: Code signature verification failed on $target_app" >&2
      return 1
    }

    # 4. Entitlements inspection
    echo "  ✓ Inspecting embedded entitlements..."
    local ent_dump
    ent_dump="$(codesign -d --entitlements :- "$target_app" 2>/dev/null || true)"
    if [[ "$MODE" == "appstore" ]] || echo "$ent_dump" | grep -q "com.apple.security.app-sandbox"; then
      echo "$ent_dump" | grep -q "com.apple.security.app-sandbox" || {
        echo "ERROR: Missing com.apple.security.app-sandbox entitlement for sandboxed build." >&2
        return 1
      }
      echo "$ent_dump" | grep -q "com.apple.application-identifier" || {
        echo "ERROR: Missing com.apple.application-identifier entitlement (required for App Store / TestFlight)." >&2
        return 1
      }
      echo "  ✓ Sandboxing & application-identifier verified."
      if [[ ! -s "$target_app/Contents/embedded.provisionprofile" ]]; then
        echo "  ! Note: Contents/embedded.provisionprofile is empty or not present."
      else
        echo "  ✓ Embedded provisioning profile present."
      fi
    fi
  fi

  # 5. Gatekeeper assessment
  if command -v spctl >/dev/null 2>&1; then
    echo "  ✓ Checking Gatekeeper assessment (spctl)..."
    if [[ "$AD_HOC" -eq 1 ]]; then
      echo "    (Ad-hoc signed builds are bypassed from Gatekeeper assessment)"
    else
      spctl --assess --type exec --verbose "$target_app" 2>&1 | sed 's/^/    /' || {
        if [[ "$SKIP_NOTARIZE" -eq 1 ]]; then
          echo "    (Gatekeeper assessment reported un-notarized as expected for --skip-notarize)"
        else
          echo "WARNING: Gatekeeper assessment reported issues for $target_app" >&2
        fi
      }
    fi
  fi

  echo "==> [Validate] Application bundle validation passed: $target_app"
}

validate_pkg() {
  local target_pkg="$1"
  echo "==> [Validate] Inspecting installer package: $target_pkg"

  if [[ ! -f "$target_pkg" ]]; then
    echo "ERROR: Package file not found: $target_pkg" >&2
    return 1
  fi

  # 1. Package signature check
  if command -v pkgutil >/dev/null 2>&1; then
    echo "  ✓ Checking package signature (pkgutil)..."
    pkgutil --check-signature "$target_pkg" 2>&1 | sed 's/^/    /' || {
      echo "ERROR: Package signature check failed on $target_pkg" >&2
      return 1
    }
  fi

  # 2. Gatekeeper install check
  if command -v spctl >/dev/null 2>&1; then
    echo "  ✓ Checking installer Gatekeeper assessment (spctl)..."
    spctl --assess --type install --verbose "$target_pkg" 2>&1 | sed 's/^/    /' || {
      echo "WARNING: Gatekeeper install assessment reported issues for $target_pkg" >&2
    }
  fi

  # 3. Remote App Store Connect pre-flight validation (xcrun altool)
  if command -v xcrun >/dev/null 2>&1; then
    local altool_ran=0
    local api_key="${ALTOOL_KEY_ID:-${ALTOOL_API_KEY:-}}"
    local api_issuer="${ALTOOL_ISSUER_ID:-${ALTOOL_API_ISSUER:-}}"
    local altool_user="${ALTOOL_USER:-${APPLE_ID:-}}"
    local altool_password="${ALTOOL_PASSWORD:-${APPLE_PASSWORD:-}}"

    if [[ -n "$api_key" && -n "$api_issuer" ]]; then
      echo "  ✓ Running App Store Connect validation via xcrun altool (API Key)..."
      xcrun altool --validate-app -f "$target_pkg" -t osx \
        --apiKey "$api_key" --apiIssuer "$api_issuer"
      altool_ran=1
    elif [[ -n "$altool_user" && -n "$altool_password" ]]; then
      echo "  ✓ Running App Store Connect validation via xcrun altool (Credentials)..."
      xcrun altool --validate-app -f "$target_pkg" -t osx \
        -u "$altool_user" -p "$altool_password"
      altool_ran=1
    fi

    if [[ "$altool_ran" -eq 0 ]]; then
      echo "  ℹ Note: Remote App Store Connect validation skipped."
      echo "    To enable remote pre-flight validation against App Store Connect, set:"
      echo "    ALTOOL_KEY_ID + ALTOOL_ISSUER_ID (App Store Connect API key), or"
      echo "    ALTOOL_USER + ALTOOL_PASSWORD (Apple ID credentials)."
    fi
  fi

  echo "==> [Validate] Package validation passed: $target_pkg"
}

# ---------- Standalone validation dispatch ----------
if [[ -n "$VALIDATE_TARGET" ]]; then
  if [[ "$VALIDATE_TARGET" == *.pkg ]]; then
    validate_pkg "$VALIDATE_TARGET"
  elif [[ "$VALIDATE_TARGET" == *.app || -d "$VALIDATE_TARGET/Contents" ]]; then
    validate_bundle "$VALIDATE_TARGET"
  else
    echo "ERROR: Unrecognized target to validate: $VALIDATE_TARGET (expected .app or .pkg)" >&2
    exit 1
  fi
  exit 0
fi

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

  if [[ "$VALIDATE" -eq 1 ]]; then
    validate_bundle "$APP_DIR"
    validate_pkg "$PKG"
  fi

  echo "==> Done: $PKG  → drag into Transporter (build $BUILD_NUMBER must be unique per upload)"
  exit 0
fi

# ---------- Developer ID & Ad-Hoc channels ----------
OUT_ZIP="${BINARY_NAME}-macos-${TARGET%%-*}-${VERSION}.zip"

if [[ "$AD_HOC" -eq 1 ]]; then
  ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"
  if [[ "$VALIDATE" -eq 1 ]]; then
    validate_bundle "$APP_DIR"
  fi
  echo "==> Done (ad-hoc, for development/CI): $OUT_ZIP"
  exit 0
fi

if [[ "$SKIP_NOTARIZE" -eq 1 ]]; then
  echo "==> Skipping notarization (--skip-notarize)"
  ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"
  if [[ "$VALIDATE" -eq 1 ]]; then
    validate_bundle "$APP_DIR"
  fi
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

if [[ "$VALIDATE" -eq 1 ]]; then
  validate_bundle "$APP_DIR"
fi

echo "==> Done: $OUT_ZIP"
