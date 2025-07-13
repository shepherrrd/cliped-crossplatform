# 📋 Cliped Cross-Platform - Beautiful Cross-Platform Clipboard Manager

A modern, beautiful clipboard manager built with Rust (Tauri) and React. Features a stunning frosted glass UI and seamless clipboard monitoring across all platforms.

![Cliped Screenshot](https://raw.githubusercontent.com/shepherrrd/cliped-crossplatform/main/assets/screenshot.png)

## ✨ Features

- 🎨 **Stunning Froste## 📋 Naming Convention

This project uses a specific naming convention:

- **Repository Name**: `cliped-crossplatform`
- **App Display Name**: "Cliped" (clean UI name)
- **Binary Name**: `cliped` (for user convenience)

This allows clear identification while providing a simple user experience.- Modern, translucent interface that adapts to your system
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

### Installation

Visit the [Releases page](https://github.com/shepherrrd/cliped-crossplatform/releases) and download the appropriate installer for your operating system:

- **🍎 macOS**: Download the `.dmg` file
- **🪟 Windows**: Download the `.msi` installer  
- **🐧 Linux**: Download the `.deb` package or `.AppImage`

#### Installation Instructions

##### macOS
1. Download the `.dmg` file from releases
2. Open the DMG and drag Cliped to Applications
3. Right-click on Cliped and select "Open" (first time only)

##### Windows  
1. Download the `.msi` installer from releases
2. Run the installer and follow the setup wizard
3. Launch Cliped from the Start menu

##### Linux (Debian/Ubuntu)
```bash
# Download the .deb file from releases, then:
sudo dpkg -i cliped_*.deb

# Or for AppImage:
chmod +x cliped_*.AppImage
./cliped_*.AppImage
```

### Build from Source

If you prefer to build from source:

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

# Install Node.js
# Download from https://nodejs.org/ or use your preferred package manager
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

### macOS Code Signing & Notarization (Required)

To prevent "This file is damaged" errors on user machines, macOS apps must be **code signed and notarized**.

#### Quick Setup

1. **Configure signing credentials:**
   ```bash
   ./setup-signing.sh
   ```

2. **Build signed and notarized DMG:**
   ```bash
   # Universal binary (Intel + ARM64) - recommended
   ./build-mac-universal.sh

   # Or build for specific architecture:
   ./build-mac-intel.sh      # Intel Macs only
   ./build-mac-arm64.sh      # Apple Silicon only
   ```

#### Manual Setup

If you prefer to configure manually, edit the build scripts with your:
- Developer ID Application certificate
- Apple ID and app-specific password
- Team ID

See `MACOS_DISTRIBUTION_FIX.md` for detailed instructions.

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

## 📦 Releases

All releases are available on the [GitHub Releases page](https://github.com/shepherrrd/cliped-crossplatform/releases). 

Each release includes:
- **🍎 macOS**: `.dmg` installer (Intel and Apple Silicon)
- **🪟 Windows**: `.msi` installer (x64)  
- **🐧 Linux**: `.deb` package and `.AppImage` (x64)

### For Developers

To create a new release:

```bash
# Build for your current platform
npm run tauri build

# The built files will be in:
# - macOS: src-tauri/target/release/bundle/macos/ and src-tauri/target/release/bundle/dmg/
# - Windows: src-tauri/target/release/bundle/msi/
# - Linux: src-tauri/target/release/bundle/deb/ and src-tauri/target/release/bundle/appimage/
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
