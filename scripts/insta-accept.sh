#!/bin/bash
# Wrapper around `cargo insta accept` that automatically compresses
# accepted snapshots to .snap.gz format.
#
# Usage: ./scripts/insta-accept.sh [--keep-diffs] [--verbose] [insta args...]
#
# Options:
#   --keep-diffs    Do not remove .snap.diff files (keep for review)
#   --verbose       Show detailed processing information
#
# This script:
# 1. Runs `cargo insta accept` with any provided arguments
# 2. Compresses any .snap files to .snap.gz
# 3. Removes .snap.diff and .snap.old files (unless --keep-diffs is specified)

set -e

# Parse our custom options
KEEP_DIFFS=false
VERBOSE=false
INSTA_ARGS=()
for arg in "$@"; do
    if [[ "$arg" == "--keep-diffs" ]]; then
        KEEP_DIFFS=true
    elif [[ "$arg" == "--verbose" ]]; then
        VERBOSE=true
    else
        INSTA_ARGS+=("$arg")
    fi
done

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

verbose "Running cargo insta accept to handle snapshot updates..."
cargo insta accept "${INSTA_ARGS[@]}"

verbose ""
verbose "Synchronizing .snap files with .snap.gz storage..."

compressed_count=0
diff_count=0
old_count=0

for dir in "${SNAPSHOT_DIRS[@]}"; do
    full_path="$PROJECT_ROOT/$dir"
    if [[ -d "$full_path" ]]; then
        # Step 1: Compress any .snap files that are newer than their .snap.gz counterparts
        # This maintains both .snap (working files) and .snap.gz (storage files) in sync
        while IFS= read -r -d '' snap_file; do
            base_name="${snap_file%.snap}"
            gz_file="${base_name}.snap.gz"
            
            # Compress if .snap.gz doesn't exist or .snap is newer
            if [[ ! -f "$gz_file" ]] || [[ "$snap_file" -nt "$gz_file" ]]; then
                verbose "  Synchronizing: $(basename "$snap_file") -> $(basename "$gz_file")"
                
                # Compress .snap to .snap.gz (preserve both files)
                gzip -c "$snap_file" > "$gz_file"
                
                # Keep the .snap file for continued insta usage
                # Both files now contain identical content
                ((compressed_count++))
            fi
        done < <(find "$full_path" -maxdepth 1 -name "*.snap" -type f -print0 2>/dev/null)
        
        # Step 2: Remove .snap.diff files (cleanup) - unless --keep-diffs is set
        if [[ "$KEEP_DIFFS" == "false" ]]; then
            while IFS= read -r -d '' diff_file; do
                verbose "  Removing diff: $diff_file"
                rm -f "$diff_file"
                ((diff_count++))
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.diff" -type f -print0 2>/dev/null)
        fi
        
        # Step 3: Remove .snap.old files (cleanup) - unless --keep-diffs is set
        if [[ "$KEEP_DIFFS" == "false" ]]; then
            while IFS= read -r -d '' old_file; do
                verbose "  Removing old: $old_file"
                rm -f "$old_file"
                ((old_count++))
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.old" -type f -print0 2>/dev/null)
        fi
    fi
done

# Only show summary if verbose or if there's something to report
if [[ "$VERBOSE" == "true" ]] || [[ $compressed_count -gt 0 ]] || [[ $diff_count -gt 0 ]] || [[ $old_count -gt 0 ]]; then
    echo ""
    echo "Summary:"
    if [[ $compressed_count -gt 0 ]]; then
        echo "  - Synchronized $compressed_count .snap file(s) to .snap.gz storage"
    fi
    if [[ $diff_count -gt 0 ]]; then
        echo "  - Removed $diff_count .snap.diff file(s)"
    fi
    if [[ $old_count -gt 0 ]]; then
        echo "  - Removed $old_count .snap.old file(s)"
    fi
    if [[ "$KEEP_DIFFS" == "true" ]]; then
        echo "  - Kept .snap.diff, .snap.old, and .snap.new files (--keep-diffs)"
    fi
    if [[ $compressed_count -eq 0 && $diff_count -eq 0 && $old_count -eq 0 && "$KEEP_DIFFS" == "false" ]]; then
        echo "  - No snapshot files to process"
    fi
    verbose "Done."
fi
