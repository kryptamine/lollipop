#!/usr/bin/env bash
# Works through the ready-for-agent issues one at a time. The agent only commits;
# this script owns the gate, the push, the pull request and the merge.
# Usage: scripts/afk.sh [--dry-run]. AFK_BUDGET_USD caps one agent run (default 30).
set -euo pipefail

label=ready-for-agent
budget=${AFK_BUDGET_USD:-30}
max_gate_retries=2
branch_pattern='^(feat|fix|docs|test|refactor|perf|chore)/[a-z0-9-]+$'

root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
wt="$root/../$(basename "$root")-afk"
logs="$root/.afk-logs"
cd "$root"
repo=$(gh repo view --json nameWithOwner --jq .nameWithOwner)

schema='{"type":"object","required":["status"],"additionalProperties":false,"properties":{
  "status":{"enum":["done","needs_human"]},
  "question":{"type":"string"},
  "pr_title":{"type":"string"},
  "pr_body":{"type":"string"}}}'

# The commands the script runs itself stay denied, so main only moves through the gate.
# Not a security boundary: make, cargo and pnpm can run anything on this machine.
allowed=(Read Edit Write Glob Grep WebFetch WebSearch TodoWrite
  'Bash(git *)' 'Bash(gh issue view *)' 'Bash(gh issue comment *)'
  'Bash(make *)' 'Bash(cargo *)' 'Bash(pnpm *)' 'Bash(node *)' 'Bash(shellcheck *)'
  'Bash(ls *)' 'Bash(cat *)' 'Bash(head *)' 'Bash(tail *)' 'Bash(wc *)' 'Bash(rg *)'
  'Bash(grep *)' 'Bash(find *)' 'Bash(sed *)' 'Bash(awk *)' 'Bash(perl *)' 'Bash(jq *)'
  'Bash(mkdir *)' 'Bash(mv *)' 'Bash(cp *)' 'Bash(rm *)' 'Bash(chmod *)')
denied=('Bash(git push *)' 'Bash(gh pr *)')

merged=""
skipped=""
current=""
runs=0
answer=""
result=""

notify() {
  osascript -e 'on run argv' -e 'display notification (item 1 of argv) with title "Lollipop AFK"' \
    -e 'end run' "$1" >/dev/null 2>&1 || true
}

summary() {
  printf '\nMerged:%s\nSkipped:%s\nLogs: %s\n' "${merged:- none}" "${skipped:- none}" "$logs"
  notify "AFK run finished. Merged:${merged:- none}"
}

# Prints: number, open blocker count, title. GitHub counts only open blockers.
list_issues() {
  local n
  for n in $(gh issue list --label "$label" --state open --limit 100 --json number \
    --jq 'map(.number) | sort | .[]'); do
    gh api "repos/$repo/issues/$n" \
      --jq '[.number, (.issue_dependencies_summary.blocked_by // 0), .title] | @tsv'
  done
}

next_issue() {
  local n blocked title
  while IFS=$'\t' read -r n blocked title; do
    if [[ "$blocked" == 0 && " $skipped " != *" $n "* ]]; then
      echo "$n"
      return 0
    fi
  done < <(list_issues)
}

# Sets $answer. Returns 1 when the human skips the issue.
ask() {
  notify "Issue #$current needs you"
  printf '\a\n=== Issue #%s needs you ===\n%s\n\n' "$current" "$1"
  answer=""
  while [[ -z "$answer" ]]; do
    printf 'Reply (skip = next issue, stop = quit): '
    read -r answer </dev/tty || answer=stop
  done
  case "$answer" in
    stop) exit 0 ;;
    skip) return 1 ;;
  esac
}

# Sets $result to the run's final event, empty when the process died before it.
run_agent() {
  local session=$1 prompt=$2 log args
  runs=$((runs + 1))
  log="$logs/issue-$current-run-$runs.jsonl"
  args=(--output-format stream-json --verbose --json-schema "$schema"
    --permission-mode acceptEdits --max-budget-usd "$budget"
    --allowedTools "${allowed[@]}" --disallowedTools "${denied[@]}")
  if [[ -n "$session" ]]; then
    args+=(--resume "$session")
  fi
  (cd "$wt" && claude -p "$prompt" "${args[@]}" </dev/null) | tee "$log" |
    jq --unbuffered -r 'select(.type == "assistant") | .message.content[]? |
      if .type == "text" then .text elif .type == "tool_use" then "  -> \(.name)" else empty end' ||
    true
  result=$(jq -c 'select(.type == "result")' "$log" 2>/dev/null | tail -n 1)
}

field() {
  jq -r "$1 // \"\"" <<<"$result"
}

# Prints why the branch may not merge. make check is last, as it is the slow one.
gate() {
  local branch output
  branch=$(git -C "$wt" branch --show-current)
  if [[ ! "$branch" =~ $branch_pattern ]]; then
    echo "The worktree is on '$branch', not a branch named <type>/<kebab-summary>."
    return 1
  fi
  if [[ -n "$(git -C "$wt" status --porcelain)" ]]; then
    echo "The worktree has uncommitted changes:"
    git -C "$wt" status --short
    return 1
  fi
  git -C "$wt" fetch -q origin main
  if ! git -C "$wt" merge-base --is-ancestor origin/main HEAD; then
    echo "The branch is behind origin/main. Rebase it onto origin/main."
    return 1
  fi
  if [[ -z "$(git -C "$wt" rev-list origin/main..HEAD)" ]]; then
    echo "The branch has no commits on top of origin/main."
    return 1
  fi
  if git -C "$wt" log origin/main..HEAD --format=%B | grep -qi '^co-authored-by:'; then
    echo "A commit carries a co-author trailer, which AGENTS.md forbids. Remove it."
    return 1
  fi
  if ! output=$(make -C "$wt" check 2>&1); then
    echo "make check failed:"
    tail -n 80 <<<"$output"
    return 1
  fi
}

publish() {
  local branch=$1 title=$2 body=$3 url
  git -C "$wt" push -u origin "$branch" || return 1
  url=$(gh pr view "$branch" --json url --jq .url 2>/dev/null) ||
    url=$(gh pr create --base main --head "$branch" --title "$title" \
      --body "$body"$'\n\n'"Closes #$current") || return 1
  gh pr merge "$url" --merge || return 1
  echo "Merged $url"
  # gh's --delete-branch would switch to main, which the main checkout already holds.
  git -C "$wt" switch -q --detach
  git -C "$wt" push -q origin --delete "$branch" || true
  git -C "$wt" branch -qD "$branch" || true
  if [[ "$(gh issue view "$current" --json state --jq .state)" != CLOSED ]]; then
    gh issue close "$current" --comment "Merged in $url" || true
  fi
}

prepare_worktree() {
  git fetch -q origin main
  if [[ ! -d "$wt" ]]; then
    git worktree add -q --detach "$wt" origin/main
  fi
  # A stash, because an interrupted run may have left work worth keeping.
  if [[ -n "$(git -C "$wt" status --porcelain)" ]]; then
    git -C "$wt" stash push -q -u -m "afk leftovers before #$current"
  fi
  git -C "$wt" switch -q --detach origin/main
  make -C "$wt" install
}

implement_prompt() {
  cat <<EOF
You are working AFK on GitHub issue #$current, in a git worktree detached at origin/main.
A second agent reviews your branch, and it then merges with no human reading it.

1. Read the issue with \`gh issue view $current --comments\`, and the spec section it links
   (its path in this checkout is under docs/spec/). AGENTS.md governs how you work.
2. Cut a branch named <type>/<kebab-summary> and build every acceptance criterion, with
   the tests AGENTS.md asks for. Commit by its convention, with no co-author trailer or
   attribution line, and leave nothing uncommitted.
3. Make \`make check\` pass.

You cannot push, open or merge a pull request, and you do not close the issue: the
script does that once you report done. A criterion that asks to record a result on the
issue is done with \`gh issue comment\`.

Decide for yourself wherever the spec, the code and AGENTS.md give an answer or a sensible
default. Report status needs_human, with a self-contained question, only when you are
blocked on what a human alone can give: a decision the spec leaves open and that is costly
to get wrong, a credential or account, or a check in the real League client. Never fake
such a check or mark it done. Build everything else first, then ask.

When finished, report status done with pr_title (a commit-style subject) and pr_body
(why and how it was tested, no attribution, no closing keyword).
EOF
}

review_prompt() {
  cat <<EOF
You review the branch checked out here before it merges with no human reading it.
Read issue #$current with \`gh issue view $current --comments\`, the spec section it links, and
\`git diff origin/main...HEAD\`.

Check each acceptance criterion against the code and its tests, and the diff against
AGENTS.md. Look for real defects: a criterion not met, a test that proves nothing, a
broken rule, a bug. Fix what you find in new commits on this branch, with no co-author
trailer, and keep \`make check\` passing. Do not widen the scope or restyle working code.

Report status done when the branch is fit to merge, leaving pr_title and pr_body out.
Report needs_human, with a self-contained question, only for what a human alone can give.
EOF
}

# Returns 1 when the human skips the issue. Reaching the merge is the only success.
work_on() {
  local session="" previous="" prompt title="" body="" reviewed=0 retries=0 status failure branch
  current=$1
  prepare_worktree
  prompt=$(implement_prompt)
  while true; do
    run_agent "$session" "$prompt"
    session=$(field .session_id || true)
    session=${session:-$previous}
    previous=$session
    status=$(field .structured_output.status)
    case "$status" in
      needs_human)
        ask "$(field .structured_output.question)" || return 1
        prompt="The human answered: $answer"
        continue
        ;;
      done) ;;
      *)
        ask "The agent stopped with no result ($(field .subtype)). See $logs. Instructions to resume with?" || return 1
        prompt="Carry on. The human says: $answer"
        continue
        ;;
    esac

    if [[ "$reviewed" == 0 ]]; then
      title=$(field .structured_output.pr_title)
      body=$(field .structured_output.pr_body)
    fi
    if [[ -n "$title" ]] && failure=$(gate); then
      if [[ "$reviewed" == 0 ]]; then
        reviewed=1
        retries=0
        session=""
        prompt=$(review_prompt)
        continue
      fi
      break
    fi
    failure=${failure:-Report pr_title and pr_body with status done.}

    retries=$((retries + 1))
    if ((retries > max_gate_retries)); then
      ask "The gate still fails after $max_gate_retries retries:"$'\n'"$failure" || return 1
      retries=0
      prompt="The gate failed:"$'\n'"$failure"$'\n'"The human says: $answer"
    else
      prompt="The gate refused the branch. Fix it, commit, and report again."$'\n'"$failure"
    fi
  done

  branch=$(git -C "$wt" branch --show-current)
  until publish "$branch" "$title" "$body"; do
    ask "Publishing $branch failed; see the output above. Fix it, then reply anything to retry." || return 1
  done
}

if [[ "${1:-}" == --dry-run ]]; then
  list_issues | awk -F '\t' '{ printf "#%s %s  %s\n", $1, ($2 == 0 ? "ready  " : "blocked"), $3 }'
  exit 0
fi

mkdir -p "$logs"
trap summary EXIT
# A sleeping Mac would stall the run; caffeinate exits along with this script.
caffeinate -i -w $$ &

while true; do
  issue=$(next_issue)
  if [[ -z "$issue" ]]; then
    break
  fi
  echo "=== Issue #$issue ==="
  if work_on "$issue"; then
    merged+=" #$issue"
  else
    skipped+=" $issue"
  fi
done
