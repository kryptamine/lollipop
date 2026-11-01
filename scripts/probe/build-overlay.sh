#!/usr/bin/env bash
# Bundles the overlay into .probe-logs/Lollipop.app without needing tauri-cli.
# Usage: scripts/probe/build-overlay.sh [release|debug]
set -euo pipefail

profile="${1:-release}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
flags=(--manifest-path "$repo/src-tauri/Cargo.toml" --features tauri/custom-protocol)
[ "$profile" = release ] && flags+=(--release)
cargo build "${flags[@]}"

app="$repo/.probe-logs/Lollipop.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS"
cp "$repo/src-tauri/target/$profile/lollipop" "$app/Contents/MacOS/lollipop"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>lollipop</string>
  <key>CFBundleIdentifier</key><string>dev.lollipop.overlay</string>
  <key>CFBundleName</key><string>Lollipop</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
</dict>
</plist>
PLIST
codesign --force --sign - "$app"
echo "$app"
