#!/bin/bash
# Test Homebrew Formula Locally

set -e

echo "🧪 Testing Homebrew Formula for Cliped Cross-Platform"
echo ""

FORMULA_PATH="dist/cliped-crossplatform.rb"

if [ ! -f "$FORMULA_PATH" ]; then
    echo "❌ Homebrew formula not found at $FORMULA_PATH"
    echo "Run './build-macos.sh' first to generate the formula"
    exit 1
fi

if [ ! -f "dist/cliped-macos.tar.gz" ]; then
    echo "❌ macOS tarball not found at dist/cliped-macos.tar.gz"
    echo "Run './build-macos.sh' first to create the tarball"
    exit 1
fi

echo "🔍 Validating formula syntax..."
brew ruby -e "require './dist/cliped-crossplatform.rb'; puts 'Formula syntax is valid!'"

echo ""
echo "📦 Formula details:"
echo "   Name: cliped-crossplatform"
echo "   Version: $(grep 'version' $FORMULA_PATH | head -1 | cut -d'"' -f2)"
echo "   Description: $(grep 'desc' $FORMULA_PATH | cut -d'"' -f2)"
echo ""

echo "🏠 To test installation locally:"
echo "   brew install --build-from-source $FORMULA_PATH"
echo ""
echo "⚠️  Note: This will install from the local tarball."
echo "   For production, update the URL to point to GitHub releases."
echo ""

echo "🧹 To uninstall after testing:"
echo "   brew uninstall cliped-crossplatform"
echo ""

echo "✅ Formula validation complete!"
