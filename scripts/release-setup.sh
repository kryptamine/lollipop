#!/usr/bin/env bash
# Walks the owner through the one-time setup of the public release pipeline.
# Above the "STAGES" marker is the shared /wizard library, kept unedited.

set -euo pipefail

# ──────────────────────────────────────────────────────────────────────────
# Wizard library: delightful, consistent UX, identical across every wizard.
# ──────────────────────────────────────────────────────────────────────────

if [[ -t 1 ]] && command -v tput >/dev/null 2>&1 && [[ "$(tput colors 2>/dev/null || echo 0)" -ge 8 ]]; then
  BOLD=$(tput bold); DIM=$(tput dim); RESET=$(tput sgr0)
  BLUE=$(tput setaf 4); GREEN=$(tput setaf 2); YELLOW=$(tput setaf 3); RED=$(tput setaf 1)
else
  BOLD=""; DIM=""; RESET=""; BLUE=""; GREEN=""; YELLOW=""; RED=""
fi

# Author sets this at the top of the stages section.
TOTAL_STAGES=0

_STAGE_INDEX=0
ENV_FILE="${ENV_FILE:-.env}"
WRITTEN_ENV=()    # KEYs written to ENV_FILE this run
WRITTEN_SECRET=() # secret NAMEs set this run
SKIPPED=()        # things we couldn't do (e.g. gh missing)

# _clear wipes the terminal so only the current step is on screen. No-op when
# output isn't a terminal, so piped logs stay readable.
_clear() {
  [[ -t 1 ]] || return 0
  if command -v tput >/dev/null 2>&1; then tput clear; else printf '\033[2J\033[3J\033[H'; fi
}

# banner "Title" shows the opening frame: what this wizard does.
banner() {
  _clear
  printf '\n%s%s  %s%s\n' "$BOLD" "$BLUE" "$1" "$RESET"
  printf '%s  %s stages%s\n\n' "$DIM" "$TOTAL_STAGES" "$RESET"
  printf '%s  You drive the browser; this wizard tells you exactly what to do and\n' "$DIM"
  printf '  captures the values you copy back. Stop any time with Ctrl-C and re-run\n'
  printf '  later, since it remembers values already saved.%s\n' "$RESET"
  pause "Ready to start?"
}

# stage "Name" clears the screen, then announces a stage and shows progress.
# Clearing keeps only the current step on screen.
stage() {
  _clear
  _STAGE_INDEX=$((_STAGE_INDEX + 1))
  printf '\n%s%s▸ Stage %s/%s · %s%s\n' \
    "$BOLD" "$BLUE" "$_STAGE_INDEX" "$TOTAL_STAGES" "$1" "$RESET"
}

# say "..." prints a plain instruction line.
say()  { printf '  %s\n' "$1"; }
# step "..." is a numbered-feeling action the human takes in the browser.
step() { printf '  %s•%s %s\n' "$BLUE" "$RESET" "$1"; }
note() { printf '  %s%s%s\n' "$DIM" "$1" "$RESET"; }
warn() { printf '  %s⚠ %s%s\n' "$YELLOW" "$1" "$RESET"; }

# open_url URL opens it in the human's browser, cross-platform incl. WSL.
open_url() {
  local url="$1"
  printf '  %s↗ opening%s %s\n' "$GREEN" "$RESET" "$url"
  { if   command -v wslview     >/dev/null 2>&1; then wslview "$url"
    elif command -v explorer.exe >/dev/null 2>&1; then explorer.exe "$url"
    elif command -v xdg-open    >/dev/null 2>&1; then xdg-open "$url"
    elif command -v open        >/dev/null 2>&1; then open "$url"
    else warn "couldn't open a browser; visit it manually: $url"; fi
  } >/dev/null 2>&1 || warn "couldn't open a browser, so visit it manually: $url"
}

# pause "msg" waits for the human to confirm they've done the manual part.
pause() {
  printf '  %s%s%s ' "$DIM" "${1:-Press Enter to continue}" "$RESET"
  read -r _ || true
}

# confirm "question" is a y/N gate; returns success on yes.
confirm() {
  local reply=""
  printf '  %s? %s [y/N] ' "$YELLOW" "$1"
  read -r reply || true
  [[ "$reply" =~ ^[Yy] ]]
}

# _existing KEY: current value of KEY in ENV_FILE, if any.
_existing() {
  [[ -f "$ENV_FILE" ]] || return 1
  local line; line=$(grep -E "^${1}=" "$ENV_FILE" | tail -n1) || return 1
  printf '%s' "${line#*=}"
}

# ask KEY "Prompt" reads a value into $KEY. Offers the existing .env value as
# a default on re-runs (Enter keeps it). Visible input (non-secret).
ask() {
  local key="$1" prompt="$2" current input
  current=$(_existing "$key" || true)
  if [[ -n "$current" ]]; then
    printf '  %s%s%s %s[Enter keeps current]%s ' "$BOLD" "$prompt" "$RESET" "$DIM" "$RESET"
  else
    printf '  %s%s%s ' "$BOLD" "$prompt" "$RESET"
  fi
  read -r input || true
  [[ -z "$input" && -n "$current" ]] && input="$current"
  printf -v "$key" '%s' "$input"
}

# ask_secret KEY "Prompt" is like ask, but input is hidden.
ask_secret() {
  local key="$1" prompt="$2" current input
  current=$(_existing "$key" || true)
  if [[ -n "$current" ]]; then
    printf '  %s%s%s %s[Enter keeps current]%s ' "$BOLD" "$prompt" "$RESET" "$DIM" "$RESET"
  else
    printf '  %s%s%s ' "$BOLD" "$prompt" "$RESET"
  fi
  read -rs input || true
  printf '\n'
  [[ -z "$input" && -n "$current" ]] && input="$current"
  printf -v "$key" '%s' "$input"
}

# write_env KEY VALUE upserts KEY=VALUE into ENV_FILE (creates it; replaces
# any existing line). Idempotent.
write_env() {
  local key="$1" value="$2" tmp
  touch "$ENV_FILE"
  tmp=$(mktemp)
  grep -vE "^${key}=" "$ENV_FILE" > "$tmp" || true
  printf '%s=%s\n' "$key" "$value" >> "$tmp"
  mv "$tmp" "$ENV_FILE"
  WRITTEN_ENV+=("$key")
  printf '  %s✓ wrote%s %s → %s\n' "$GREEN" "$RESET" "$key" "$ENV_FILE"
}

# set_secret NAME VALUE sets a GitHub Actions repo secret via gh. Falls back
# to a warning (and records it) if gh is unavailable or unauthenticated.
set_secret() {
  local name="$1" value="$2"
  if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    if printf '%s' "$value" | gh secret set "$name" >/dev/null 2>&1; then
      WRITTEN_SECRET+=("$name")
      printf '  %s✓ set%s GitHub secret %s\n' "$GREEN" "$RESET" "$name"
      return
    fi
  fi
  SKIPPED+=("GitHub secret $name (set it manually: gh secret set $name)")
  warn "skipped GitHub secret $name: gh not ready; set it later"
}

# set_var NAME VALUE sets a GitHub Actions repo variable (non-secret).
set_var() {
  local name="$1" value="$2"
  if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    if gh variable set "$name" --body "$value" >/dev/null 2>&1; then
      printf '  %s✓ set%s GitHub variable %s\n' "$GREEN" "$RESET" "$name"
      return
    fi
  fi
  SKIPPED+=("GitHub variable $name")
  warn "skipped GitHub variable $name, gh not ready; set it later"
}

# finish clears, then shows a closing summary of everything configured.
finish() {
  _clear
  printf '\n%s%s  ✓ Setup complete%s\n' "$BOLD" "$GREEN" "$RESET"
  (( ${#WRITTEN_ENV[@]} ))    && note "wrote ${#WRITTEN_ENV[@]} value(s) to $ENV_FILE: ${WRITTEN_ENV[*]}"
  (( ${#WRITTEN_SECRET[@]} )) && note "set ${#WRITTEN_SECRET[@]} GitHub secret(s): ${WRITTEN_SECRET[*]}"
  if (( ${#SKIPPED[@]} )); then
    printf '\n'; warn "still to do by hand:"
    for s in "${SKIPPED[@]}"; do note "  - $s"; done
  fi
  printf '\n'
}

# ──────────────────────────────────────────────────────────────────────────
# STAGES: the one-time setup behind .github/workflows/release.yml.
# ──────────────────────────────────────────────────────────────────────────

cd "$(dirname "$0")/.."
TOTAL_STAGES=7
RELEASES_REPO=kryptamine/lollipop-releases
UPDATER_KEY="$HOME/.tauri/lollipop-updater.key"
CONFIG=crates/app/tauri.conf.json

fail() {
  printf '  %s✗ %s%s\n' "$RED" "$1" "$RESET"
  exit 1
}

# need NAME stops the wizard before an empty value reaches a secret.
need() { [[ -n "${!1}" ]] || fail "$1 is empty, run the wizard again"; }

# A file read inside $(...) would not trip set -e, so read it here first.
read_file() { [[ -f "$1" ]] || fail "no file at $1"; cat "$1"; }

[[ -t 0 ]] || fail "run this in a terminal: it reads your answers"
gh auth status >/dev/null 2>&1 || fail "gh is not logged in: run gh auth login first"

banner "Lollipop public release setup"

stage "Updater signing key"
say "The app trusts an update only if this key signed it (ADR 0007)."
say "The private key goes to GitHub secrets and a password manager, nowhere else."
if [[ -f "$UPDATER_KEY" ]]; then
  note "found $UPDATER_KEY, reusing it"
elif confirm "Generate a new key pair at $UPDATER_KEY?"; then
  step "Choose a strong password when asked, and keep it."
  pnpm tauri signer generate --write-keys "$UPDATER_KEY"
fi
ask_secret TAURI_SIGNING_PRIVATE_KEY_PASSWORD "Type the key's password:"
UPDATER_PRIVATE_KEY=$(read_file "$UPDATER_KEY")
read_file "$UPDATER_KEY.pub" >/dev/null
need TAURI_SIGNING_PRIVATE_KEY_PASSWORD
set_secret TAURI_SIGNING_PRIVATE_KEY "$UPDATER_PRIVATE_KEY"
set_secret TAURI_SIGNING_PRIVATE_KEY_PASSWORD "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
node -e '
  const fs = require("fs");
  const [config, pubkey] = process.argv.slice(1);
  const parsed = JSON.parse(fs.readFileSync(config, "utf8"));
  parsed.plugins.updater.pubkey = fs.readFileSync(pubkey, "utf8").trim();
  fs.writeFileSync(config, JSON.stringify(parsed, null, 2) + "\n");
' "$CONFIG" "$UPDATER_KEY.pub"
printf '  %s✓ wrote%s the public key → %s\n' "$GREEN" "$RESET" "$CONFIG"
warn "Back up $UPDATER_KEY and its password: a lost key strands every install."
pause

stage "Public releases repository"
if gh repo view "$RELEASES_REPO" >/dev/null 2>&1; then
  note "$RELEASES_REPO already exists"
elif confirm "Create the public repository $RELEASES_REPO?"; then
  gh repo create "$RELEASES_REPO" --public --add-readme \
    --description "Signed macOS releases of Lollipop"
fi
pause

# Without the Apple Developer Program, builds go out ad-hoc signed (ADR 0012).
stage "Apple Developer Program"
APPLE_ENROLLED=no
if confirm "Is your Apple ID enrolled in the paid Apple Developer Program?"; then
  APPLE_ENROLLED=yes
else
  note "Skipping the certificate and notarization: releases will be ad-hoc signed."
  note "Enroll at https://developer.apple.com/programs/enroll/ and run this again."
fi
pause

stage "Developer ID Application certificate"
if [[ $APPLE_ENROLLED == yes ]]; then
  note "Needs the Account Holder role on the Apple Developer account."
  step "Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority."
  step "Enter your email, pick 'Saved to disk', save the .certSigningRequest."
  open_url "https://developer.apple.com/account/resources/certificates/add"
  step "Pick 'Developer ID Application', then the G2 Sub-CA, upload the request."
  step "Download the .cer and double-click it to add it to the login keychain."
  step "Keychain Access → My Certificates → right-click it → Export → .p12, set a password."
  ask CERTIFICATE_P12 "Path to the exported .p12:"
  ask_secret APPLE_CERTIFICATE_PASSWORD "The .p12 password:"
  CERTIFICATE_P12=${CERTIFICATE_P12/#\~/$HOME}
  read_file "$CERTIFICATE_P12" >/dev/null
  need APPLE_CERTIFICATE_PASSWORD
  set_secret APPLE_CERTIFICATE "$(base64 -i "$CERTIFICATE_P12")"
  set_secret APPLE_CERTIFICATE_PASSWORD "$APPLE_CERTIFICATE_PASSWORD"
  say "Identities in your keychain:"
  security find-identity -v -p codesigning | grep "Developer ID Application" || true
  ask APPLE_SIGNING_IDENTITY "Paste the quoted name, e.g. Developer ID Application: Name (TEAMID):"
  need APPLE_SIGNING_IDENTITY
  set_secret APPLE_SIGNING_IDENTITY "$APPLE_SIGNING_IDENTITY"
else
  note "skipped: no Apple Developer Program"
fi
pause

stage "App Store Connect API key for notarization"
if [[ $APPLE_ENROLLED == yes ]]; then
  open_url "https://appstoreconnect.apple.com/access/integrations/api"
  step "Team Keys → Generate API Key (+), name it 'lollipop-notary', access 'Developer'."
  step "Copy the Issuer ID shown above the table."
  ask APPLE_API_ISSUER "Issuer ID:"
  step "Copy the new key's Key ID."
  ask APPLE_API_KEY "Key ID:"
  step "Download the .p8: Apple offers it only once."
  ask API_KEY_P8 "Path to the downloaded AuthKey_*.p8:"
  API_KEY_P8_CONTENT=$(read_file "${API_KEY_P8/#\~/$HOME}")
  need APPLE_API_ISSUER
  need APPLE_API_KEY
  set_secret APPLE_API_ISSUER "$APPLE_API_ISSUER"
  set_secret APPLE_API_KEY "$APPLE_API_KEY"
  set_secret APPLE_API_KEY_P8 "$API_KEY_P8_CONTENT"
else
  note "skipped: no Apple Developer Program"
fi
pause

stage "Token to publish into $RELEASES_REPO"
open_url "https://github.com/settings/personal-access-tokens/new"
step "Fine-grained token, resource owner kryptamine, expiry as you like."
step "Repository access: Only select repositories → lollipop-releases."
step "Permissions → Repository → Contents: Read and write. Generate and copy."
ask_secret RELEASES_TOKEN "Paste the token:"
need RELEASES_TOKEN
set_secret RELEASES_TOKEN "$RELEASES_TOKEN"
pause

stage "Commit the public key"
say "$CONFIG now carries the updater public key."
step "Commit it on a branch and merge it before tagging the first release."
step "Then: bump the version, tag vX.Y.Z on main, push the tag."
pause

finish
