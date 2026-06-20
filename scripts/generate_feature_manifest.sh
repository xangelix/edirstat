#!/usr/bin/env bash

# ==============================================================================
# Script Name: generate_manifest.sh
# Description: Generates a JSON manifest of CPU features using rustc.
# Usage:       ./generate_manifest.sh "cpu_list" "output_path"
# ==============================================================================

set -o errexit  # Exit on error
set -o nounset  # Exit on use of undeclared variables
set -o pipefail # Return exit status of the last command in the pipe that failed

# ------------------------------------------------------------------------------
# Functions
# ------------------------------------------------------------------------------

log_info() {
    echo -e "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

log_error() {
    echo -e "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

cleanup() {
    # Delete temp file if it exists and script exits unexpectedly
    if [[ -n "${temp_file:-}" ]] && [[ -f "$temp_file" ]]; then
        rm -f "$temp_file"
    fi
}

# Set up trap to call cleanup on exit, error, or interruption
trap cleanup EXIT

check_dependencies() {
    if ! command -v rustc &> /dev/null; then
        log_error "rustc is not installed or not in PATH."
        exit 1
    fi
}

# ------------------------------------------------------------------------------
# Main Execution
# ------------------------------------------------------------------------------

main() {
    if [[ $# -ne 2 ]]; then
        echo "Usage: $0 <cpu_list> <output_file>"
        exit 1
    fi

    local input_cpus="$1"
    local output_file="$2"

    check_dependencies

    # Convert space-separated string to array
    IFS=' ' read -r -a cpu_array <<< "$input_cpus"

    if [[ ${#cpu_array[@]} -eq 0 ]]; then
        log_error "No CPUs provided."
        exit 1
    fi

    log_info "Generating manifest for ${#cpu_array[@]} targets..."
    
    # Declare temp_file at global scope for trap cleanup
    # We create it now
    temp_file=$(mktemp)
    
    # Start JSON
    echo "{" > "$temp_file"
    echo "    \"builds\": [" >> "$temp_file"

    local cpu_count=${#cpu_array[@]}
    local i=0

    for cpu in "${cpu_array[@]}"; do
        # Use pre-increment ((++i)) so it never evaluates to 0 (false)
        ((++i))
        
        log_info "Processing target: $cpu ($i/$cpu_count)"

        if ! rustc --print cfg -C target-cpu="$cpu" > /dev/null 2>&1; then
             log_error "rustc failed to process target-cpu='$cpu'. Skipping."
             exit 1
        fi

        # Extract features
        # We disable pipefail strictly for this block to allow grep to return empty
        # if a CPU has no features, without crashing the script.
        set +o pipefail
        mapfile -t features < <( \
            rustc --print cfg -C target-cpu="$cpu" \
            | grep 'target_feature=' \
            | sed -n 's/target_feature="\(.*\)"/\1/p' \
            | sort \
        )
        set -o pipefail

        echo "        {" >> "$temp_file"
        echo "            \"path\": \"target/X-${cpu}\"," >> "$temp_file"
        echo "            \"features\": [" >> "$temp_file"

        local num_features=${#features[@]}
        local j=0
        
        for feature in "${features[@]}"; do
            ((++j))
            if [[ $j -lt $num_features ]]; then
                echo "                \"$feature\"," >> "$temp_file"
            else
                echo "                \"$feature\"" >> "$temp_file"
            fi
        done

        echo "            ]" >> "$temp_file"

        if [[ $i -lt $cpu_count ]]; then
            echo "        }," >> "$temp_file"
        else
            echo "        }" >> "$temp_file"
        fi
    done

    echo "    ]" >> "$temp_file"
    echo "}" >> "$temp_file"

    mv "$temp_file" "$output_file"
    
    # Clear temp_file variable so trap doesn't try to delete the moved file
    temp_file=""
    
    log_info "Successfully wrote manifest to $output_file"
}

main "$@"
