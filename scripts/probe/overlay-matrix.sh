#!/usr/bin/env bash
# Cycles the overlay variants hands-free, announcing each aloud, so the player can stay in the game.
# Usage: scripts/probe/overlay-matrix.sh <fullscreen|borderless|windowed>
set -euo pipefail

display_mode="${1:?usage: overlay-matrix.sh <fullscreen|borderless|windowed>}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
binary="$repo/.probe-logs/Lollipop.app/Contents/MacOS/lollipop"
log="$repo/.probe-logs/overlay-matrix.ndjson"
read -r -a variants <<< "${PROBE_VARIANTS:-baseline accessory accessory-aux accessory-aux-shield}"
seconds_per_variant="${PROBE_SECONDS:-15}"
lead_in_seconds="${PROBE_LEAD_IN:-15}"

swiftc -o "$repo/.probe-logs/windows" "$repo/scripts/probe/windows.swift"

echo "Switch to the game now. Starting in ${lead_in_seconds}s."
sleep "$lead_in_seconds"

for index in "${!variants[@]}"; do
  variant="${variants[$index]}"
  say "variant $((index + 1))"
  LOLLIPOP_PROBE="$variant" "$binary" &
  pid=$!
  sleep "$seconds_per_variant"
  snapshot="$("$repo/.probe-logs/windows")"
  kill "$pid"
  wait "$pid" 2>/dev/null || true
  printf '{"displayMode":"%s","variant":"%s","windows":%s}\n' "$display_mode" "$variant" "$snapshot" >> "$log"
done
say "done"

echo "Variants in order: ${variants[*]}"
read -r -p "Which numbers drew over the game (e.g. 3 4, or none)? " seen
read -r -p "Any variant that took focus or broke clicks? " issues
printf '{"displayMode":"%s","seen":"%s","issues":"%s"}\n' "$display_mode" "$seen" "$issues" >> "$log"
echo "Logged to $log"
