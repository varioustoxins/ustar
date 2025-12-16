#!/bin/bash
# Generate grammar files (called by CI and locally)

set -e

echo "=== Generating Grammar Files ==="

# Generate grammar files by building ustar-parser
echo "Building ustar-parser to generate grammar files..."
cargo build --verbose -p ustar-parser

# Verify grammar files were generated
echo "Verifying grammar files generated:"
ls -la ustar-parser/src/star_*.pest

echo "Generated grammar files:"
echo "- ASCII: $(wc -l < ustar-parser/src/star_ascii.pest) lines"
echo "- Extended: $(wc -l < ustar-parser/src/star_extended.pest) lines" 
echo "- Unicode: $(wc -l < ustar-parser/src/star_unicode.pest) lines"

echo "✅ Grammar generation completed"