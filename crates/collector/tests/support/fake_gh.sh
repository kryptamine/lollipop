#!/bin/sh
# Stands in for gh, with issues in a file. A `fail-<word>-<word>` file makes that command fail.
state="$FAKE_GH_STATE"
command="$1 $2"
shift 2
if [ -e "$state/fail-$(printf '%s' "$command" | tr ' ' '-')" ]; then
  echo "$command is failing" >&2
  exit 1
fi

case "$command" in
"issue list")
  while [ "$#" -gt 0 ]; do
    case "$1" in
    --state) listed="$2" ;;
    esac
    shift
  done
  if [ -e "$state/issues" ]; then
    sed 's/: .*//' "$state/issues"
  fi
  # Like the real one, this lists open issues unless told otherwise.
  if [ "$listed" = all ] && [ -e "$state/closed-issues" ]; then
    sed 's/: .*//' "$state/closed-issues"
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
