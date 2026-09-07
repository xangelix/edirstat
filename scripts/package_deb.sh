#!/usr/bin/env bash
# package_deb.sh — Build a Debian / Ubuntu (.deb) package for eDirStat.
#
# Usage:
#   ./scripts/package_deb.sh [options]
#
# Options:
#   --binary <path>     Path to compiled binary (defaults to target/.../edirstat_multivers or target/release/edirstat)
#   --version <ver>     Package version (auto-detected from Cargo.toml if omitted)
#   --build-num <num>   Debian package revision (default: 1)
#   --arch <arch>       Target architecture (default: amd64)
#   --out-dir <dir>     Output directory (default: staging-linux-installer)
#   -h, --help          Show this help message
#
# Artifacts produced:
#   <out-dir>/edirstat_<version>-<build-num>_<arch>.deb

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
ARCH="amd64"
OUT_DIR="staging-linux-installer"

# ---------- Args Parsing ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary)           BINARY_PATH="$2"; shift 2 ;;
    --version)          VERSION="$2"; shift 2 ;;
    --build-num|--pkgrel) BUILD_NUM="$2"; shift 2 ;;
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

echo "==> Packaging eDirStat for Debian / Ubuntu"
echo "  • Version:      $VERSION-$BUILD_NUM"
echo "  • Architecture: $ARCH"
echo "  • Source binary: $BINARY_PATH"
echo "  • Output dir:   $OUT_DIR"

mkdir -p "$OUT_DIR"

# ---------- Stage Payload Directory ----------
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/edirstat_deb_pkg.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

PKG_ROOT="$TMP_DIR/root"
mkdir -p \
  "$PKG_ROOT/usr/bin" \
  "$PKG_ROOT/usr/share/applications" \
  "$PKG_ROOT/usr/share/icons/hicolor/scalable/apps" \
  "$PKG_ROOT/usr/share/doc/edirstat" \
  "$PKG_ROOT/DEBIAN"

# 1. Binary
install -m 755 "$BINARY_PATH" "$PKG_ROOT/usr/bin/edirstat"

# 2. Desktop Entry
DESKTOP_SRC="assets/linux/edirstat.desktop"
if [[ -f "$DESKTOP_SRC" ]]; then
  install -m 644 "$DESKTOP_SRC" "$PKG_ROOT/usr/share/applications/edirstat.desktop"
else
  cat > "$PKG_ROOT/usr/share/applications/edirstat.desktop" <<'EOF'
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
    dest_dir="$PKG_ROOT/usr/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$dest_dir"
    install -m 644 "$icon_src" "$dest_dir/edirstat.png"
  fi
done

SVG_SRC="assets/img/icon-transparent.svg"
if [[ ! -f "$SVG_SRC" ]]; then
  SVG_SRC="assets/img/icon.svg"
fi
if [[ -f "$SVG_SRC" ]]; then
  install -m 644 "$SVG_SRC" "$PKG_ROOT/usr/share/icons/hicolor/scalable/apps/edirstat.svg"
fi

# 4. Documentation & Licenses
if [[ -f "LICENSE" ]]; then
  install -m 644 "LICENSE" "$PKG_ROOT/usr/share/doc/edirstat/copyright"
fi
if [[ -f "README.md" ]]; then
  install -m 644 "README.md" "$PKG_ROOT/usr/share/doc/edirstat/README.md"
fi

# 5. Compute Installed-Size in KB
INSTALLED_SIZE="$(du -sk "$PKG_ROOT/usr" | cut -f1)"

# 6. Generate DEBIAN/control
cat > "$PKG_ROOT/DEBIAN/control" <<EOF
Package: edirstat
Version: ${VERSION}-${BUILD_NUM}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6 (>= 2.31), libxkbcommon0, libfontconfig1
Recommends: libwayland-client0, libx11-6
Homepage: https://edirstat.com
Description: Fast, cross-platform disk usage analyzer and deduplication engine
 eDirStat is a modern disk usage analyzer in Rust (WinDirStat/KDirStat-inspired),
 featuring a work-stealing parallel scanner, zero-copy arena data model,
 zstd-compressed snapshots, an interactive egui treemap GUI, an NTFS \$MFT
 parser, and a 7-stage BLAKE3 deduplication engine.
EOF

OUT_FILE="$OUT_DIR/edirstat_${VERSION}-${BUILD_NUM}_${ARCH}.deb"
rm -f "$OUT_FILE"

# ---------- Build .deb Archive ----------
if command -v dpkg-deb >/dev/null 2>&1; then
  echo "==> Building Debian package with dpkg-deb..."
  dpkg-deb --build --root-owner-group "$PKG_ROOT" "$OUT_FILE"
else
  echo "==> dpkg-deb not found; assembling .deb package using POSIX ar + tar..."
  
  # Standard Debian package components:
  # 1. debian-binary (contains "2.0\n")
  # 2. control.tar.gz (contains DEBIAN/ files)
  # 3. data.tar.xz (contains usr/ tree)
  
  STAGE_ARCHIVE="$TMP_DIR/archive"
  mkdir -p "$STAGE_ARCHIVE"
  
  echo "2.0" > "$STAGE_ARCHIVE/debian-binary"
  
  # control.tar.gz
  (
    cd "$PKG_ROOT/DEBIAN"
    tar --numeric-owner --owner=0 --group=0 -czf "$STAGE_ARCHIVE/control.tar.gz" .
  )
  
  # data.tar.xz (or .gz)
  if command -v xz >/dev/null 2>&1; then
    DATA_FILE="data.tar.xz"
    (
      cd "$PKG_ROOT"
      tar --numeric-owner --owner=0 --group=0 -c usr | xz -T0 -9 > "$STAGE_ARCHIVE/$DATA_FILE"
    )
  else
    DATA_FILE="data.tar.gz"
    (
      cd "$PKG_ROOT"
      tar --numeric-owner --owner=0 --group=0 -czf "$STAGE_ARCHIVE/$DATA_FILE" usr
    )
  fi
  
  (
    cd "$STAGE_ARCHIVE"
    ar -rc "$OUT_FILE" debian-binary control.tar.gz "$DATA_FILE"
  )
fi

# ---------- Validation ----------
if [[ -f "$OUT_FILE" ]]; then
  PKG_SIZE="$(du -h "$OUT_FILE" | cut -f1)"
  echo "==> Successfully built Debian package: $OUT_FILE ($PKG_SIZE)"
  
  if command -v dpkg-deb >/dev/null 2>&1; then
    echo "==> Package info:"
    dpkg-deb -I "$OUT_FILE" | head -15 | sed 's/^/    /'
  fi
else
  echo "ERROR: Failed to create $OUT_FILE" >&2
  exit 1
fi
