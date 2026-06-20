#!/usr/bin/env bash

# ==============================================================================
# Script Name: generate_licenses.sh
# Description: Generates target-specific third party licenses markdown files
#              using cargo-about.
# ==============================================================================

set -o errexit  # Exit on error
set -o nounset  # Exit on use of undeclared variables
set -o pipefail # Return exit status of the last command in the pipe that failed

log_info() {
    echo -e "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

log_error() {
    echo -e "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

check_dependencies() {
    if ! command -v cargo-about &> /dev/null; then
        log_error "cargo-about is not installed or not in PATH."
        exit 1
    fi
}

main() {
    check_dependencies

    # Target-specific markdown files
    local targets=(
        "x86_64-unknown-linux-gnu:assets/licenses/linux.md"
        "x86_64-pc-windows-msvc:assets/licenses/windows.md"
        "x86_64-apple-darwin:assets/licenses/macos.md"
    )

    # Ensure assets/licenses directory exists
    mkdir -p assets/licenses

    for entry in "${targets[@]}"; do
        local target="${entry%%:*}"
        local output_file="${entry##*:}"

        log_info "Generating licenses for target '$target' -> '$output_file'..."
        
        # We run cargo about generate with --target
        cargo about generate \
            --target "$target" \
            -o "$output_file" \
            licenses-md.hbs
    done

    log_info "Successfully generated all target licenses."
}

main "$@"
