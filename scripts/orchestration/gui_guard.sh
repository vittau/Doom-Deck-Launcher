#!/bin/sh
# Orchestrator guard (docs/orchestration.md): workers must never pop up windows or capture the
# owner's screen. Every 5 s, kill GUI runs of the dev `ddl` binary (headless flags are allowed)
# and any `screencapture`, printing one line per kill so a Claude Code Monitor reports it.
while true; do
  ps -axo pid=,command= | while read -r pid cmd; do
    case "$cmd" in
      *target/debug/ddl*|*target/release/ddl*)
        case "$cmd" in
          *--version*|*--help*|*--screenshot*|*cargo*|*rustc*|*ddl-*|*/deps/*) ;;
          *) kill "$pid" 2>/dev/null && echo "$(date +%H:%M:%S) killed GUI ddl pid=$pid: $cmd" ;;
        esac ;;
      screencapture*|*/screencapture*)
        kill "$pid" 2>/dev/null && echo "$(date +%H:%M:%S) killed screencapture pid=$pid: $cmd" ;;
    esac
  done
  sleep 5
done
