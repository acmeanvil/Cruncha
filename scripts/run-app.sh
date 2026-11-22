#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Launch the Cruncha desktop app (Tauri dev mode) for UI and functional review.
#
#   scripts/run-app.sh                # install front-end deps if needed, then run the app
#   scripts/run-app.sh --check        # first run svelte-check and the backend tests
#   scripts/run-app.sh --reset-prefs  # start with default preferences (old file kept as .bak)
#   scripts/run-app.sh --open <doc>   # start with that .cruncha document open
#
# In the app: "New sample…" creates a small beam-check document with history and a named
# version; "Open…" opens any existing .cruncha directory. The first build compiles DataFusion
# and Tauri and can take 10+ minutes; later launches take seconds.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="$root/app"
port=5173
prefs_dir="$HOME/Library/Application Support/io.github.acmeanvil.cruncha"

check=false
reset_prefs=false
open_doc=""
while [ $# -gt 0 ]; do
  arg="$1"; shift
  case "$arg" in
    --open) open_doc="${1:?--open needs a path}"; shift ;;
    --check) check=true ;;
    --reset-prefs) reset_prefs=true ;;
    -h|--help) sed -n '4,13p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $arg (try --help)" >&2; exit 2 ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "error: '$1' not found on PATH. $2" >&2; exit 1; }
}
need cargo "Install Rust (this machine uses Homebrew's rust)."
need node  "Install Node.js 20 or newer."
need npm   "Install Node.js 20 or newer."

if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "error: port $port is already in use (an old 'npm run tauri dev'?)." >&2
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

echo "==> Starting Cruncha (Vite on :$port + Rust backend). Close the window or press Ctrl-C to stop."
if [ -n "$open_doc" ]; then
  export CRUNCHA_OPEN="$(cd "$(dirname "$open_doc")" && pwd)/$(basename "$open_doc")"
fi
exec npm run tauri dev
