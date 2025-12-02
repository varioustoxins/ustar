#!/bin/bash

# Script to add "data_unres" at the beginning of *_UNRES_it*.nef files

#!/bin/bash

DIR=.

# Check if directory exists
if [ ! -d "$DIR" ]; then
    echo "Error: Directory $DIR does not exist"
    exit 1
fi

# Find all matching files
files=$(find "$DIR" -name "*_UNRES_it*.nef" -type f)

if [ -z "$files" ]; then
    echo "No files matching *_UNRES_it*.nef found in $DIR"
    exit 0
fi

echo "Found files to modify:"
echo "$files"
echo ""

# Counter for modified files
count=0

# Process each file
for file in $files; do
    echo "Processing: $file"
    
    # Check if file already starts with "data_"
    first_line=$(head -n 1 "$file")
    if [[ "$first_line" == data_* ]]; then
        echo "  ⚠ File already starts with 'data_' - skipping"
        continue
    fi
    
    # Create a temporary file with data_unres prepended
    temp_file="${file}.tmp"
    echo "data_unres" > "$temp_file"
    echo "" >> "$temp_file"
    cat "$file" >> "$temp_file"
    
    # Replace original file with modified version
    mv "$temp_file" "$file"
    
    echo "  ✓ Added 'data_unres' to beginning of file"
    ((count++))
done

echo ""
echo "Modified $count file(s)"
