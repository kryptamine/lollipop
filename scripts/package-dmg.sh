#!/usr/bin/env bash
# Builds the private-phase DMG from ADR 0007: ad-hoc signed, for this Mac only.
set -euo pipefail
cd "$(dirname "$0")/.."

pnpm install --frozen-lockfile
# "-" is codesign's ad-hoc identity; a Developer ID build sets its own.
pnpm tauri build --bundles dmg --config '{"bundle":{"macOS":{"signingIdentity":"-"}}}'

version=$(node -p 'require("./crates/app/tauri.conf.json").version')
arch=$(uname -m)
# Tauri names the DMG with Rust's architecture names, not macOS's.
case $arch in
  arm64) arch=aarch64 ;;
  x86_64) arch=x64 ;;
esac
dmg="target/release/bundle/dmg/Lollipop_${version}_${arch}.dmg"

# Tauri deletes the .app once the DMG holds it, so the check reads the image.
image_dir=$(mktemp -d)
trap 'diskutil eject "$image_dir" >/dev/null 2>&1; rmdir "$image_dir"' EXIT
diskutil image attach --mountOptions nobrowse --readOnly --mountPoint "$image_dir" "$dmg" >/dev/null
codesign --verify --deep --strict "$image_dir/Lollipop.app"
echo "$dmg"
