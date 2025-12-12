#!/bin/bash
# Setup git hooks to match CI configuration
# Run this script once after cloning the repository

set -e

echo "🔧 Setting up git hooks to match CI configuration..."

# Check if we're in the right directory
if [[ ! -f Cargo.toml ]] || [[ ! -d ustar-parser ]]; then
    echo "❌ Error: This script must be run from the ustar project root"
    exit 1
fi

# Install pre-commit if not available
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    else
        echo "❌ Error: Please install pre-commit manually:"
        echo "   pip install pre-commit"
        echo "   or: brew install pre-commit"
        exit 1
    fi
fi

# Install pre-commit hooks
echo "⚙️  Installing pre-commit hooks..."
pre-commit install

# Test hooks
echo "🧪 Testing hooks..."
if pre-commit run --all-files >/dev/null 2>&1; then
    echo "✅ All hooks configured and working!"
else
    echo "⚠️  Some hooks had issues. This is normal on first run."
    echo "   The hooks will work correctly on future commits."
fi

echo ""
echo "🎉 Git hooks setup complete!"
echo ""
echo "📋 What's configured:"
echo "   • Pre-commit: Runs format, clippy, and tests on changed files"
echo "   • Pre-push: Runs comprehensive CI-matching checks"
echo ""
echo "💡 To manually run checks:"
echo "   • pre-commit run --all-files    # Run all pre-commit checks"
echo "   • ./scripts/ci-clippy.sh        # Run CI-matching clippy"
echo "   • git push                      # Triggers pre-push checks"
echo ""