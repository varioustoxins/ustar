#!/bin/bash
# Wrapper around `cargo insta accept` that automatically compresses
# accepted snapshots to .snap.gz format.
#
# Usage: ./scripts/insta-accept.sh [insta args...]
#
# This script:
# 1. Runs `cargo insta accept` with any provided arguments
# 2. Finds any new .snap files in the test snapshot directories
# 3. Compresses them to .snap.gz and removes the uncompressed version

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Snapshot directories to check
SNAPSHOT_DIRS=(
    "ustar-parser/tests/snapshots"
    "ustar-tools/tests/snapshots"
    "ustar-test-utils/src/snapshots"
)

echo "Running cargo insta accept $*..."
cargo insta accept "$@"

echo ""
echo "Compressing new .snap files to .snap.gz..."

compressed_count=0

for dir in "${SNAPSHOT_DIRS[@]}"; do
    full_path="$PROJECT_ROOT/$dir"
    if [[ -d "$full_path" ]]; then
        # Find .snap files (not .snap.gz or .snap.new)
        while IFS= read -r -d '' snap_file; do
            echo "  Compressing: $snap_file"
            gzip -f "$snap_file"
            ((compressed_count++))
        done < <(find "$full_path" -maxdepth 1 -name "*.snap" -type f -print0 2>/dev/null)
    fi
done

if [[ $compressed_count -eq 0 ]]; then
    echo "  No new .snap files found to compress."
else
    echo ""
    echo "Compressed $compressed_count snapshot(s) to .snap.gz format."
fi

echo "Done."
