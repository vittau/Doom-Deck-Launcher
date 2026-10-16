#!/bin/sh
# Orchestrator guard (docs/orchestration.md): workers must never pop up windows or capture the
# owner's screen. Every 5 s, kill GUI runs of the dev `ddl` binary (headless flags are allowed)
# and any `screencapture`, printing one line per kill so a Claude Code Monitor reports it.
while true; do
  ps -axo pid=,command= | while read -r pid cmd; do
    case "$cmd" in
      # Only the GUI entry points open a window: `ddl` with no arguments, or `--windowed`.
      # Subcommands (`ddl catalog lint ...`) and flags like --version/--help are headless.
      *target/debug/ddl|*target/release/ddl|*target/debug/ddl\ --windowed*|*target/release/ddl\ --windowed*)
        kill "$pid" 2>/dev/null && echo "$(date +%H:%M:%S) killed GUI ddl pid=$pid: $cmd" ;;
      screencapture*|*/screencapture*)
        kill "$pid" 2>/dev/null && echo "$(date +%H:%M:%S) killed screencapture pid=$pid: $cmd" ;;
    esac
  done
  sleep 5
done
