#!/bin/bash
# macOS Build Script for Cliped Cross-Platform

set -e

echo "🚀 Building Cliped Cross-Platform for macOS..."

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script is for macOS only"
    exit 1
fi

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install Node.js"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ cargo is not installed. Please install Rust"
    exit 1
fi

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build the app
echo "🔨 Building Cliped Cross-Platform..."
npm run tauri build

# Check if build succeeded
if [ ! -d "src-tauri/target/release/bundle/macos" ]; then
    echo "❌ Build failed - no macOS bundle found"
    exit 1
fi

APP_PATH="src-tauri/target/release/bundle/macos/Cliped.app"
DMG_PATH="src-tauri/target/release/bundle/dmg/Cliped.dmg"

echo "✅ Build successful!"
echo "📱 App bundle: $APP_PATH"

# Create distribution directory
mkdir -p dist

# Copy app bundle
cp -r "$APP_PATH" dist/

# Create tarball for Homebrew
echo "📦 Creating distribution tarball..."
cd dist
tar -czf cliped-macos.tar.gz Cliped.app
cd ..

# Calculate SHA256 for Homebrew formula
SHA256=$(shasum -a 256 dist/cliped-macos.tar.gz | cut -d' ' -f1)

echo "🍺 Homebrew Formula SHA256: $SHA256"

# Create Homebrew formula template
cat > dist/cliped-crossplatform.rb << EOF
class ClipedCrossplatform < Formula
  desc "Beautiful cross-platform clipboard manager"
  homepage "https://github.com/shepherrrd/cliped-crossplatform"
  url "https://github.com/shepherrrd/cliped-crossplatform/releases/download/v1.0.0/cliped-macos.tar.gz"
  sha256 "$SHA256"
  version "1.0.0"

  def install
    prefix.install "Cliped.app"
    bin.write_exec_script "#{prefix}/Cliped.app/Contents/MacOS/Cliped"
  end

  def caveats
    <<~EOS
      To start Cliped:
        cliped

      Or run directly:
        open #{prefix}/Cliped.app
    EOS
  end

  test do
    assert_predicate prefix/"Cliped.app", :exist?
  end
end
EOF

echo "✅ macOS build complete!"
echo ""
echo "📁 Distribution files:"
echo "   - App Bundle: dist/Cliped.app"
echo "   - Tarball: dist/cliped-macos.tar.gz"
echo "   - Homebrew Formula: dist/cliped-crossplatform.rb"
echo ""
echo "🍺 To publish to Homebrew:"
echo "   1. Upload cliped-macos.tar.gz to GitHub Releases"
echo "   2. Update the URL in cliped-crossplatform.rb"
echo "   3. Submit a PR to homebrew-core or create your own tap"
echo ""
echo "🚀 To test locally:"
echo "   open dist/Cliped.app"
