#!/bin/bash
# Linux Build Script for Cliped Cross-Platform

set -e

echo "🐧 Building Cliped Cross-Platform for Linux..."

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ This script is for Linux only"
    exit 1
fi

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install Node.js"
    echo "   Ubuntu/Debian: sudo apt install nodejs npm"
    echo "   Fedora: sudo dnf install nodejs npm"
    echo "   Arch: sudo pacman -S nodejs npm"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ cargo is not installed. Please install Rust"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for required system dependencies
echo "🔍 Checking system dependencies..."

MISSING_DEPS=()

if ! pkg-config --exists webkit2gtk-4.0; then
    MISSING_DEPS+=("webkit2gtk-4.0-dev")
fi

if ! pkg-config --exists gtk+-3.0; then
    MISSING_DEPS+=("libgtk-3-dev")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "❌ Missing system dependencies: ${MISSING_DEPS[*]}"
    echo ""
    echo "Install them with:"
    if command -v apt &> /dev/null; then
        echo "   sudo apt update"
        echo "   sudo apt install ${MISSING_DEPS[*]// /-dev }-dev"
    elif command -v dnf &> /dev/null; then
        echo "   sudo dnf install webkit2gtk4.0-devel gtk3-devel"
    elif command -v pacman &> /dev/null; then
        echo "   sudo pacman -S webkit2gtk gtk3"
    fi
    exit 1
fi

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build the app
echo "🔨 Building Cliped Cross-Platform..."
npm run tauri build

# Check if build succeeded
BUNDLE_DIR="src-tauri/target/release/bundle"
if [ ! -d "$BUNDLE_DIR" ]; then
    echo "❌ Build failed - no bundle directory found"
    exit 1
fi

echo "✅ Build successful!"

# Create distribution directory
mkdir -p dist

# Copy all built packages
if [ -d "$BUNDLE_DIR/deb" ]; then
    echo "📦 Found .deb package"
    cp "$BUNDLE_DIR/deb"/*.deb dist/
fi

if [ -d "$BUNDLE_DIR/appimage" ]; then
    echo "📦 Found AppImage"
    cp "$BUNDLE_DIR/appimage"/*.AppImage dist/
fi

if [ -d "$BUNDLE_DIR/rpm" ]; then
    echo "📦 Found .rpm package"
    cp "$BUNDLE_DIR/rpm"/*.rpm dist/
fi

# Make AppImage executable
if ls dist/*.AppImage 1> /dev/null 2>&1; then
    chmod +x dist/*.AppImage
fi

# Create AUR PKGBUILD
echo "📦 Creating AUR PKGBUILD..."
cat > dist/PKGBUILD << 'EOF'
pkgname=cliped-crossplatform
pkgver=1.0.0
pkgrel=1
pkgdesc="Beautiful cross-platform clipboard manager built with Rust and React"
arch=('x86_64')
url="https://github.com/shepherrrd/cliped-crossplatform"
license=('MIT')
depends=('webkit2gtk' 'gtk3')
makedepends=('nodejs' 'npm' 'rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')  # Update with actual SHA256

build() {
    cd "$pkgname-$pkgver"
    npm install
    npm run tauri build
}

package() {
    cd "$pkgname-$pkgver"
    
    # Install binary (note: binary is named 'cliped' even though package is 'cliped-crossplatform')
    install -Dm755 "src-tauri/target/release/cliped" "$pkgdir/usr/bin/cliped"
    
    # Install desktop file
    install -Dm644 "src-tauri/target/release/bundle/deb/cliped.desktop" \
                   "$pkgdir/usr/share/applications/cliped.desktop"
    
    # Install icon
    install -Dm644 "src-tauri/icons/icon.png" \
                   "$pkgdir/usr/share/pixmaps/cliped.png"
}
EOF

echo "✅ Linux build complete!"
echo ""
echo "📁 Distribution files:"
ls -la dist/
echo ""
echo "📦 Package Installation:"

if ls dist/*.deb 1> /dev/null 2>&1; then
    echo "   Debian/Ubuntu: sudo dpkg -i dist/*.deb"
fi

if ls dist/*.rpm 1> /dev/null 2>&1; then
    echo "   Fedora/RHEL: sudo rpm -i dist/*.rpm"
fi

if ls dist/*.AppImage 1> /dev/null 2>&1; then
    echo "   AppImage: ./dist/*.AppImage"
fi

echo ""
echo "🏗️ For AUR (Arch User Repository):"
echo "   1. Update PKGBUILD with correct source URL and SHA256"
echo "   2. Submit to AUR: https://aur.archlinux.org/"
echo ""
echo "🚀 To test locally:"
if ls dist/*.AppImage 1> /dev/null 2>&1; then
    echo "   ./$(ls dist/*.AppImage | head -1)"
fi
