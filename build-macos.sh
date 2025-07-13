#!/bin/bash
# macOS Universal Build Script for Cliped Cross-Platform
# Builds a universal binary that prevents "app is damaged" errors

set -e

# Configuration - Update these with your Apple Developer credentials
DEVELOPER_ID_APPLICATION="Developer ID Application: Alfred Onuada (UQCXJ5M9UP)"
APPLE_ID="shepherdumana2@gmail.com"
APP_SPECIFIC_PASSWORD="wpnl-puyl-ixyw-rhqk"
TEAM_ID="UQCXJ5M9UP"

APP_NAME="Cliped"
BUNDLE_ID="com.cliped-crossplatform.app"
VERSION="1.0.0"

echo "🚀 Building Cliped Cross-Platform for macOS (Universal)..."

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

# Add required targets
echo "🎯 Adding build targets..."
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf src-tauri/target/*/release/bundle
rm -rf dist
rm -rf *.dmg
rm -rf *.app

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🔨 Building frontend..."
npm run build

# Function to create clean app bundle without resource forks
create_clean_app_bundle() {
    local app_path="$1"
    local intel_binary="$2"
    local arm_binary="$3"
    
    echo "🏗️  Creating clean app bundle: $app_path"
    
    # Remove existing bundle
    rm -rf "$app_path"
    
    # Create app structure
    mkdir -p "$app_path/Contents/MacOS"
    mkdir -p "$app_path/Contents/Resources"
    
    # Create universal binary
    lipo -create "$intel_binary" "$arm_binary" -output "$app_path/Contents/MacOS/$APP_NAME"
    chmod +x "$app_path/Contents/MacOS/$APP_NAME"
    
    # Create Info.plist
    cat > "$app_path/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
EOF
    
    # Copy icon
    if [ -f "src-tauri/icons/icon.icns" ]; then
        cp "src-tauri/icons/icon.icns" "$app_path/Contents/Resources/"
    fi
    
    # Clean extended attributes and resource forks
    find "$app_path" -exec xattr -c {} \; 2>/dev/null || true
    find "$app_path" -name "._*" -delete 2>/dev/null || true
    find "$app_path" -name ".DS_Store" -delete 2>/dev/null || true
    
    # Use dot_clean if available
    if command -v dot_clean &> /dev/null; then
        dot_clean -m "$app_path" 2>/dev/null || true
    fi
}

# Build for Intel (x86_64)
echo "🔨 Building for Intel (x86_64)..."
cd src-tauri
cargo build --release --target x86_64-apple-darwin
cd ..

# Build for Apple Silicon (ARM64)
echo "🔨 Building for Apple Silicon (ARM64)..."
cd src-tauri
cargo build --release --target aarch64-apple-darwin
cd ..

# Check if both binaries exist
INTEL_BINARY="src-tauri/target/x86_64-apple-darwin/release/$APP_NAME"
ARM_BINARY="src-tauri/target/aarch64-apple-darwin/release/$APP_NAME"

if [ ! -f "$INTEL_BINARY" ]; then
    echo "❌ Intel binary not found at $INTEL_BINARY"
    exit 1
fi

if [ ! -f "$ARM_BINARY" ]; then
    echo "❌ ARM binary not found at $ARM_BINARY"
    exit 1
fi

# Create distribution directory
mkdir -p dist

# Create universal app bundle
UNIVERSAL_APP_PATH="dist/$APP_NAME.app"
create_clean_app_bundle "$UNIVERSAL_APP_PATH" "$INTEL_BINARY" "$ARM_BINARY"

echo "✅ Universal app bundle created!"

# Verify architecture
echo "🔍 Verifying universal binary..."
lipo -info "$UNIVERSAL_APP_PATH/Contents/MacOS/$APP_NAME"

# Sign the app if certificate is available
if security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
    echo "🔐 Code signing app..."
    
    # Create a temporary clean copy for signing
    TEMP_APP_PATH="/tmp/$APP_NAME-universal.app"
    rm -rf "$TEMP_APP_PATH"
    cp -R "$UNIVERSAL_APP_PATH" "$TEMP_APP_PATH"
    
    # Clean extended attributes
    find "$TEMP_APP_PATH" -exec xattr -c {} \; 2>/dev/null || true
    
    # Sign the binary
    codesign --force --verify --verbose --sign "$DEVELOPER_ID_APPLICATION" \
        --options runtime \
        "$TEMP_APP_PATH/Contents/MacOS/$APP_NAME"
    
    # Sign the app bundle with entitlements if available
    ENTITLEMENTS_FILE="src-tauri/entitlements.plist"
    if [ -f "$ENTITLEMENTS_FILE" ]; then
        echo "📋 Using entitlements: $ENTITLEMENTS_FILE"
        codesign --force --verify --verbose --sign "$DEVELOPER_ID_APPLICATION" \
            --entitlements "$ENTITLEMENTS_FILE" \
            --options runtime \
            "$TEMP_APP_PATH"
    else
        codesign --force --verify --verbose --sign "$DEVELOPER_ID_APPLICATION" \
            --options runtime \
            "$TEMP_APP_PATH"
    fi
    
    # Verify signature
    if codesign --verify --deep --strict --verbose=2 "$TEMP_APP_PATH" 2>/dev/null; then
        echo "✅ Code signing successful!"
        
        # Replace original with signed version
        rm -rf "$UNIVERSAL_APP_PATH"
        mv "$TEMP_APP_PATH" "$UNIVERSAL_APP_PATH"
        
        # Create signed DMG
        echo "� Creating signed DMG..."
        DMG_NAME="$APP_NAME-$VERSION-universal-macos.dmg"
        hdiutil create -volname "$APP_NAME Universal" -srcfolder "$UNIVERSAL_APP_PATH" -ov -format UDZO "dist/$DMG_NAME"
        
        # Sign DMG
        codesign --force --sign "$DEVELOPER_ID_APPLICATION" "dist/$DMG_NAME"
        
        # Optionally notarize
        echo "📋 To notarize (recommended for distribution):"
        echo "   xcrun notarytool submit dist/$DMG_NAME --apple-id $APPLE_ID --password $APP_SPECIFIC_PASSWORD --team-id $TEAM_ID --wait"
        echo "   xcrun stapler staple dist/$DMG_NAME"
        
    else
        echo "⚠️  Code signing failed, keeping unsigned version"
        rm -rf "$TEMP_APP_PATH"
    fi
else
    echo "⚠️  No Developer ID certificate found, creating unsigned bundle"
fi

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
