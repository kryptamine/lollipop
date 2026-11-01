#!/bin/sh
# Stands in for gh, with the Release as a directory. A `fail-<word>-<word>` file makes
# that command fail. Upload with --clobber deletes first, as the real one does.
state="$FAKE_GH_STATE"
assets="$state/assets"
command="$1 $2"
shift 2
if [ -e "$state/fail-$(printf '%s' "$command" | tr ' ' '-')" ]; then
  echo "$command is failing" >&2
  exit 1
fi

case "$command" in
"release view")
  if [ ! -d "$assets" ]; then
    echo "release not found" >&2
    exit 1
  fi
  ls "$assets"
  ;;
"release create")
  echo "$1" >"$state/tag"
  mkdir "$assets"
  ;;
"release download")
  while [ "$#" -gt 0 ]; do
    case "$1" in
    --pattern) name="$2" ;;
    --output) output="$2" ;;
    esac
    shift
  done
  cp "$assets/$name" "$output"
  ;;
"release upload")
  name=$(basename "$2")
  rm -f "$assets/$name"
  if [ -e "$state/lose-upload-$name" ]; then
    echo "upload of $name failed" >&2
    exit 1
  fi
  cp "$2" "$assets/$name"
  ;;
"release delete-asset")
  rm "$assets/$2"
  ;;
"issue list")
  if [ -e "$state/issues" ]; then
    sed 's/: .*//' "$state/issues"
  fi
  ;;
"issue create")
  while [ "$#" -gt 0 ]; do
    case "$1" in
    --title) title="$2" ;;
    --body) body="$2" ;;
    esac
    shift
  done
  printf '%s: %s\n' "$title" "$body" >>"$state/issues"
  ;;
*)
  echo "fake gh does not know $command" >&2
  exit 1
  ;;
esac
