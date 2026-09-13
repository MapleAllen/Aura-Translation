#!/usr/bin/env bash
#
# macOS acceptance: the interaction contract, verified by counting real HTTP requests.
#
# The milestone's P0 rule is that recalling or collapsing the same text must not issue a new
# request. Unit tests assert the decision function; this script asserts the observable outcome
# against a stub provider, which is the evidence the proposal actually asks for.
#
# Usage:
#   scripts/release/accept-interaction-macos.sh [--port 8787] [--app "/Applications/Aura Translation.app"]
#
# What it does:
#   1. starts scripts/release/mock-translate-server.py
#   2. points an isolated Aura config at it (no Keychain, no real provider)
#   3. reports the request count so a human can drive the hotkey and read the counters
#
# Steps 3 and the assertions are interactive on purpose: driving a global hotkey and a menu-bar
# toggle from a shell script would require accessibility permissions the app does not ask for.
# The script prints the exact commands and expectations instead of pretending to automate them.

set -euo pipefail

PORT=8787
APP="${AURA_APP:-}"
WORKDIR=""

usage() {
  sed -n '3,20p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --port) PORT="$2"; shift 2 ;;
    --app) APP="$2"; shift 2 ;;
    -h|--help) usage 0 ;;
    *) echo "unknown argument: $1" >&2; usage 1 ;;
  esac
done

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$REPO_ROOT/artifacts/macos-trial"
mkdir -p "$ARTIFACT_DIR"

WORKDIR="$(mktemp -d)"
ISOLATED_HOME="$WORKDIR/home"
# `dirs::config_dir()` resolves to ~/Library/Application Support on macOS, not ~/.config. Writing
# to the wrong directory silently leaves the app on its defaults, which would quietly invalidate
# every step below. Verified against the release bundle: with `setup_completed` true here, no
# onboarding window opens and the tray reports ready.
CONFIG_DIR="$ISOLATED_HOME/Library/Application Support/aura-translation"
mkdir -p "$CONFIG_DIR"

SERVER_PID=""
cleanup() {
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

count_requests() {
  curl -fsS "http://127.0.0.1:$PORT/__count" | python3 -c 'import json,sys; print(json.load(sys.stdin)["count"])'
}

reset_requests() {
  curl -fsS -X POST "http://127.0.0.1:$PORT/__reset" >/dev/null
}

echo "== starting stub provider on port $PORT =="
python3 "$REPO_ROOT/scripts/release/mock-translate-server.py" --port "$PORT" &
SERVER_PID=$!

for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$PORT/__count" >/dev/null 2>&1; then break; fi
  sleep 0.1
done
if ! curl -fsS "http://127.0.0.1:$PORT/__count" >/dev/null 2>&1; then
  echo "stub provider did not start" >&2
  exit 1
fi

# An isolated config keeps the acceptance run away from the real Keychain, history, and config.
cat > "$CONFIG_DIR/config.json" <<JSON
{
  "api_key": "sk-acceptance-stub",
  "api_key_storage": "plaintext_fallback",
  "active_profile_id": "default",
  "model": "stub-model",
  "source_lang": "auto",
  "target_lang": "Chinese",
  "provider": "deepseek",
  "api_base_url": "http://127.0.0.1:$PORT",
  "available_models": ["stub-model"],
  "setup_completed": true,
  "notifications_enabled": false
}
JSON

cat <<EOF

== isolated environment ready ==
  HOME=$ISOLATED_HOME
  config=$CONFIG_DIR/config.json
  stub=http://127.0.0.1:$PORT

Launch Aura against this environment (leave this script running):

  HOME="$ISOLATED_HOME" CFFIXED_USER_HOME="$ISOLATED_HOME" AURA_TRACE=1 \
    "${APP:-$REPO_ROOT/src-tauri/target/release/bundle/macos/Aura Translation.app/Contents/MacOS/aura-translation}"

Then drive the contract and read the counter after each step:

  current count:   curl -s http://127.0.0.1:$PORT/__count
  reset counter:   curl -s -X POST http://127.0.0.1:$PORT/__reset

| Step | Action | Expected |
|---|---|---|
| 1 | Copy "Hello world", press the hotkey | count 1 |
| 2 | Press the hotkey again, window visible | count still 1 (collapsed, no request) |
| 3 | Press the hotkey again, window hidden | count still 1 (recalled, no request) |
| 4 | Press Esc | window hidden, count still 1 |
| 5 | Copy "Goodbye world", press the hotkey | count 2 |
| 6 | Toggle 自动翻译 off in the menu bar | count stays 2 while copying text |
| 7 | Toggle 自动翻译 on, copy new text | count increases by exactly 1 |

Record the observed numbers in docs/macOS-Adaptation-Checklist.md. Any step whose counter
increases when it should not is a P0 regression.

EOF

printf 'Press Enter to reset the counter and continue, or Ctrl+C to stop... '
read -r _
reset_requests
echo "counter reset to $(count_requests)"

printf 'Press Enter when the interactive run is finished to write evidence... '
read -r _

FINAL_COUNT="$(count_requests)"
python3 - "$ARTIFACT_DIR/interaction.json" "$FINAL_COUNT" "$PORT" <<'PY'
import json, sys, time
path, final_count, port = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
payload = {
    "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%S"),
    "stubPort": port,
    "observedRequestCount": final_count,
    "note": (
        "Interactive run. Compare observedRequestCount against the step table printed by "
        "accept-interaction-macos.sh; the counter must not move on recall or collapse."
    ),
}
with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, ensure_ascii=False, indent=2)
print(f"wrote {path}")
PY
