#!/bin/sh
# Stands in for rclone, with the remote "tables" as a directory. A `fail-rclone-<command>`
# file makes that command fail. As the real one does, copyto of a missing object exits 0.
state="$FAKE_GH_STATE"
remote="$state/remote"
command="$1"
shift
if [ -e "$state/fail-rclone-$command" ]; then
  echo "$command is failing" >&2
  exit 1
fi

on_disk() {
  case "$1" in
  tables:*) printf '%s/%s' "$remote" "${1#tables:}" ;;
  *) printf '%s' "$1" ;;
  esac
}

case "$command" in
lsf)
  if [ -d "$(on_disk "$1")" ]; then
    ls "$(on_disk "$1")"
  fi
  ;;
copyto)
  name=$(basename "$2")
  if [ "$3" = --header-upload ] && [ -e "$state/fail-upload-$name" ]; then
    echo "upload of $name failed" >&2
    exit 1
  fi
  if [ -e "$state/lose-download-$(basename "$1")" ] || [ ! -e "$(on_disk "$1")" ]; then
    exit 0
  fi
  mkdir -p "$(dirname "$(on_disk "$2")")"
  cp "$(on_disk "$1")" "$(on_disk "$2")"
  if [ "$3" = --header-upload ]; then
    printf '%s\n' "$4" >"$state/header-$name"
  fi
  ;;
deletefile)
  rm "$(on_disk "$1")"
  ;;
*)
  echo "fake rclone does not know $command" >&2
  exit 1
  ;;
esac
