# 📋 Cliped Cross-Platform - Beautiful Cross-Platform Clipboard Manager

A modern, beautiful clipboard manager built with Rust (Tauri) and React. Features a stunning frosted glass UI and seamless clipboard monitoring across all platforms.

![Cliped Screenshot](https://raw.githubusercontent.com/shepherrrd/cliped-crossplatform/main/assets/screenshot.png)

## ✨ Features

- 🎨 **Stunning Frosted Glass UI** - Modern, translucent interface that adapts to your system
- 📋 **Real-time Clipboard Monitoring** - Automatically captures clipboard changes in LIFO order
- 🔍 **Smart Search** - Instantly find clipboard items with fuzzy search
- ⚡ **Lightning Fast** - Built with Rust for maximum performance
- 🖱️ **Drag & Drop** - Draggable window with intuitive controls
- ✅ **Confirmation Prompts** - Safe deletion with confirmation dialogs
- ↺ **Undo Support** - Restore deleted items with one click
- 🔔 **Smart Notifications** - Visual feedback for all actions
- 🎯 **LIFO Order** - Most recent items appear first
- 🌍 **Cross-Platform** - Works on macOS, Windows, and Linux
- 📱 **Minimizable** - Standard window controls for seamless workflow

## 🚀 Quick Start

### Prerequisites

- **Node.js** (v18 or later)
- **Rust** (latest stable)
- **Platform-specific requirements:**
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft C++ Build Tools
  - **Linux**: Development packages (see below)

### Installation Options

#### Option 1: Install via Package Manager (Recommended)

##### macOS (Homebrew)
```bash
# Add the tap for the cross-platform version
brew tap shepherrrd/cliped-crossplatform

# Install Cliped Cross-Platform  
brew install cliped-crossplatform

# Run (binary is installed as 'cliped')
cliped
```

##### Linux (Debian/Ubuntu)
```bash
# Download and install .deb package from cliped-crossplatform repo
wget https://github.com/shepherrrd/cliped-crossplatform/releases/latest/download/cliped_amd64.deb
sudo dpkg -i cliped_amd64.deb

# Run (binary is installed as 'cliped')
cliped
```

##### Linux (Arch/Manjaro)
```bash
# Install from AUR (cliped-crossplatform package)
yay -S cliped-crossplatform
# or
paru -S cliped-crossplatform

# Run (binary is installed as 'cliped')
cliped
```

#### Option 2: Download Pre-built Binaries

Visit the [Releases page](https://github.com/shepherrrd/cliped-crossplatform/releases) and download the appropriate binary for your platform:

- **macOS**: `Cliped.app.tar.gz`
- **Windows**: `Cliped_x64_en-US.msi`
- **Linux**: `cliped_amd64.AppImage` or `cliped_amd64.deb`

#### Option 3: Build from Source

```bash
# Clone the cliped-crossplatform repository
git clone https://github.com/shepherrrd/cliped-crossplatform.git
cd cliped-crossplatform

# Install dependencies
npm install

# Build and run in development mode
npm run tauri dev

# Build for production (binary will be named 'cliped')
npm run tauri build
```

## 🔧 Development Setup

### Prerequisites for Development

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via Homebrew)
brew install node
```

#### Windows
```bash
# Install Rust
# Visit https://rustup.rs/ and follow the instructions

# Install Node.js
# Download from https://nodejs.org/

# Install Microsoft C++ Build Tools
# Download from https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

#### Linux (Ubuntu/Debian)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install development dependencies
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    javascriptcoregtk-4.1 \
    libsoup-3.0 \
    webkit2gtk-4.1
```

#### Linux (Fedora)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
sudo dnf install nodejs npm

# Install development dependencies
sudo dnf install -y \
    webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

#### Linux (Arch)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js and dependencies
sudo pacman -Syu
sudo pacman -S nodejs npm webkit2gtk base-devel curl wget file openssl appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg libvips
```

### Development Commands

```bash
# Clone and setup
git clone https://github.com/shepherrrd/cliped-crossplatform.git
cd cliped-crossplatform
npm install

# Development server (hot reload)
npm run tauri dev

# Build for production
npm run tauri build

# Run tests
npm test
cargo test

# Format code
npm run format
cargo fmt

# Lint code
npm run lint
cargo clippy
```

## 🎨 Usage

1. **Launch Cliped** - The app starts monitoring your clipboard automatically
2. **Copy anything** - Text, images, code - it's all captured in real-time
3. **Search items** - Use the search bar to quickly find what you need
4. **Click to restore** - Click any item to copy it back to your clipboard
5. **Delete safely** - Confirmation prompts protect against accidental deletions
6. **Undo mistakes** - Restore deleted items with the undo button
7. **Drag to move** - Click and drag the header to reposition the window
8. **Minimize when needed** - Use standard window controls to minimize

### Keyboard Shortcuts

- **Search**: Focus on the search bar and start typing
- **Escape**: Clear search or close confirmations
- **Enter**: Select the first search result

## 📁 Project Structure

```
cliped-crossplatform/
├── src/                          # React frontend
│   ├── Components/
│   │   ├── ClipboardItem.tsx     # Individual clipboard item
│   │   ├── ClipboardList.tsx     # Main list component
│   │   └── Notification.tsx     # Notification system
│   ├── Hooks/
│   │   └── useClipboard.ts       # Main clipboard logic
│   ├── Styles/
│   │   └── styles.css           # Frosted glass styling
│   ├── App.tsx                  # Main app component
│   ├── main.tsx                 # React entry point
│   └── types.ts                 # TypeScript definitions
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── main.rs             # Tauri app & clipboard monitoring
│   │   └── lib.rs              # Library exports
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri configuration
│   └── build.rs                # Build script
├── public/                      # Static assets
├── package.json                 # Node.js dependencies
└── README.md                    # This file
```

## 🔧 Configuration

### Window Settings

Edit `src-tauri/tauri.conf.json` to customize window behavior:

```json
{
  "app": {
    "windows": [{
      "width": 500,
      "height": 650,
      "transparent": true,
      "decorations": true,
      "alwaysOnTop": true,
      "resizable": false
    }]
  }
}
```

### Clipboard Settings

Modify clipboard behavior in `src-tauri/src/main.rs`:

```rust
// Maximum items to store (default: 100)
if history.len() > 100 {
    history.truncate(100);
}

// Polling interval (default: 500ms)
tokio::time::sleep(Duration::from_millis(500)).await;
```

## 🏗️ Building for Distribution

### macOS App Bundle

```bash
# Build the app
npm run tauri build

# The .app bundle will be in src-tauri/target/release/bundle/macos/
# Create a .dmg for distribution
npm run tauri build -- --target universal-apple-darwin
```

### Windows Installer

```bash
# Build MSI installer
npm run tauri build

# Output: src-tauri/target/release/bundle/msi/Cliped_x64_en-US.msi
```

### Linux Packages

```bash
# Build all Linux formats
npm run tauri build

# Output files:
# - src-tauri/target/release/bundle/deb/cliped_1.0.0_amd64.deb
# - src-tauri/target/release/bundle/appimage/cliped_1.0.0_amd64.AppImage
```

## 📦 Publishing

### Homebrew (macOS)

1. **Create a Homebrew tap**:
```bash
# Create the repository
mkdir homebrew-cliped
cd homebrew-cliped

# Create formula
mkdir Formula
cat > Formula/cliped-crossplatform.rb << 'EOF'
class ClipedCrossplatform < Formula
  desc "Beautiful cross-platform clipboard manager"
  homepage "https://github.com/shepherrrd/cliped-crossplatform"
  url "https://github.com/shepherrrd/cliped-crossplatform/releases/download/v1.0.0/cliped-macos.tar.gz"
  sha256 "YOUR_SHA256_HERE"
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
```

2. **Publish the tap**:
```bash
git add .
git commit -m "Add Cliped formula"
git push origin main
```

3. **Users can then install with**:
```bash
brew tap shepherrrd/cliped-crossplatform https://github.com/shepherrrd/cliped-crossplatform.git
brew install cliped-crossplatform
```

### AUR (Arch Linux)

1. **Create PKGBUILD**:
```bash
cat > PKGBUILD << 'EOF'
pkgname=cliped-crossplatform
pkgver=1.0.0
pkgrel=1
pkgdesc="Beautiful cross-platform clipboard manager"
arch=('x86_64')
url="https://github.com/shepherrrd/cliped-crossplatform"
license=('MIT')
depends=('webkit2gtk' 'gtk3')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('YOUR_SHA256_HERE')

build() {
  cd "$pkgname-$pkgver"
  npm install
  npm run tauri build
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 src-tauri/target/release/cliped "$pkgdir/usr/bin/cliped"
}
EOF
```

### APT Repository (Ubuntu/Debian)

1. **Build .deb package**:
```bash
npm run tauri build
```

2. **Create APT repository**:
```bash
# Create repository structure
mkdir -p apt-repo/pool/main/c/cliped-crossplatform
cp src-tauri/target/release/bundle/deb/*.deb apt-repo/pool/main/c/cliped-crossplatform/

# Generate Packages file
cd apt-repo
dpkg-scanpackages pool/ /dev/null | gzip -9c > Packages.gz

# Create Release file
cat > Release << EOF
Archive: stable
Component: main
Origin: Cliped
Label: Cliped
Architecture: amd64
EOF
```

## 🔒 Security

- **Local Storage**: All clipboard data is stored locally on your device
- **No Network Access**: The app doesn't send data over the internet
- **Permissions**: Only requires clipboard access permissions
- **Open Source**: Full source code available for audit

## 🐛 Troubleshooting

### Common Issues

#### "Failed to initialize clipboard monitoring"
```bash
# Linux: Install required packages
sudo apt install xclip xsel  # Debian/Ubuntu
sudo dnf install xclip xsel  # Fedora
sudo pacman -S xclip xsel    # Arch
```

#### "App won't start"
```bash
# Check if port 1420 is available
lsof -i :1420

# Kill any conflicting processes
pkill -f "cliped"
```

#### "Build fails on Linux"
```bash
# Install missing webkit dependencies
sudo apt install libwebkit2gtk-4.0-dev libgtk-3-dev
```

### Performance Tuning

```bash
# Reduce memory usage by limiting history
# Edit src-tauri/src/main.rs and change:
if history.len() > 50 {  // Reduced from 100
    history.truncate(50);
}
```

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- **Rust Code**: Follow `cargo fmt` and `cargo clippy` standards
- **TypeScript**: Use strict typing and follow ESLint rules
- **UI**: Maintain the frosted glass aesthetic
- **Performance**: Keep the app lightweight and responsive

## � Naming Convention

This project uses a specific naming convention to distinguish it from other versions:

- **Repository/Website Name**: `cliped-crossplatform`
- **Package Names**: 
  - Homebrew: `cliped-crossplatform`
  - AUR: `cliped-crossplatform` 
  - Debian/Ubuntu: Package built from `cliped-crossplatform` repo
- **Installed Binary**: `cliped` (for user convenience)
- **App Display Name**: "Cliped" (clean UI name)

This allows multiple versions to coexist in package managers while providing a simple `cliped` command to users.

## �📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing Rust-based app framework
- [React](https://reactjs.org/) - For the frontend framework
- [Arboard](https://crates.io/crates/arboard) - For cross-platform clipboard access
- [Tokio](https://tokio.rs/) - For async runtime

## 📧 Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/shepherrrd/cliped-crossplatform/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/shepherrrd/cliped-crossplatform/discussions)
- 📝 **Documentation**: [Wiki](https://github.com/shepherrrd/cliped-crossplatform/wiki)

---

Made with ❤️ by [shepherrrd](https://github.com/shepherrrd)
