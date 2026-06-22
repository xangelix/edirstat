#!/usr/bin/env bash
set -Eeuo pipefail

# -------------------------------------------------------------------------
# Configuration
# -------------------------------------------------------------------------

TARGET_CPUS="${TARGET_CPUS:-x86-64-v2 x86-64-v3 x86-64-v4 znver3 znver4 znver5 skylake alderlake}"
MULTIVERS_ZSTD_LEVEL="${MULTIVERS_ZSTD_LEVEL:-20}"

TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
PKG_NAME="edirstat"
BIN_NAME="edirstat"
CRATE_SUBDIR="runner"

# Determine executable extension and static linking flags based on target
EXE_EXT=""
STATIC_FLAG=""

if [[ "$TARGET" == *"windows"* ]]; then
    EXE_EXT=".exe"
    STATIC_FLAG="-C target-feature=+crt-static"
elif [[ "$TARGET" == *"musl"* ]]; then
    STATIC_FLAG="-C target-feature=+crt-static"
else
    # Default GNU Linux or other targets where static glibc is problematic
    STATIC_FLAG=""
fi

PROJECT_DIR="$(pwd)"
BIN_PATH="${TARGET}/release/${BIN_NAME}"
OUT_DIR="${PROJECT_DIR}/target/${TARGET}/release"

echo "==========================================================="
echo "🚀 Building local NON-FIPS multivers binary"
echo "🎯 Target: $TARGET"
echo "🧠 CPUs:   $TARGET_CPUS"
echo "🔗 Static: $([ -n "$STATIC_FLAG" ] && echo "Enabled" || echo "Disabled (GNU/Dynamic)")"
echo "==========================================================="

# 1. Ensure target is installed
rustup target add "$TARGET"
mkdir -p "$OUT_DIR"

# -------------------------------------------------------------------------
# 2. Build inner binaries (Matrix step equivalent)
# -------------------------------------------------------------------------
for CPU in $TARGET_CPUS; do
    echo "-----------------------------------------------------------"
    echo "🔨 Compiling for CPU: $CPU"
    echo "-----------------------------------------------------------"
    
    # Remove any existing binaries or symlinks to prevent same-file/override errors
    rm -f "${OUT_DIR}/${BIN_NAME}${EXE_EXT}"
    rm -f "${OUT_DIR}/${BIN_NAME}-${CPU}${EXE_EXT}"
    
    # Build with default features so update checks and egui features work fully
    RUSTFLAGS="--cfg reqwest_unstable -C target-cpu=${CPU} ${STATIC_FLAG}" \
    cargo build --release \
        --target "$TARGET" \
        --package "$PKG_NAME" \
        --bin "$BIN_NAME"
    
    # Move and append the CPU suffix and correct executable extension
    mv "${OUT_DIR}/${BIN_NAME}${EXE_EXT}" "${OUT_DIR}/${BIN_NAME}-${CPU}${EXE_EXT}"
done

# -------------------------------------------------------------------------
# 3. Generate Manifest
# -------------------------------------------------------------------------
echo "-----------------------------------------------------------"
echo "📝 Generating Feature Manifest"
echo "-----------------------------------------------------------"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ ! -f "${SCRIPT_DIR}/generate_feature_manifest.sh" ]; then
    echo "❌ Error: ${SCRIPT_DIR}/generate_feature_manifest.sh not found."
    exit 1
fi

"${SCRIPT_DIR}/generate_feature_manifest.sh" "$TARGET_CPUS" "${PROJECT_DIR}/multivers_manifest.json"

# Replace 'target/X' with the absolute path to the base binary.
# AND append the appropriate executable extension to the filenames.
sed "s|target/X|${PROJECT_DIR}/target/${BIN_PATH}|g" "${PROJECT_DIR}/multivers_manifest.json" | \
sed "s|${BIN_NAME}\(-[a-zA-Z0-9_-]*\)\"|${BIN_NAME}\1${EXE_EXT}\"|g" > "${PROJECT_DIR}/builds_absolute.json"

# -------------------------------------------------------------------------
# 4. Compile the Runner Wrapper
# -------------------------------------------------------------------------
echo "-----------------------------------------------------------"
echo "📦 Compiling Runner Wrapper"
echo "-----------------------------------------------------------"

# Isolate the wrapper build just like the CI does
WRAPPER_TMP=$(mktemp -d)

# Cleanup trap to ensure we don't leave artifacts laying around locally
trap 'rm -rf "$WRAPPER_TMP"; rm -f "${PROJECT_DIR}/multivers_manifest.json" "${PROJECT_DIR}/builds_absolute.json"' EXIT

cd "$WRAPPER_TMP"
cargo new runner-wrapper --bin
cd runner-wrapper

# Copy metadata, runner files, and icons from the repository
cp "${PROJECT_DIR}/${CRATE_SUBDIR}/Cargo.toml" Cargo.toml
# Make relative path dependencies absolute so Cargo can resolve them from the temp directory
sed -i "s|path = \"|path = \"${PROJECT_DIR}/${CRATE_SUBDIR}/|g" Cargo.toml
cp "${PROJECT_DIR}/${CRATE_SUBDIR}/src/main.rs" src/main.rs
if [ -f "${PROJECT_DIR}/${CRATE_SUBDIR}/build.rs" ]; then
    cp "${PROJECT_DIR}/${CRATE_SUBDIR}/build.rs" build.rs
fi
# Copy the icon from the workspace assets directory to the wrapper build root
if [ -f "${PROJECT_DIR}/assets/img/icon.ico" ]; then
    cp "${PROJECT_DIR}/assets/img/icon.ico" icon.ico
fi

# Point the multivers crate macro to our absolute manifest
export MULTIVERS_BUILDS_DESCRIPTION_PATH="${PROJECT_DIR}/builds_absolute.json"
export MULTIVERS_ZSTD_LEVEL

# Build the runner with matched target link configurations
RUSTFLAGS="${STATIC_FLAG}" cargo build --release --target "$TARGET"

# -------------------------------------------------------------------------
# 5. Finalize
# -------------------------------------------------------------------------
cd "$PROJECT_DIR"

FINAL_EXE="${OUT_DIR}/${BIN_NAME}_multivers${EXE_EXT}"

# Include /runner-wrapper/ in the path!
mv "$WRAPPER_TMP/runner-wrapper/target/${TARGET}/release/edirstat-runner${EXE_EXT}" "$FINAL_EXE"

echo "==========================================================="
echo "✅ Success! Non-FIPS multivers binary built:"
echo "   $FINAL_EXE"
echo "==========================================================="
