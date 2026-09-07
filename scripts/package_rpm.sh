#!/usr/bin/env bash
# package_rpm.sh — Build a Red Hat / Fedora / openSUSE (.rpm) package for eDirStat.
#
# Usage:
#   ./scripts/package_rpm.sh [options]
#
# Options:
#   --binary <path>     Path to compiled binary (defaults to target/.../edirstat_multivers or target/release/edirstat)
#   --version <ver>     Package version (auto-detected from Cargo.toml if omitted)
#   --build-num <num>   RPM release number (default: 1)
#   --arch <arch>       Target architecture (default: x86_64)
#   --out-dir <dir>     Output directory (default: staging-linux-installer)
#   -h, --help          Show this help message
#
# Artifacts produced:
#   <out-dir>/edirstat-<version>-<build-num>.<arch>.rpm

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$REPO_ROOT" ]]; then
  SCRIPT_PATH="$(realpath "$0" 2>/dev/null || readlink -f "$0" 2>/dev/null || echo "$0")"
  REPO_ROOT="$(cd "$(dirname "$SCRIPT_PATH")/.." && pwd -P)"
fi
cd "$REPO_ROOT"

# ---------- Config & Defaults ----------
BINARY_PATH=""
VERSION=""
BUILD_NUM="1"
ARCH="x86_64"
OUT_DIR="staging-linux-installer"

# ---------- Args Parsing ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary)           BINARY_PATH="$2"; shift 2 ;;
    --version)          VERSION="$2"; shift 2 ;;
    --build-num|--release) BUILD_NUM="$2"; shift 2 ;;
    --arch)             ARCH="$2"; shift 2 ;;
    --out-dir)          OUT_DIR="$2"; shift 2 ;;
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

# ---------- Resolve Version ----------
if [[ -z "$VERSION" ]]; then
  VERSION="$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/' || true)"
fi
VERSION="${VERSION#v}" # strip leading 'v' if present

if [[ -z "$VERSION" ]]; then
  echo "ERROR: Unable to detect package version from Cargo.toml. Pass --version <ver>." >&2
  exit 1
fi

# ---------- Resolve Binary ----------
if [[ -z "$BINARY_PATH" ]]; then
  CANDIDATES=(
    "target/x86_64-unknown-linux-gnu/release/edirstat_multivers"
    "target/release/edirstat"
    "target/x86_64-unknown-linux-gnu/release/edirstat"
  )
  for c in "${CANDIDATES[@]}"; do
    if [[ -f "$c" ]]; then
      BINARY_PATH="$c"
      break
    fi
  done
fi

if [[ -z "$BINARY_PATH" || ! -f "$BINARY_PATH" ]]; then
  echo "ERROR: Target binary not found. Pass --binary <path> or run 'cargo build --release' first." >&2
  exit 1
fi

if ! command -v rpmbuild >/dev/null 2>&1; then
  echo "ERROR: 'rpmbuild' is required to build RPM packages." >&2
  echo "       Install it via: sudo pacman -S rpm-tools (Arch), or sudo apt install rpm (Debian/Ubuntu), or sudo dnf install rpm-build (Fedora)." >&2
  exit 1
fi

echo "==> Packaging eDirStat for Red Hat / Fedora / openSUSE (.rpm)"
echo "  • Version:       $VERSION-$BUILD_NUM"
echo "  • Architecture:  $ARCH"
echo "  • Source binary: $BINARY_PATH"
echo "  • Output dir:    $OUT_DIR"

mkdir -p "$OUT_DIR"

# ---------- Setup Isolated RPM Build Tree ----------
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/edirstat_rpm_build.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

BUILDROOT="$TMP_DIR/BUILDROOT/edirstat-${VERSION}-${BUILD_NUM}.${ARCH}"
mkdir -p \
  "$TMP_DIR/BUILD" \
  "$TMP_DIR/RPMS" \
  "$TMP_DIR/SOURCES" \
  "$TMP_DIR/SPECS" \
  "$TMP_DIR/SRPMS" \
  "$BUILDROOT/usr/bin" \
  "$BUILDROOT/usr/share/applications" \
  "$BUILDROOT/usr/share/icons/hicolor/scalable/apps" \
  "$BUILDROOT/usr/share/doc/edirstat" \
  "$BUILDROOT/usr/share/licenses/edirstat"

# 1. Binary
install -m 755 "$BINARY_PATH" "$BUILDROOT/usr/bin/edirstat"

# 2. Desktop Entry
DESKTOP_SRC="assets/linux/edirstat.desktop"
if [[ -f "$DESKTOP_SRC" ]]; then
  install -m 644 "$DESKTOP_SRC" "$BUILDROOT/usr/share/applications/edirstat.desktop"
else
  cat > "$BUILDROOT/usr/share/applications/edirstat.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=eDirStat
Comment=Fast, interactive graphical disk usage analyzer & deduplication engine
GenericName=Disk Usage Analyzer
Exec=edirstat %U
Icon=edirstat
Terminal=false
StartupNotify=true
Categories=System;Filesystem;Utility;
Keywords=disk;usage;analyzer;storage;size;cleanup;dedup;treemap;
MimeType=application/octet-stream;
EOF
fi

# 3. Icons (PNG raster sizes + SVG scalable)
for size in 16 32 48 64 128 256 512; do
  icon_src="assets/img/icon_${size}x.png"
  if [[ -f "$icon_src" ]]; then
    dest_dir="$BUILDROOT/usr/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$dest_dir"
    install -m 644 "$icon_src" "$dest_dir/edirstat.png"
  fi
done

SVG_SRC="assets/img/icon-transparent.svg"
if [[ ! -f "$SVG_SRC" ]]; then
  SVG_SRC="assets/img/icon.svg"
fi
if [[ -f "$SVG_SRC" ]]; then
  install -m 644 "$SVG_SRC" "$BUILDROOT/usr/share/icons/hicolor/scalable/apps/edirstat.svg"
fi

# 4. Documentation & Licenses
if [[ -f "LICENSE" ]]; then
  install -m 644 "LICENSE" "$BUILDROOT/usr/share/licenses/edirstat/LICENSE"
fi
if [[ -f "README.md" ]]; then
  install -m 644 "README.md" "$BUILDROOT/usr/share/doc/edirstat/README.md"
fi

# 5. Generate Spec File
SPEC_FILE="$TMP_DIR/SPECS/edirstat.spec"
cat > "$SPEC_FILE" <<EOF
Name:           edirstat
Version:        ${VERSION}
Release:        ${BUILD_NUM}%{?dist}
Summary:        Fast, interactive graphical disk usage analyzer & deduplication engine
License:        MIT
URL:            https://edirstat.com
BuildArch:      ${ARCH}

AutoReqProv:    no
Requires:       glibc >= 2.31
Requires:       libxkbcommon
Requires:       fontconfig
Recommends:     libwayland-client
Recommends:     libX11

%description
eDirStat is a modern disk usage analyzer in Rust (WinDirStat/KDirStat-inspired),
featuring a work-stealing parallel scanner, zero-copy arena data model,
zstd-compressed snapshots, an interactive egui treemap GUI, an NTFS \$MFT
parser, and a 7-stage BLAKE3 deduplication engine.

%files
/usr/bin/edirstat
/usr/share/applications/edirstat.desktop
/usr/share/icons/hicolor/*/apps/edirstat.*
%doc /usr/share/doc/edirstat/README.md
%license /usr/share/licenses/edirstat/LICENSE
EOF

# ---------- Build RPM Package ----------
echo "==> Running rpmbuild..."
rpmbuild -bb \
  --define "_topdir $TMP_DIR" \
  --buildroot "$BUILDROOT" \
  "$SPEC_FILE"

# ---------- Collect Built Artifact ----------
BUILT_RPM="$(find "$TMP_DIR/RPMS" -type f -name "edirstat-*.rpm" | head -1 || true)"

if [[ -z "$BUILT_RPM" || ! -f "$BUILT_RPM" ]]; then
  echo "ERROR: rpmbuild failed to produce an RPM package." >&2
  exit 1
fi

DEST_RPM="$OUT_DIR/$(basename "$BUILT_RPM")"
cp "$BUILT_RPM" "$DEST_RPM"

PKG_SIZE="$(du -h "$DEST_RPM" | cut -f1)"
echo "==> Successfully built RPM package: $DEST_RPM ($PKG_SIZE)"

if command -v rpm >/dev/null 2>&1; then
  echo "==> Package info:"
  rpm -qip "$DEST_RPM" | head -15 | sed 's/^/    /' || true
fi
