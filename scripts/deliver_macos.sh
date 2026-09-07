#!/usr/bin/env bash
# deliver_macos.sh — Validate, upload, and submit Mac App Store packages (.pkg) to App Store Connect.
#
# This script handles:
#   1. Strict credential security (umask 077, ephemeral key isolation, GitHub Actions log masking, trap cleanup).
#   2. Pre-flight signature check and xcrun altool validation against Apple ingest servers.
#   3. Package upload via xcrun altool (--upload-package / --upload-app).
#   4. Build processing polling and automated submission to App Review via App Store Connect REST API.
#
# Usage:
#   ./scripts/deliver_macos.sh [options]
#
# Options:
#   --pkg <file>            Path to .pkg file (auto-detects edirstat-appstore-*.pkg if omitted)
#   --key-id <id>           App Store Connect API Key ID (or $APPLE_API_KEY_ID)
#   --issuer-id <uuid>      App Store Connect Issuer ID (or $APPLE_API_ISSUER)
#   --api-key <val>         Private key .p8 path, raw PEM string, or base64 (or $APPLE_API_KEY)
#   --bundle-id <id>        Bundle ID (default: com.edirstat.app)
#   --build-number <num>    Override build number (CFBundleVersion)
#   --version <ver>         Override release version (CFBundleShortVersionString)
#   --skip-submit           Upload to App Store Connect only (do not submit for review)
#   --submit-only           Skip upload, only poll and submit existing build for review
#   --validate-only         Run altool pre-flight validation only (no upload, no submit)
#   --wait-timeout <sec>    Max seconds to wait for Apple build processing (default: 1800)
#   --poll-interval <sec>   Seconds between build processing status checks (default: 30)
#   --dry-run               Check files and credentials without performing network operations
#   -h, --help              Show this help message
#
# Security & Hygiene:
#   - Writes all temporary keys with umask 077 in an isolated temporary directory.
#   - Installs an EXIT/INT/TERM trap to ensure all private keys and directories are purged.
#   - Automatically invokes ::add-mask:: on secrets when running in GitHub Actions.
#   - Uses standard library Python + OpenSSL for ES256 JWT generation (0 external dependencies).

set -euo pipefail

# Enforce strict umask for all created files
umask 077

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$REPO_ROOT" ]]; then
  SCRIPT_PATH="$(realpath "$0" 2>/dev/null || readlink -f "$0" 2>/dev/null || echo "$0")"
  REPO_ROOT="$(cd "$(dirname "$SCRIPT_PATH")/.." && pwd -P)"
fi
cd "$REPO_ROOT"

# ---------- Defaults & Config ----------
PKG_PATH=""
KEY_ID="${APPLE_API_KEY_ID:-}"
ISSUER_ID="${APPLE_API_ISSUER:-}"
API_KEY_RAW="${APPLE_API_KEY:-}"
BUNDLE_ID="com.edirstat.app"
BUILD_NUMBER=""
APP_VERSION=""
SKIP_SUBMIT=0
SUBMIT_ONLY=0
VALIDATE_ONLY=0
DRY_RUN=0
WAIT_TIMEOUT=1800
POLL_INTERVAL=30

# ---------- Args Parsing ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --pkg)                  PKG_PATH="$2"; shift 2 ;;
    --key-id)               KEY_ID="$2"; shift 2 ;;
    --issuer-id)            ISSUER_ID="$2"; shift 2 ;;
    --api-key)              API_KEY_RAW="$2"; shift 2 ;;
    --bundle-id)            BUNDLE_ID="$2"; shift 2 ;;
    --build-number)         BUILD_NUMBER="$2"; shift 2 ;;
    --version)              APP_VERSION="$2"; shift 2 ;;
    --skip-submit|--upload-only) SKIP_SUBMIT=1; shift ;;
    --submit-only)          SUBMIT_ONLY=1; shift ;;
    --validate-only)        VALIDATE_ONLY=1; shift ;;
    --wait-timeout)         WAIT_TIMEOUT="$2"; shift 2 ;;
    --poll-interval)        POLL_INTERVAL="$2"; shift 2 ;;
    --dry-run)              DRY_RUN=1; shift ;;
    -h|--help)
      grep '^#   ' "$0" | sed 's/^#   //'
      exit 0
      ;;
    *)
      echo "ERROR: Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

# ---------- Auto-detect .pkg if not specified ----------
if [[ -z "$PKG_PATH" && "$SUBMIT_ONLY" -eq 0 ]]; then
  # Look in repo root and staging dirs
  PKG_PATH="$(find . -maxdepth 2 -type f -name "edirstat-appstore-*.pkg" 2>/dev/null | sort -V | tail -1 || true)"
  if [[ -z "$PKG_PATH" ]]; then
    PKG_PATH="$(find . -maxdepth 2 -type f -name "*.pkg" 2>/dev/null | sort -V | tail -1 || true)"
  fi
fi

if [[ "$SUBMIT_ONLY" -eq 0 ]]; then
  if [[ -z "$PKG_PATH" || ! -f "$PKG_PATH" ]]; then
    echo "ERROR: No .pkg package found. Specify --pkg <path> or build one via ./scripts/package_macos.sh --appstore." >&2
    exit 1
  fi
  PKG_PATH="$(cd "$(dirname "$PKG_PATH")" && pwd -P)/$(basename "$PKG_PATH")"
  echo "==> Target package: $PKG_PATH"
fi

# ---------- Extract Version & Build Number from package filename or Cargo.toml ----------
if [[ -z "$APP_VERSION" ]]; then
  if [[ -n "$PKG_PATH" && "$(basename "$PKG_PATH")" =~ ([0-9]+\.[0-9]+\.[0-9]+) ]]; then
    APP_VERSION="${BASH_REMATCH[1]}"
  else
    APP_VERSION="$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/' || true)"
  fi
fi

if [[ -z "$BUILD_NUMBER" ]]; then
  if [[ -n "$PKG_PATH" && "$(basename "$PKG_PATH")" =~ -([0-9]+)\.pkg$ ]]; then
    BUILD_NUMBER="${BASH_REMATCH[1]}"
  else
    BUILD_NUMBER="1"
  fi
fi

echo "==> App Version:    $APP_VERSION"
echo "==> Build Number:   $BUILD_NUMBER"
echo "==> Bundle ID:      $BUNDLE_ID"

# ---------- Validate Required Credentials ----------
if [[ -z "$KEY_ID" ]]; then
  echo "ERROR: Missing App Store Connect Key ID. Provide --key-id or set APPLE_API_KEY_ID." >&2
  exit 1
fi

if [[ -z "$ISSUER_ID" ]]; then
  echo "ERROR: Missing App Store Connect Issuer ID. Provide --issuer-id or set APPLE_API_ISSUER." >&2
  exit 1
fi

if [[ -z "$API_KEY_RAW" ]]; then
  echo "ERROR: Missing App Store Connect API Private Key. Provide --api-key or set APPLE_API_KEY." >&2
  exit 1
fi

# Mask secrets in GitHub Actions if present
if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  echo "::add-mask::$KEY_ID"
  echo "::add-mask::$ISSUER_ID"
fi

# ---------- Setup Ephemeral Private Key Storage with Trap Cleanup ----------
TMP_KEY_DIR="$(mktemp -d "${TMPDIR:-/tmp}/edirstat_deliver.XXXXXX")"
chmod 700 "$TMP_KEY_DIR"
KEY_FILE="$TMP_KEY_DIR/AuthKey_${KEY_ID}.p8"

# Paths where xcrun altool expects the API key
ALTOOL_APPSTORE_DIR="$HOME/.appstoreconnect/private_keys"
ALTOOL_LEGACY_DIR="$HOME/private_keys"
ALTOOL_LINKED_APPSTORE=0
ALTOOL_LINKED_LEGACY=0

cleanup() {
  local exit_code=$?
  if [[ "$ALTOOL_LINKED_APPSTORE" -eq 1 ]]; then
    rm -f "$ALTOOL_APPSTORE_DIR/AuthKey_${KEY_ID}.p8" 2>/dev/null || true
  fi
  if [[ "$ALTOOL_LINKED_LEGACY" -eq 1 ]]; then
    rm -f "$ALTOOL_LEGACY_DIR/AuthKey_${KEY_ID}.p8" 2>/dev/null || true
  fi
  if [[ -d "$TMP_KEY_DIR" ]]; then
    rm -rf "$TMP_KEY_DIR" 2>/dev/null || true
  fi
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

# Materialize .p8 key file securely
if [[ -f "$API_KEY_RAW" ]]; then
  cp "$API_KEY_RAW" "$KEY_FILE"
elif [[ "$API_KEY_RAW" =~ ^[A-Za-z0-9+/=[:space:]]+$ && ! "$API_KEY_RAW" =~ "BEGIN" ]]; then
  # Base64 encoded
  echo "$API_KEY_RAW" | base64 -d > "$KEY_FILE"
else
  # Raw PEM content
  echo "$API_KEY_RAW" > "$KEY_FILE"
fi
chmod 600 "$KEY_FILE"

# Place in altool discovery locations
mkdir -p "$ALTOOL_APPSTORE_DIR" "$ALTOOL_LEGACY_DIR"
chmod 700 "$ALTOOL_APPSTORE_DIR" "$ALTOOL_LEGACY_DIR"

if [[ ! -f "$ALTOOL_APPSTORE_DIR/AuthKey_${KEY_ID}.p8" ]]; then
  cp "$KEY_FILE" "$ALTOOL_APPSTORE_DIR/AuthKey_${KEY_ID}.p8"
  chmod 600 "$ALTOOL_APPSTORE_DIR/AuthKey_${KEY_ID}.p8"
  ALTOOL_LINKED_APPSTORE=1
fi

if [[ ! -f "$ALTOOL_LEGACY_DIR/AuthKey_${KEY_ID}.p8" ]]; then
  cp "$KEY_FILE" "$ALTOOL_LEGACY_DIR/AuthKey_${KEY_ID}.p8"
  chmod 600 "$ALTOOL_LEGACY_DIR/AuthKey_${KEY_ID}.p8"
  ALTOOL_LINKED_LEGACY=1
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "==> [Dry Run] Credentials materialized and validated successfully."
  echo "==> [Dry Run] Ephemeral key path: $KEY_FILE"
  echo "==> [Dry Run] Exiting without performing network upload or submission."
  exit 0
fi

# ---------- Pre-flight Package Signature Verification ----------
if [[ "$SUBMIT_ONLY" -eq 0 ]]; then
  echo "==> Verifying package signature..."
  if command -v pkgutil >/dev/null 2>&1; then
    pkgutil --check-signature "$PKG_PATH" | sed 's/^/    /' || {
      echo "ERROR: Package signature verification failed on $PKG_PATH" >&2
      exit 1
    }
  else
    echo "  (pkgutil not available, skipping local signature check)"
  fi

  # ---------- Pre-flight Validation via xcrun altool ----------
  echo "==> Running remote package pre-flight validation via xcrun altool..."
  if command -v xcrun >/dev/null 2>&1; then
    set +e
    VAL_OUT="$(xcrun altool --validate-package -f "$PKG_PATH" -t osx --apiKey "$KEY_ID" --apiIssuer "$ISSUER_ID" 2>&1)"
    VAL_CODE=$?
    if [[ $VAL_CODE -ne 0 ]]; then
      # Try fallback to --validate-app
      VAL_OUT="$(xcrun altool --validate-app -f "$PKG_PATH" -t osx --apiKey "$KEY_ID" --apiIssuer "$ISSUER_ID" 2>&1)"
      VAL_CODE=$?
    fi
    set -e

    if [[ $VAL_CODE -ne 0 ]]; then
      echo "ERROR: Remote validation failed via xcrun altool:" >&2
      echo "$VAL_OUT" >&2
      exit 1
    fi
    echo "  ✓ Pre-flight package validation succeeded."
  else
    echo "WARNING: xcrun not found; remote altool validation skipped."
  fi

  if [[ "$VALIDATE_ONLY" -eq 1 ]]; then
    echo "==> Validation-only requested. Done!"
    exit 0
  fi

  # ---------- Upload Package via xcrun altool ----------
  echo "==> Uploading package to App Store Connect..."
  set +e
  UPLOAD_OUT="$(xcrun altool --upload-package -f "$PKG_PATH" -t osx --apiKey "$KEY_ID" --apiIssuer "$ISSUER_ID" 2>&1)"
  UPLOAD_CODE=$?
  if [[ $UPLOAD_CODE -ne 0 ]]; then
    # Fallback to --upload-app
    UPLOAD_OUT="$(xcrun altool --upload-app -f "$PKG_PATH" -t osx --apiKey "$KEY_ID" --apiIssuer "$ISSUER_ID" 2>&1)"
    UPLOAD_CODE=$?
  fi
  set -e

  if [[ $UPLOAD_CODE -ne 0 ]]; then
    echo "ERROR: Package upload to App Store Connect failed:" >&2
    echo "$UPLOAD_OUT" >&2
    exit 1
  fi
  echo "$UPLOAD_OUT" | grep -E -i "No errors uploading|Delivery succeeded" || echo "$UPLOAD_OUT"
  echo "==> Package upload successful!"
fi

if [[ "$SKIP_SUBMIT" -eq 1 ]]; then
  echo "==> --skip-submit specified. Package uploaded without submitting for review."
  exit 0
fi

# ---------- Poll Build Processing & Submit for Review via App Store Connect REST API ----------
echo "==> Initiating App Store Connect review submission workflow..."

python3 - <<PY_SCRIPT
import sys
import os
import time
import json
import base64
import subprocess
import urllib.request
import urllib.error

key_id = "$KEY_ID"
issuer_id = "$ISSUER_ID"
key_file = "$KEY_FILE"
bundle_id = "$BUNDLE_ID"
build_number = "$BUILD_NUMBER"
app_version = "$APP_VERSION"
wait_timeout = int("$WAIT_TIMEOUT")
poll_interval = int("$POLL_INTERVAL")

def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("utf-8")

def generate_jwt() -> str:
    """Generate App Store Connect ES256 JWT using OpenSSL (0 external python deps)."""
    header = {"alg": "ES256", "kid": key_id, "typ": "JWT"}
    now = int(time.time())
    payload = {
        "iss": issuer_id,
        "iat": now,
        "exp": now + 1200,  # 20 minutes max valid lifetime
        "aud": "appstoreconnect-v1"
    }
    h_b64 = b64url(json.dumps(header, separators=(",", ":")).encode("utf-8"))
    p_b64 = b64url(json.dumps(payload, separators=(",", ":")).encode("utf-8"))
    signing_input = f"{h_b64}.{p_b64}".encode("utf-8")

    proc = subprocess.run(
        ["openssl", "dgst", "-sha256", "-sign", key_file],
        input=signing_input,
        capture_output=True,
        check=True
    )
    der_sig = proc.stdout

    # Parse ASN.1 DER SEQUENCE to IEEE P1363 (raw r || s 64 bytes)
    if der_sig[0] != 0x30:
        raise ValueError("Invalid ECDSA signature format from openssl")
    pos = 2
    if der_sig[1] & 0x80:
        pos = 2 + (der_sig[1] & 0x7f)
    if der_sig[pos] != 0x02:
        raise ValueError("Invalid ASN.1 INTEGER marker for r")
    r_len = der_sig[pos + 1]
    r = der_sig[pos + 2 : pos + 2 + r_len].lstrip(b"\x00").rjust(32, b"\x00")
    pos = pos + 2 + r_len
    if der_sig[pos] != 0x02:
        raise ValueError("Invalid ASN.1 INTEGER marker for s")
    s_len = der_sig[pos + 1]
    s = der_sig[pos + 2 : pos + 2 + s_len].lstrip(b"\x00").rjust(32, b"\x00")

    raw_sig = r + s
    return f"{h_b64}.{p_b64}.{b64url(raw_sig)}"

def api_request(method: str, path: str, data: dict = None) -> dict:
    url = f"https://api.appstoreconnect.apple.com{path}"
    headers = {
        "Authorization": f"Bearer {generate_jwt()}",
        "Content-Type": "application/json",
        "Accept": "application/json",
        "User-Agent": "eDirStat-Delivery/2.2.0"
    }
    body = json.dumps(data).encode("utf-8") if data else None
    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            resp_body = resp.read()
            if resp_body:
                return json.loads(resp_body.decode("utf-8"))
            return {}
    except urllib.error.HTTPError as e:
        err_content = e.read().decode("utf-8", errors="replace")
        try:
            err_json = json.loads(err_content)
            errors = err_json.get("errors", [])
            err_msgs = [f"{err.get('title', '')}: {err.get('detail', '')}" for err in errors]
            msg = " | ".join(err_msgs)
        except Exception:
            msg = err_content
        print(f"ERROR: App Store Connect API [{method} {path}] failed ({e.code}): {msg}", file=sys.stderr)
        raise RuntimeError(f"API Error {e.code}: {msg}")

print("  ✓ Authenticating with App Store Connect REST API...")

# 1. Fetch App ID
app_resp = api_request("GET", f"/v1/apps?filter[bundleId]={bundle_id}")
app_data = app_resp.get("data", [])
if not app_data:
    raise RuntimeError(f"No app found in App Store Connect with bundle ID: {bundle_id}")
app_id = app_data[0]["id"]
print(f"  ✓ Found App: {app_data[0]['attributes'].get('name', bundle_id)} (ID: {app_id})")

# 2. Wait / Poll for Build Processing
print(f"  ==> Waiting for Build {build_number} to process (timeout: {wait_timeout}s)...")
start_time = time.time()
build_obj = None

while time.time() - start_time < wait_timeout:
    builds_resp = api_request("GET", f"/v1/builds?filter[app]={app_id}&filter[version]={build_number}&sort=-uploadedDate")
    builds = builds_resp.get("data", [])
    if builds:
        candidate = builds[0]
        state = candidate["attributes"].get("processingState")
        if state == "VALID":
            build_obj = candidate
            print(f"  ✓ Build {build_number} processed successfully (State: VALID, Build ID: {build_obj['id']})")
            break
        elif state in ("FAILED", "INVALID"):
            raise RuntimeError(f"Build {build_number} processing failed on App Store Connect (State: {state})")
        else:
            elapsed = int(time.time() - start_time)
            print(f"    ... Build status: {state} ({elapsed}s elapsed, checking again in {poll_interval}s)")
    else:
        elapsed = int(time.time() - start_time)
        print(f"    ... Ingestion pending ({elapsed}s elapsed, checking again in {poll_interval}s)")
    time.sleep(poll_interval)

if not build_obj:
    raise TimeoutError(f"Timed out waiting {wait_timeout}s for Build {build_number} to finish processing.")

build_id = build_obj["id"]

# 3. Locate or Create App Store Version
print(f"  ==> Resolving App Store Version {app_version} (platform: MAC_OS)...")
versions_resp = api_request("GET", f"/v1/apps/{app_id}/appStoreVersions?filter[platform]=MAC_OS")
versions = versions_resp.get("data", [])
target_version = None

for v in versions:
    if v["attributes"].get("versionString") == app_version:
        target_version = v
        break

if not target_version:
    # Check if there is an editable version we can rename or create a new one
    print(f"  ℹ Version {app_version} not found; creating new MAC_OS App Store Version...")
    new_version_payload = {
        "data": {
            "type": "appStoreVersions",
            "attributes": {
                "platform": "MAC_OS",
                "versionString": app_version
            },
            "relationships": {
                "app": {
                    "data": {
                        "type": "apps",
                        "id": app_id
                    }
                }
            }
        }
    }
    create_resp = api_request("POST", "/v1/appStoreVersions", new_version_payload)
    target_version = create_resp["data"]
    print(f"  ✓ Created App Store Version {app_version} (ID: {target_version['id']})")
else:
    print(f"  ✓ Found App Store Version {app_version} (ID: {target_version['id']}, State: {target_version['attributes'].get('appStoreState')})")

version_id = target_version["id"]

# 4. Attach Processed Build to Version
print(f"  ==> Linking Build {build_number} to Version {app_version}...")
patch_build_payload = {
    "data": {
        "type": "appStoreVersions",
        "id": version_id,
        "relationships": {
            "build": {
                "data": {
                    "type": "builds",
                    "id": build_id
                }
            }
        }
    }
}
api_request("PATCH", f"/v1/appStoreVersions/{version_id}", patch_build_payload)
print("  ✓ Build successfully attached to version.")

# 5. Create Review Submission & Submit to App Review
print("  ==> Preparing App Review Submission...")
# Check for existing ready review submissions
sub_resp = api_request("GET", f"/v1/apps/{app_id}/reviewSubmissions?filter[platform]=MAC_OS&filter[state]=READY_FOR_REVIEW")
subs = sub_resp.get("data", [])
review_sub_id = None

if subs:
    review_sub_id = subs[0]["id"]
    print(f"  ✓ Reusing existing active review submission container (ID: {review_sub_id})")
else:
    create_sub_payload = {
        "data": {
            "type": "reviewSubmissions",
            "attributes": {
                "platform": "MAC_OS"
            },
            "relationships": {
                "app": {
                    "data": {
                        "type": "apps",
                        "id": app_id
                    }
                }
            }
        }
    }
    new_sub = api_request("POST", "/v1/reviewSubmissions", create_sub_payload)
    review_sub_id = new_sub["data"]["id"]
    print(f"  ✓ Created new review submission container (ID: {review_sub_id})")

# Add the version item to review submission
print(f"  ==> Adding version {app_version} to review submission...")
try:
    item_payload = {
        "data": {
            "type": "reviewSubmissionItems",
            "relationships": {
                "reviewSubmission": {
                    "data": {
                        "type": "reviewSubmissions",
                        "id": review_sub_id
                    }
                },
                "appStoreVersion": {
                    "data": {
                        "type": "appStoreVersions",
                        "id": version_id
                    }
                }
            }
        }
    }
    api_request("POST", "/v1/reviewSubmissionItems", item_payload)
    print("  ✓ Version item added to review submission.")
except Exception as e:
    # If already added, continue
    print(f"  ℹ Note: Item addition notice: {e}")

# Submit for App Review
print("  ==> Finalizing submission to App Review...")
submit_payload = {
    "data": {
        "type": "reviewSubmissions",
        "id": review_sub_id,
        "attributes": {
            "submitted": True
        }
    }
}
api_request("PATCH", f"/v1/reviewSubmissions/{review_sub_id}", submit_payload)

print(f"\n🎉 Successfully submitted eDirStat {app_version} (Build {build_number}) to Apple for App Review!")
PY_SCRIPT

echo "==> Deliver macOS App Store completed successfully."
