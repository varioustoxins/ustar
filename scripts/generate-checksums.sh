#!/bin/bash

# Script to generate SHA-1 checksum files for specified directories
# Creates checksums.sha1 in each directory using platform shasum command
#
# Usage: 
#   ./generate-checksums.sh <dir1> [dir2] [dir3] ...
#   ./generate-checksums.sh -v <dir1> [dir2] [dir3] ...  (verbose mode)
#   ./generate-checksums.sh ustar-parser/tests/test_data/bmrb_stars
#   ./generate-checksums.sh ustar-parser/tests/test_data/*

set -e  # Exit on any error

# Function to print verbose messages
verbose_echo() {
    if [[ "$VERBOSE" == true ]]; then
        echo "$@"
    fi
}

# Parse options
VERBOSE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [-v|--verbose] <directory1> [directory2] [directory3] ..."
            echo ""
            echo "Options:"
            echo "  -v, --verbose    Show detailed output during processing"
            echo "  -h, --help       Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 ustar-parser/tests/test_data/bmrb_stars"
            echo "  $0 -v ustar-parser/tests/test_data/bmrb_stars ustar-parser/tests/test_data/cod_cifs"
            echo "  $0 ustar-parser/tests/test_data/*"
            exit 0
            ;;
        -*)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

# Check if at least one directory argument is provided
if [ $# -eq 0 ]; then
    echo "Error: No directories specified" >&2
    echo "Use -h or --help for usage information" >&2
    exit 1
fi

# File extensions to include in checksums
EXTENSIONS=("*.str" "*.cif" "*.nef" "*.dic" "*.mmcif")

verbose_echo "Generating SHA-1 checksum files for specified directories..."

# Process each directory argument
for dir in "$@"; do
    if [[ ! -d "$dir" ]]; then
        verbose_echo "Warning: Directory $dir does not exist, skipping..." >&2
        continue
    fi
    
    verbose_echo "Processing $dir..."
    
    # Change to the directory
    cd "$dir"
    
    # Find all files matching our extensions and generate checksums
    # Use -f flag to suppress "no matches found" errors for missing extensions
    found_files=false
    temp_checksum_file=$(mktemp)
    
    for ext in "${EXTENSIONS[@]}"; do
        if ls $ext 1> /dev/null 2>&1; then
            shasum $ext >> "$temp_checksum_file"
            found_files=true
        fi
    done
    
    if [[ "$found_files" == true ]]; then
        # Sort the checksums for consistent output
        sort "$temp_checksum_file" > checksums.sha1
        verbose_echo "  Created checksums.sha1 with $(wc -l < checksums.sha1) files"
    else
        echo "Warning: No test data files found in $dir" >&2
    fi
    
    # Clean up
    rm -f "$temp_checksum_file"
    
    # Go back to original directory
    cd - > /dev/null
done

verbose_echo ""
verbose_echo "Checksum generation complete!"
verbose_echo ""
verbose_echo "To verify checksums later, use:"
verbose_echo "  cd <test_data_directory>"
verbose_echo "  shasum -c checksums.sha1"