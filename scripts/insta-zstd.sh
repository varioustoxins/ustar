#!/bin/bash
# Full wrapper around `cargo insta` with automatic zstd snapshot compression
# and cleanup functionality.
#
# Usage: ./scripts/insta-zstd.sh [clean|accept|test|review|show|...] [--keep-diffs] [--verbose] [insta args...]
#
# Commands:
#   clean           Remove all .snap.old, .snap.new, and .snap.diff files
#   accept          Run `cargo insta accept` (default if no command specified)
#   test|review|... Any other `cargo insta` subcommand
#
# Options:
#   --keep-diffs    Do not remove .snap.diff files (keep for review)
#   --verbose       Show detailed processing information
#
# This script:
# 1. Ensures all .snap files are available for insta (decompresses .snap.zst if needed)
# 2. Runs any `cargo insta` command with provided arguments
# 3. Automatically compresses new/changed .snap files to .snap.zst using zstd
# 4. Removes temporary snapshot files (unless --keep-diffs is specified)

set -e

# Parse command and custom options
COMMAND=""
KEEP_DIFFS=false
VERBOSE=false
INSTA_ARGS=()

# First argument might be a command
if [[ $# -gt 0 && "$1" != --* ]]; then
    COMMAND="$1"
    shift
fi

# Parse remaining arguments for our custom options
for arg in "$@"; do
    if [[ "$arg" == "--keep-diffs" ]]; then
        KEEP_DIFFS=true
    elif [[ "$arg" == "--verbose" ]]; then
        VERBOSE=true
    else
        INSTA_ARGS+=("$arg")
    fi
done

# Default to 'accept' if no command specified
if [[ -z "$COMMAND" ]]; then
    COMMAND="accept"
fi

# Verbose output helper
verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo "$@"
    fi
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Snapshot directories to check
SNAPSHOT_DIRS=(
    "ustar-parser/tests/snapshots"
    "ustar-tools/tests/snapshots"
)

# Function to ensure decompressed files exist for any missing .snap files
ensure_decompressed_snapshots() {
    local decompressed_count=0
    
    for dir in "${SNAPSHOT_DIRS[@]}"; do
        full_path="$PROJECT_ROOT/$dir"
        if [[ -d "$full_path" ]]; then
            # Find .snap.zst files that don't have corresponding .snap files
            while IFS= read -r -d '' zst_file; do
                snap_file="${zst_file%.zst}"
                if [[ ! -f "$snap_file" ]]; then
                    verbose "  Decompressing: $(basename "$zst_file") -> $(basename "$snap_file")"
                    zstd -d -c "$zst_file" > "$snap_file"
                    ((decompressed_count++))
                fi
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.zst" -type f -print0 2>/dev/null)
        fi
    done
    
    if [[ $decompressed_count -gt 0 ]]; then
        echo "Decompressed $decompressed_count .snap.zst files to .snap files"
    fi
}

# Function to clean temporary snapshot files
clean_temporary_files() {
    local cleaned_count=0
    
    for dir in "${SNAPSHOT_DIRS[@]}"; do
        full_path="$PROJECT_ROOT/$dir"
        if [[ -d "$full_path" ]]; then
            # Remove .snap.old files
            while IFS= read -r -d '' old_file; do
                verbose "  Removing old: $(basename "$old_file")"
                rm -f "$old_file"
                ((cleaned_count++))
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.old" -type f -print0 2>/dev/null)
            
            # Remove .snap.new files
            while IFS= read -r -d '' new_file; do
                verbose "  Removing new: $(basename "$new_file")"
                rm -f "$new_file"
                ((cleaned_count++))
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.new" -type f -print0 2>/dev/null)
            
            # Remove .snap.diff files
            while IFS= read -r -d '' diff_file; do
                verbose "  Removing diff: $(basename "$diff_file")"
                rm -f "$diff_file"
                ((cleaned_count++))
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.diff" -type f -print0 2>/dev/null)
        fi
    done
    
    echo "Cleaned $cleaned_count temporary snapshot files"
}

# Function to record snapshot states for comparison
record_snapshot_states() {
    local state_file="$1"
    > "$state_file"
    
    for dir in "${SNAPSHOT_DIRS[@]}"; do
        full_path="$PROJECT_ROOT/$dir"
        if [[ -d "$full_path" ]]; then
            while IFS= read -r -d '' snap_file; do
                # Record file path and modification time
                echo "$(stat -f "%m" "$snap_file") $snap_file" >> "$state_file"
            done < <(find "$full_path" -maxdepth 1 -name "*.snap" -type f -print0 2>/dev/null)
        fi
    done
}

# Function to compress snapshots (only files that changed)
compress_snapshots() {
    local before_state="$1"
    local compressed_count=0
    local diff_count=0
    local old_count=0
    local new_count=0

    # Create current state
    local after_state=$(mktemp)
    record_snapshot_states "$after_state"
    
    # Find files that changed (different modification times or new files)
    local changed_files=$(mktemp)
    
    # Get files that exist now
    cut -d' ' -f2- "$after_state" | sort > "${changed_files}.after"
    
    if [[ -f "$before_state" ]]; then
        # Get files that existed before
        cut -d' ' -f2- "$before_state" | sort > "${changed_files}.before"
        
        # Files that are new or potentially modified
        comm -13 "${changed_files}.before" "${changed_files}.after" > "${changed_files}.new"
        
        # Check modification times for existing files
        while IFS= read -r file_path; do
            if [[ -n "$file_path" ]]; then
                before_time=$(grep " $file_path$" "$before_state" 2>/dev/null | cut -d' ' -f1 || echo "0")
                after_time=$(grep " $file_path$" "$after_state" 2>/dev/null | cut -d' ' -f1 || echo "0")
                
                if [[ "$before_time" != "$after_time" ]]; then
                    echo "$file_path" >> "${changed_files}.new"
                fi
            fi
        done < "${changed_files}.before"
    else
        # No before state - all files are "new"
        cat "${changed_files}.after" > "${changed_files}.new"
    fi
    
    # Compress files that changed
    while IFS= read -r snap_file; do
        if [[ -n "$snap_file" && -f "$snap_file" ]]; then
            base_name="${snap_file%.snap}"
            zst_file="${base_name}.snap.zst"
            
            verbose "  Compressing: $(basename "$snap_file") -> $(basename "$zst_file")"
            zstd -c "$snap_file" > "$zst_file"
            ((compressed_count++))
        fi
    done < "${changed_files}.new"
    
    # Clean up temp files
    rm -f "$after_state" "${changed_files}"*

    for dir in "${SNAPSHOT_DIRS[@]}"; do
        full_path="$PROJECT_ROOT/$dir"
        if [[ -d "$full_path" ]]; then
            # Remove temporary files unless --keep-diffs is set
            if [[ "$KEEP_DIFFS" == "false" ]]; then
                # Remove .snap.diff files
                while IFS= read -r -d '' diff_file; do
                    verbose "  Removing diff: $(basename "$diff_file")"
                    rm -f "$diff_file"
                    ((diff_count++))
                done < <(find "$full_path" -maxdepth 1 -name "*.snap.diff" -type f -print0 2>/dev/null)
                
                # Remove .snap.old files
                while IFS= read -r -d '' old_file; do
                    verbose "  Removing old: $(basename "$old_file")"
                    rm -f "$old_file"
                    ((old_count++))
                done < <(find "$full_path" -maxdepth 1 -name "*.snap.old" -type f -print0 2>/dev/null)
                
                # Remove .snap.new files
                while IFS= read -r -d '' new_file; do
                    verbose "  Removing new: $(basename "$new_file")"
                    rm -f "$new_file"
                    ((new_count++))
                done < <(find "$full_path" -maxdepth 1 -name "*.snap.new" -type f -print0 2>/dev/null)
            fi
        fi
    done
    
    # Show summary if there's something to report
    if [[ "$VERBOSE" == "true" ]] || [[ $compressed_count -gt 0 ]] || [[ $diff_count -gt 0 ]] || [[ $old_count -gt 0 ]] || [[ $new_count -gt 0 ]]; then
        echo ""
        echo "Summary:"
        if [[ $compressed_count -gt 0 ]]; then
            echo "  - Compressed $compressed_count changed .snap file(s) to .snap.zst"
        fi
        if [[ $diff_count -gt 0 ]]; then
            echo "  - Removed $diff_count .snap.diff file(s)"
        fi
        if [[ $old_count -gt 0 ]]; then
            echo "  - Removed $old_count .snap.old file(s)"
        fi
        if [[ $new_count -gt 0 ]]; then
            echo "  - Removed $new_count .snap.new file(s)"
        fi
        if [[ "$KEEP_DIFFS" == "true" ]]; then
            echo "  - Kept temporary snapshot files (--keep-diffs)"
        fi
        if [[ $compressed_count -eq 0 && $diff_count -eq 0 && $old_count -eq 0 && $new_count -eq 0 && "$KEEP_DIFFS" == "false" ]]; then
            echo "  - No snapshot files to process"
        fi
        verbose "Done."
    fi
}

# Handle special 'clean' command
if [[ "$COMMAND" == "clean" ]]; then
    verbose "Cleaning temporary snapshot files..."
    clean_temporary_files
    exit 0
fi

# Ensure decompressed snapshots exist before running insta
verbose "Ensuring .snap files are available for insta..."
ensure_decompressed_snapshots

# Record snapshot states before running insta
verbose "Recording snapshot states before cargo insta $COMMAND..."
before_state=$(mktemp)
record_snapshot_states "$before_state"

# Run cargo insta command
verbose "Running cargo insta $COMMAND with arguments: ${INSTA_ARGS[*]}"
if cargo insta "$COMMAND" "${INSTA_ARGS[@]}"; then
    # Compress and cleanup snapshots only if command succeeded
    verbose ""
    verbose "Processing snapshot files after cargo insta $COMMAND..."
    compress_snapshots "$before_state"
else
    verbose "cargo insta command failed, skipping compression"
    exit 1
fi

# Clean up temp state file
rm -f "$before_state"