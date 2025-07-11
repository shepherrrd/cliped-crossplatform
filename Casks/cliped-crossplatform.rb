cask "cliped-crossplatform" do
  version "1.0.0"
  sha256 "f5e49ceda921df88665056a4787bd33a37b331aa6e870dd32fe2d109020e0fc5"

  url "https://github.com/shepherrrd/cliped-crossplatform/releases/download/v#{version}/cliped-macos.tar.gz"
  name "Cliped Cross-Platform"
  desc "Beautiful cross-platform clipboard manager"
  homepage "https://github.com/shepherrrd/cliped-crossplatform"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Cliped.app"

  zap trash: [
    "~/Library/Application Support/com.cliped.app",
    "~/Library/Caches/com.cliped.app",
    "~/Library/Preferences/com.cliped.app.plist",
    "~/Library/Saved Application State/com.cliped.app.savedState",
  ]
end