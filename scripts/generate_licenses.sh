#!/usr/bin/env bash

# ==============================================================================
# Script Name: generate_licenses.sh
# Description: Generates target-specific third party licenses markdown files
#              using cargo-about.
# ==============================================================================

set -o errexit  # Exit on error
set -o nounset  # Exit on use of undeclared variables
set -o pipefail # Return exit status of the last command in the pipe that failed

# Pinned: output differs between cargo-about releases. 0.9 keeps every license
# text a crate ships and orders them deterministically; 0.8 keeps one text per
# license and breaks confidence ties by readdir order, which varies across
# filesystems (btrfs vs the CI container's overlayfs).
readonly CARGO_ABOUT_VERSION="0.9.2"

log_info() {
    echo -e "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

log_error() {
    echo -e "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

is_ci() {
    [[ "${CI:-false}" == "true" ]] || [[ "${GITHUB_ACTIONS:-false}" == "true" ]]
}

# The version `cargo about` actually runs. Cargo searches $CARGO_HOME/bin before
# PATH, so `command -v cargo-about` can name a different binary entirely.
cargo_about_version() {
    cargo about --version 2>/dev/null | awk '{print $2}' || true
}

check_dependencies() {
    local found
    found="$(cargo_about_version)"
    if [[ "$found" == "$CARGO_ABOUT_VERSION" ]]; then
        return
    fi
    if is_ci; then
        log_info "Installing cargo-about $CARGO_ABOUT_VERSION (found '${found:-none}')..."
        cargo install cargo-about --locked --features cli --version "$CARGO_ABOUT_VERSION"
        found="$(cargo_about_version)"
        if [[ "$found" == "$CARGO_ABOUT_VERSION" ]]; then
            return
        fi
    fi
    log_error "cargo-about $CARGO_ABOUT_VERSION is required, but \`cargo about\` runs '${found:-none}'."
    log_error "Install it with: cargo install cargo-about --locked --features cli --version $CARGO_ABOUT_VERSION"
    exit 1
}

main() {
    # Ensure we run from the project root directory
    cd "$(dirname "$0")/.."

    check_dependencies

    # Target-specific markdown files: target:manifest_path:output_file. The
    # native lists are the shipped core's (crates/edirstat_app: the whole
    # application plus seaworthy-launcher, which the launcher shares).
    local targets=(
        "x86_64-unknown-linux-gnu:crates/edirstat_app/Cargo.toml:crates/edirstat-gui/assets/licenses/linux.md"
        "x86_64-pc-windows-msvc:crates/edirstat_app/Cargo.toml:crates/edirstat-gui/assets/licenses/windows.md"
        "x86_64-apple-darwin:crates/edirstat_app/Cargo.toml:crates/edirstat-gui/assets/licenses/macos.md"
        "wasm32-unknown-unknown:crates/edirstat-gui/Cargo.toml:crates/edirstat-gui/assets/licenses/web.md"
    )

    # Ensure assets/licenses directory exists
    mkdir -p crates/edirstat-gui/assets/licenses

    for entry in "${targets[@]}"; do
        IFS=":" read -r target manifest output_file <<< "$entry"

        log_info "Generating licenses for target '$target' using manifest '$manifest' -> '$output_file'..."
        
        cargo about generate \
            --locked \
            --manifest-path "$manifest" \
            --target "$target" \
            -o "$output_file" \
            licenses-md.hbs
    done

    log_info "Successfully generated all target licenses."
}

main "$@"
