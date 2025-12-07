#!/bin/bash
# Wrapper around `cargo insta accept` that automatically compresses
# accepted snapshots to .snap.gz format.
#
# Usage: ./scripts/insta-accept.sh [--keep-diffs] [insta args...]
#
# Options:
#   --keep-diffs    Do not remove .snap.diff files (keep for review)
#
# This script:
# 1. Runs `cargo insta accept` with any provided arguments
# 2. Processes .snap.new files (from our custom snapshot utils) → compress to .snap.gz
# 3. Compresses any .snap files to .snap.gz
# 4. Removes .snap.diff files (unless --keep-diffs is specified)

set -e

# Parse our custom options
KEEP_DIFFS=false
INSTA_ARGS=()
for arg in "$@"; do
    if [[ "$arg" == "--keep-diffs" ]]; then
        KEEP_DIFFS=true
    else
        INSTA_ARGS+=("$arg")
    fi
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Snapshot directories to check
SNAPSHOT_DIRS=(
    "ustar-parser/tests/snapshots"
    "ustar-tools/tests/snapshots"
)

echo "Running cargo insta accept ${INSTA_ARGS[*]}..."
cargo insta accept "${INSTA_ARGS[@]}"

echo ""
echo "Processing snapshot files..."

new_count=0
compressed_count=0
diff_count=0

for dir in "${SNAPSHOT_DIRS[@]}"; do
    full_path="$PROJECT_ROOT/$dir"
    if [[ -d "$full_path" ]]; then
        # Step 1: Process .snap.new files → compress to .snap.gz
        # These are from our custom snapshot utils when comparing against .snap.gz files
        while IFS= read -r -d '' new_file; do
            # Get the base name without .snap.new, then add .snap.gz
            base_name="${new_file%.snap.new}"
            gz_file="${base_name}.snap.gz"
            echo "  Accepting .snap.new: $new_file → $gz_file"
            gzip -c "$new_file" > "$gz_file"
            rm -f "$new_file"
            ((new_count++))
        done < <(find "$full_path" -maxdepth 1 -name "*.snap.new" -type f -print0 2>/dev/null)
        
        # Step 2: Compress .snap files to .snap.gz
        # These are from cargo insta accept or other sources
        while IFS= read -r -d '' snap_file; do
            echo "  Compressing: $snap_file"
            gzip -f "$snap_file"
            ((compressed_count++))
        done < <(find "$full_path" -maxdepth 1 -name "*.snap" -type f -print0 2>/dev/null)
        
        # Step 3: Remove .snap.diff files (cleanup) - unless --keep-diffs is set
        if [[ "$KEEP_DIFFS" == "false" ]]; then
            while IFS= read -r -d '' diff_file; do
                echo "  Removing diff: $diff_file"
                rm -f "$diff_file"
                ((diff_count++))
            done < <(find "$full_path" -maxdepth 1 -name "*.snap.diff" -type f -print0 2>/dev/null)
        fi
    fi
done

echo ""
echo "Summary:"
if [[ $new_count -gt 0 ]]; then
    echo "  - Accepted $new_count .snap.new file(s)"
fi
if [[ $compressed_count -gt 0 ]]; then
    echo "  - Compressed $compressed_count .snap file(s)"
fi
if [[ $diff_count -gt 0 ]]; then
    echo "  - Removed $diff_count .snap.diff file(s)"
fi
if [[ "$KEEP_DIFFS" == "true" ]]; then
    echo "  - Kept .snap.diff files (--keep-diffs)"
fi
if [[ $new_count -eq 0 && $compressed_count -eq 0 && $diff_count -eq 0 && "$KEEP_DIFFS" == "false" ]]; then
    echo "  - No snapshot files to process"
fi

echo "Done."
