#!/usr/bin/env bash
# Keeps STATUS.md (the project state) in step with the work, across sessions and restarts.
#   session: at session start, list work files changed after STATUS.md was last edited.
#   stop:    at the end of a turn, block once if this session changed work files after STATUS.md.
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
state="$root/STATUS.md"
input="$(cat)"
sid="$(printf '%s' "$input" | sed -n 's/.*"session_id" *: *"\([^"]*\)".*/\1/p')"
marker="${XDG_RUNTIME_DIR:-/tmp}/nuclear-context-${sid:-none}"

# Work files (code, dialects, specs, tools) newer than $1. Generated and vendored folders are skipped.
changed() {
  cd "$root" && find crates dialects languages spec tools install.sh dragonfly-mobile dragonfly-backend dragonfly-web \
    dragonfly/dragonfly/src nuclear-vscode/src nuclear-jetbrains/src nuclear-zed/src \
    \( -name target -o -name node_modules -o -name build -o -name .dart_tool -o -name .venv -o -name dist-test -o -name .git \) -prune \
    -o -type f -newer "$1" -print 2>/dev/null
}

case "${1:-}" in
session)
  [ -e "$marker" ] || touch "$marker"
  files="$(changed "$state")"
  if [ -n "$files" ]; then
    echo "STATUS.md is older than $(printf '%s\n' "$files" | wc -l) work file(s): a previous session may have finished"
    echo "work without recording it. Before reporting status or planning, look at these files and record what"
    echo "they did in STATUS.md (Log and lists):"
    printf '%s\n' "$files" | head -40
  fi
  ;;
stop)
  printf '%s' "$input" | grep -q '"stop_hook_active" *: *true' && exit 0
  [ -e "$marker" ] || exit 0
  ref="$marker"; [ "$state" -nt "$marker" ] && ref="$state"
  files="$(changed "$ref")"
  if [ -n "$files" ]; then
    {
      echo "This session changed work files after the last edit of STATUS.md. Before finishing, update STATUS.md:"
      echo "one Log line and the lists (done items removed, Half done, Untested), plus the context and user"
      echo "docs of the area (docs/context/workflow.md). Files:"
      printf '%s\n' "$files" | head -20
    } >&2
    exit 2
  fi
  ;;
esac
exit 0
