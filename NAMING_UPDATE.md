# Project Naming Update Summary

## Repository & Package Names Changed to: `cliped-crossplatform`
## Installed Binary Name Remains: `cliped`

### Files Updated:

1. **README.md**
   - Updated all GitHub repository URLs to `cliped-crossplatform`
   - Updated Homebrew tap name to `shepherrrd/cliped-crossplatform`
   - Updated package names in installation instructions
   - Updated AUR package name to `cliped-crossplatform`

2. **package.json**
   - Changed name from `"cliped"` to `"cliped-crossplatform"`

3. **src-tauri/tauri.conf.json**
   - Updated productName to "Cliped" (proper capitalization)
   - Updated identifier to `com.cliped-crossplatform.app`

4. **Build Scripts**
   - **build-macos.sh**: Updated Homebrew formula to `ClipedCrossplatform`
   - **build-linux.sh**: Updated AUR PKGBUILD to use `cliped-crossplatform`
   - **build-windows.bat**: Updated title and descriptions

### Installation Commands:

#### macOS (Homebrew)
```bash
brew tap shepherrrd/cliped-crossplatform
brew install cliped-crossplatform
cliped  # Run the app
```

#### Linux (AUR)
```bash
yay -S cliped-crossplatform
cliped  # Run the app
```

#### Linux (Debian/Ubuntu)
```bash
wget https://github.com/shepherrrd/cliped-crossplatform/releases/latest/download/cliped_amd64.deb
sudo dpkg -i cliped_amd64.deb
cliped  # Run the app
```

### Key Points:
- Repository/package name: `cliped-crossplatform`
- Installed binary name: `cliped`
- Users run the app with: `cliped`
- This avoids conflicts with the existing Swift version
- All package managers will show `cliped-crossplatform` in search/install
- The actual executable remains simple: `cliped`
