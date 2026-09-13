#!/usr/bin/env bash
#
# macOS acceptance: cold-start and warm-recall latencies from the app's own trace records.
#
# AURA_TRACE=1 makes the app emit `[aura][trace] {"event":"...","t_ms":N}` lines to stderr, where
# N is milliseconds since process entry. Those are moments inside the process that no external
# tool can observe.
#
# Important: `t_ms` is process uptime, so it is only a latency when the interval starts at the
# process itself (cold start). Recall latency is the difference between a request marker and the
# show that follows it — the raw value would grow the longer the user waits before pressing.
#
# Usage:
#   scripts/release/measure-startup-macos.sh [--runs 10] [--binary <path>]
#   scripts/release/measure-startup-macos.sh --recall-only     # measure warm recall only
#
# Warm recall needs a hotkey press, which a shell script cannot synthesise without accessibility
# permissions the app does not request. It is therefore a separate, clearly-labelled pass.
#
# Output: artifacts/macos-trial/startup.json and artifacts/macos-trial/startup-recall.json

set -euo pipefail

RUNS=10
BINARY=""
RECALL_ONLY=0

while [ $# -gt 0 ]; do
  case "$1" in
    --runs) RUNS="$2"; shift 2 ;;
    --binary) BINARY="$2"; shift 2 ;;
    --recall-only) RECALL_ONLY=1; shift ;;
    -h|--help) sed -n '3,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$REPO_ROOT/artifacts/macos-trial"
mkdir -p "$ARTIFACT_DIR"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "this script measures macOS launch behavior and only runs on Darwin" >&2
  exit 1
fi

if [ -z "$BINARY" ]; then
  BINARY="$REPO_ROOT/src-tauri/target/release/bundle/macos/Aura Translation.app/Contents/MacOS/aura-translation"
fi

if [ ! -x "$BINARY" ]; then
  echo "release binary not found at: $BINARY" >&2
  echo "build it first with: npm run tauri build" >&2
  exit 1
fi

# ---------------------------------------------------------------------------------------------
# Recall pass
# ---------------------------------------------------------------------------------------------
if [ "$RECALL_ONLY" -eq 1 ]; then
  RECALL_LOG="$ARTIFACT_DIR/startup-recall.log"
  : > "$RECALL_LOG"

  cat <<EOF
== warm recall pass ==

Aura starts with tracing enabled. Leave it running and perform $RUNS recall actions: press the
configured hotkey (or use 打开翻译 in the menu bar) so the translation window appears each time.
Press the hotkey again in between so every attempt is a real re-show.

Then press Enter here to stop and read the trace.
EOF

  AURA_TRACE=1 "$BINARY" >> "$RECALL_LOG" 2>&1 &
  APP_PID=$!

  printf 'Press Enter when the %s recall actions are done... ' "$RUNS"
  read -r _

  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true

  python3 "$REPO_ROOT/scripts/release/startup_report.py" recall \
    "$RECALL_LOG" "$ARTIFACT_DIR/startup-recall.json" "$RUNS"

  echo
  echo "evidence written to $ARTIFACT_DIR/startup-recall.json"
  exit 0
fi

# ---------------------------------------------------------------------------------------------
# Cold-start pass
# ---------------------------------------------------------------------------------------------
echo "launching the release build $RUNS times with AURA_TRACE=1"

RUN_DIR="$ARTIFACT_DIR/startup-runs"
rm -rf "$RUN_DIR"
mkdir -p "$RUN_DIR"

for run in $(seq 1 "$RUNS"); do
  # A per-run log is required. With one shared, appended log, every run after the first found the
  # previous run's `tray-ready` immediately and killed the new process before it was ever ready,
  # so every sample after the first was meaningless.
  RUN_LOG="$RUN_DIR/run-$run.log"
  : > "$RUN_LOG"

  # The first run differs from later ones: the OS file cache is warm afterwards.
  AURA_TRACE=1 "$BINARY" >> "$RUN_LOG" 2>&1 &
  APP_PID=$!

  ready=0
  for _ in $(seq 1 300); do
    if grep -q '"event":"tray-ready"' "$RUN_LOG" 2>/dev/null; then
      ready=1
      break
    fi
    if ! kill -0 "$APP_PID" 2>/dev/null; then
      break
    fi
    sleep 0.05
  done

  if [ "$ready" -eq 0 ]; then
    echo "run $run never reported tray-ready; see $RUN_LOG" >&2
  fi

  # Give the event loop a moment to flush before terminating.
  sleep 0.3
  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  sleep 0.7
done

python3 "$REPO_ROOT/scripts/release/startup_report.py" cold \
  "$RUN_DIR" "$ARTIFACT_DIR/startup.json" "$RUNS" "$BINARY"

echo
echo "evidence written to $ARTIFACT_DIR/startup.json"
echo "per-run logs kept in $RUN_DIR"
echo "For warm recall, re-run with --recall-only and drive the hotkey by hand."
echo "Report the device, macOS version, and text length alongside the numbers."
