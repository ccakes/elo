cask "elo" do
  version "0.2.4"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/ccakes/elo/releases/download/v#{version}/Elo_#{version}_universal.dmg"
  name "Elo"
  desc "Notepad-style calculator for natural math expressions"
  homepage "https://github.com/ccakes/elo"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Elo.app"

  zap trash: [
    "~/Library/Application Support/com.elo.calculator",
    "~/Library/Caches/com.elo.calculator",
    "~/Library/Preferences/com.elo.calculator.plist",
    "~/Library/Saved Application State/com.elo.calculator.savedState",
    "~/Library/WebKit/com.elo.calculator",
  ]
end
