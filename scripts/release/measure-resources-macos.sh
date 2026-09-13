#!/usr/bin/env bash
#
# macOS acceptance: background resource use, and proof that a paused monitor reads nothing.
#
# Why this script exists: the proposal's "CPU below 0.5%" threshold cannot distinguish a fixed
# implementation from a broken one, because reading NSPasteboard::changeCount() every 275 ms costs
# far less than that even while the loop is running. The requirement that actually matters is
# behavioural, so this script samples the process's call stack and reports whether any
# NSPasteboard frame appears while automatic translation is off.
#
# Usage:
#   scripts/release/measure-resources-macos.sh [--seconds 600] [--warmup 5] [--auto-mode off|on]
#
# Output: artifacts/macos-trial/resources.json

set -euo pipefail

SECONDS_TO_SAMPLE=600
WARMUP=5
AUTO_MODE="off"

while [ $# -gt 0 ]; do
  case "$1" in
    --seconds) SECONDS_TO_SAMPLE="$2"; shift 2 ;;
    --warmup) WARMUP="$2"; shift 2 ;;
    --auto-mode) AUTO_MODE="$2"; shift 2 ;;
    -h|--help) sed -n '3,14p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$REPO_ROOT/artifacts/macos-trial"
mkdir -p "$ARTIFACT_DIR"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "this script measures macOS processes and only runs on Darwin" >&2
  exit 1
fi

# The dev server would pollute both the CPU reading and the attributable process list.
if pgrep -f "vite" >/dev/null 2>&1; then
  echo "a vite dev server is running; stop it so measurements reflect a release build" >&2
  exit 1
fi

main_pid() {
  pgrep -x "aura-translation" | head -1
}

PID="$(main_pid || true)"
if [ -z "$PID" ]; then
  echo "Aura is not running. Launch the release .app first, then re-run this script." >&2
  exit 1
fi

echo "sampling pid $PID for ${WARMUP}s warmup + ${SECONDS_TO_SAMPLE}s"
sleep "$WARMUP"

# Every process that belongs to the app bundle: the Rust process plus its WebKit helpers, which a
# Tauri build cannot be assumed to be free of. Built with plain shell because macOS ships bash 3.2,
# which has no `mapfile`.
CHILD_PIDS="$(pgrep -P "$PID" 2>/dev/null || true)"
RELATED="$(printf '%s\n%s\n' "$CHILD_PIDS" "$PID" | grep -v '^$' | sort -u | tr '\n' ',' | sed 's/,$//')"

SAMPLE_PS="$ARTIFACT_DIR/resources-ps.txt"
: > "$SAMPLE_PS"
INTERVAL=5
ITERATIONS=$(( SECONDS_TO_SAMPLE / INTERVAL ))
[ "$ITERATIONS" -lt 1 ] && ITERATIONS=1

echo "collecting $ITERATIONS ps samples..."
for _ in $(seq 1 "$ITERATIONS"); do
  # -o %cpu is a lifetime average on macOS, so `top` is used for an instantaneous reading.
  top -l 1 -pid "$PID" -stats pid,cpu,mem,rsize 2>/dev/null | tail -1 >> "$SAMPLE_PS" || true
  sleep "$INTERVAL"
done

# Stack sampling is what turns "no clipboard work while paused" into evidence.
SAMPLE_OUT="$ARTIFACT_DIR/resources-sample.txt"
echo "capturing a stack sample (15s)..."
sample "$PID" 15 -file "$SAMPLE_OUT" >/dev/null 2>&1 || true

CLIPBOARD_FRAMES=0
if [ -f "$SAMPLE_OUT" ]; then
  CLIPBOARD_FRAMES="$(grep -c "NSPasteboard" "$SAMPLE_OUT" 2>/dev/null || echo 0)"
fi

python3 - "$ARTIFACT_DIR/resources.json" "$SAMPLE_PS" "$PID" "$RELATED" "$SECONDS_TO_SAMPLE" "$WARMUP" "$AUTO_MODE" "$CLIPBOARD_FRAMES" "$SAMPLE_OUT" <<'PY'
import json, re, statistics, sys, time

(
    out_path,
    ps_path,
    main_pid,
    related,
    seconds,
    warmup,
    auto_mode,
    clipboard_frames,
    sample_out,
) = sys.argv[1:]

cpu_samples, mem_samples = [], []
with open(ps_path, encoding="utf-8", errors="replace") as handle:
    for line in handle:
        parts = line.split()
        if len(parts) < 3:
            continue
        try:
            cpu_samples.append(float(parts[1]))
        except ValueError:
            continue
        mem = parts[2]
        match = re.match(r"^([\d.]+)([MGK])?$", mem)
        if match:
            value = float(match.group(1))
            unit = match.group(2)
            if unit == "G":
                value *= 1024
            elif unit == "K":
                value /= 1024
            mem_samples.append(value)

def summarize(values):
    if not values:
        return None
    return {
        "min": round(min(values), 3),
        "max": round(max(values), 3),
        "mean": round(statistics.fmean(values), 3),
    }

clipboard_frames = int(clipboard_frames)
paused = auto_mode == "off"
# The gate is behavioural, not a CPU number: a paused monitor must show no pasteboard frames.
passed = clipboard_frames == 0 if paused else True

payload = {
    "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%S"),
    "mainPid": int(main_pid),
    "relatedPids": related,
    "autoMode": auto_mode,
    "warmupSeconds": int(warmup),
    "sampleSeconds": int(seconds),
    "cpuPercent": summarize(cpu_samples),
    "memoryMB": summarize(mem_samples),
    "sampleFile": sample_out,
    "clipboardFrames": clipboard_frames,
    "passed": passed,
    "gate": (
        "With automatic translation off, no NSPasteboard frame may appear in the stack sample."
        if paused
        else "Automatic translation on: informational only, polling is expected."
    ),
}
with open(out_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, ensure_ascii=False, indent=2)

print(json.dumps(payload, ensure_ascii=False, indent=2))
PY

echo
echo "evidence written to $ARTIFACT_DIR/resources.json"
if [ "$AUTO_MODE" = "off" ]; then
  echo "Gate: clipboardFrames must be 0. Also confirm usable macOS memory in Activity Monitor,"
  echo "since WebKit helpers may be reported separately from the main process."
fi
