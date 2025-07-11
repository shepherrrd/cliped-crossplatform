class ClipedCrossplatform < Formula
  desc "Beautiful cross-platform clipboard manager"
  homepage "https://github.com/shepherrrd/cliped-crossplatform"
  url "https://github.com/shepherrrd/cliped-crossplatform/releases/download/v1.0.0/cliped-macos.tar.gz"
  sha256 "f5e49ceda921df88665056a4787bd33a37b331aa6e870dd32fe2d109020e0fc5"
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
