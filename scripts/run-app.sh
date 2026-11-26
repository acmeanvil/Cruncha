#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Launch the Cruncha desktop app (Tauri dev mode) for UI and functional review.
#
#   scripts/run-app.sh                # install front-end deps if needed, then run the app
#   scripts/run-app.sh --check        # first run svelte-check and the backend tests
#   scripts/run-app.sh --reset-prefs  # start with default preferences (old file kept as .bak)
#   scripts/run-app.sh --open <doc>   # start with that .cruncha document open
#   scripts/run-app.sh --clean        # only clean up after a crash (see below), then exit
#   scripts/run-app.sh --no-restart   # don't restart the app after a crash
#
# The app's debug log (ADR-0029) is ~/Library/Logs/io.github.acmeanvil.cruncha/cruncha.log;
# in the app, the bottom panel's Log tab (⌘⌥L). CRUNCHA_LOG=debug writes every record to the
# file, and CRUNCHA_SLOW_MS=0 logs every command's time.
#
# Every start first cleans up what a crashed or frozen run can leave behind: this checkout's
# dev cruncha-app processes (the app, its MCP stdio bridges and ACP agents), the Vite server on
# :5173 if it is this app's, orphaned Python workers, and a stale agents' connection file. If
# the app crashes, it is cleaned up and restarted (at most 3 times in 5 minutes). If it
# freezes, press Ctrl-C: that cleans up too; then run this again.
#
# In the app: "New sample…" creates a small beam-check document with history and a named
# version; "Open…" opens any existing .cruncha directory. The first build compiles DataFusion
# and Tauri and can take 10+ minutes; later launches take seconds.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="$root/app"
port=5173
prefs_dir="$HOME/Library/Application Support/io.github.acmeanvil.cruncha"
log_file="$HOME/Library/Logs/io.github.acmeanvil.cruncha/cruncha.log"

check=false
reset_prefs=false
clean_only=false
restart=true
open_doc=""
while [ $# -gt 0 ]; do
  arg="$1"; shift
  case "$arg" in
    --open) open_doc="${1:?--open needs a path}"; shift ;;
    --check) check=true ;;
    --reset-prefs) reset_prefs=true ;;
    --clean) clean_only=true ;;
    --no-restart) restart=false ;;
    -h|--help) sed -n '4,21p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $arg (try --help)" >&2; exit 2 ;;
  esac
done

# What a crashed or frozen run leaves behind. Only this checkout's processes are touched.
cleanup() {
  local cleaned=0 pids pid
  # The dev app and anything it started from its own binary (MCP bridges, ACP agents).
  pids="$(pgrep -f "$root/target/debug/cruncha-app" || true)"
  for pid in $pids; do kill "$pid" 2>/dev/null && cleaned=$((cleaned + 1)); done
  # Vite on the dev port, if it is this app's.
  for pid in $(lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true); do
    if [[ "$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p')" == "$app"* ]]; then
      kill "$pid" 2>/dev/null && cleaned=$((cleaned + 1))
    fi
  done
  # Python workers whose app is gone (adopted by launchd).
  for pid in $(pgrep -f "cruncha_worker" || true); do
    if [[ "$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')" == "1" ]]; then
      kill "$pid" 2>/dev/null && cleaned=$((cleaned + 1))
    fi
  done
  # The agents' connection file of a server that no longer runs.
  local conn="$prefs_dir/agents/server.json"
  if [[ -f "$conn" ]]; then
    pid="$(sed -n 's/.*"pid": *\([0-9]*\).*/\1/p' "$conn" | head -1)"
    if [[ -n "$pid" ]] && ! kill -0 "$pid" 2>/dev/null; then
      rm -f "$conn" && cleaned=$((cleaned + 1))
    fi
  fi
  if (( cleaned > 0 )); then sleep 0.3; echo "==> Cleaned up $cleaned leftover process(es) or file(s) from an earlier run"; fi
  return 0
}

if $clean_only; then
  cleanup
  echo "==> Clean"
  exit 0
fi

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "error: '$1' not found on PATH. $2" >&2; exit 1; }
}
need cargo "Install Rust (this machine uses Homebrew's rust)."
need node  "Install Node.js 20 or newer."
need npm   "Install Node.js 20 or newer."

cleanup
if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "error: port $port is in use by another program (not this app's Vite, so it was left alone)." >&2
  lsof -nP -iTCP:"$port" -sTCP:LISTEN >&2 || true
  exit 1
fi

cd "$app"
if [[ ! -d node_modules || package-lock.json -nt node_modules ]]; then
  echo "==> Installing front-end dependencies (npm ci)"
  npm ci
  touch node_modules
fi

if $check; then
  echo "==> svelte-check"
  npm run check
  echo "==> Backend tests (cruncha-app)"
  (cd "$root" && cargo test -p cruncha-app)
fi

if $reset_prefs && [[ -f "$prefs_dir/preferences.json" ]]; then
  mv "$prefs_dir/preferences.json" "$prefs_dir/preferences.json.bak"
  echo "==> Preferences reset (previous file: $prefs_dir/preferences.json.bak)"
fi

# Python cells (ADR-0005): the app sets up its environment on request; say whether it exists.
py_base="$HOME/Library/Application Support/io.github.acmeanvil.cruncha/python"
if compgen -G "$py_base/env-*/.cruncha-ready" >/dev/null; then
  echo "==> Python environment: ready"
else
  echo "==> Python environment: not set up (the app offers \"Set up Python\" for documents with Python cells;"
  echo "    or run: cargo run -p cruncha-kernel-py --example setup_env)"
fi

if [ -n "$open_doc" ]; then
  export CRUNCHA_OPEN="$(cd "$(dirname "$open_doc")" && pwd)/$(basename "$open_doc")"
fi

# Ctrl-C (a frozen app, say) stops this run and cleans up after it.
trap 'echo; echo "==> Stopping"; cleanup; exit 130' INT TERM

crashes=()
while true; do
  echo "==> Starting Cruncha (Vite on :$port + Rust backend). Close the window or press Ctrl-C to stop."
  status=0
  npm run tauri dev || status=$?
  # Closing the window ends the run normally.
  if (( status == 0 )) || ! $restart; then
    cleanup
    exit "$status"
  fi
  now=$(date +%s)
  recent=()
  for t in "${crashes[@]+"${crashes[@]}"}"; do (( now - t < 300 )) && recent+=("$t"); done
  crashes=("${recent[@]+"${recent[@]}"}" "$now")
  echo "==> Cruncha stopped unexpectedly (exit $status)."
  # The app's debug log (ADR-0029) records the restart beside its own last lines.
  if [[ -f "$log_file" ]]; then
    echo "=== run-app.sh: the app exited with status $status; cleaned up and restarting ===" >> "$log_file"
  fi
  cleanup
  if (( ${#crashes[@]} > 3 )); then
    echo "==> It crashed ${#crashes[@]} times in 5 minutes; not restarting. Check the output above." >&2
    exit "$status"
  fi
  echo "==> Restarting in 2 s (Ctrl-C to stop)"
  sleep 2
done
