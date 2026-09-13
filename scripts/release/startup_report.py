#!/usr/bin/env python3
"""Trace analysis for the macOS startup measurements.

Split out of `measure-startup-macos.sh` so the arithmetic can be exercised on synthetic traces
without launching the app. That matters because the two ways this analysis can silently lie are
both arithmetic: reporting process uptime as if it were recall latency, and dropping the wrong run
when separating cold start from warm start.

Usage:
    startup_report.py recall <trace-log> <out-json> <expected-recalls>
    startup_report.py cold   <run-dir>   <out-json> <expected-runs> <binary>
"""

from __future__ import annotations

import json
import pathlib
import re
import statistics
import sys
import time

TRACE_PATTERN = re.compile(r'\[aura\]\[trace\] \{"event":"([^"]+)","t_ms":(\d+)\}')

RECALL_REQUEST = "recall-requested"
RECALL_SHOWN = "window-shown:translation-recall"
TRAY_READY = "tray-ready"

WARM_RECALL_P95_BUDGET_MS = 200
COLD_START_BUDGET_MS = 2000


def read_events(path: pathlib.Path) -> list[tuple[str, int]]:
    events: list[tuple[str, int]] = []
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            match = TRACE_PATTERN.search(line)
            if match:
                events.append((match.group(1), int(match.group(2))))
    return events


def first_timestamp_of(events: list[tuple[str, int]], name: str) -> int | None:
    for event, t_ms in events:
        if event == name:
            return t_ms
    return None


def percentile(ordered: list[int], fraction: float) -> int:
    index = min(len(ordered) - 1, max(0, round(fraction * (len(ordered) - 1))))
    return ordered[index]


def summarize(values: list[int]) -> dict:
    ordered = sorted(values)
    return {
        "min": min(ordered),
        "p50": percentile(ordered, 0.5),
        "p95": percentile(ordered, 0.95),
        "max": max(ordered),
        "mean": round(statistics.fmean(ordered), 1),
    }


def recall_report(log_path: pathlib.Path, out_path: pathlib.Path, expected: int) -> dict:
    """Pairs each recall request with the show that follows it.

    `t_ms` is milliseconds since process entry, so a raw value is process uptime: waiting ten
    seconds before pressing the hotkey would report roughly 10000 ms even if the window appeared
    instantly. Only the difference between the paired markers is a latency.
    """
    events = read_events(log_path)

    deltas: list[int] = []
    pending_request: int | None = None
    superseded = 0
    for name, t_ms in events:
        if name == RECALL_REQUEST:
            if pending_request is not None:
                # A second request arrived before the first produced a show. Counting these
                # separately keeps "the window never appeared" distinguishable from "the user
                # pressed again before it did".
                superseded += 1
            pending_request = t_ms
        elif name == RECALL_SHOWN and pending_request is not None:
            deltas.append(t_ms - pending_request)
            pending_request = None

    # A request that is still pending at the end never produced a show.
    unanswered = 1 if pending_request is not None else 0
    sample_count_ok = len(deltas) == expected
    summary = summarize(deltas) if deltas else None

    return {
        "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "expectedRecalls": expected,
        "observedRecalls": len(deltas),
        "sampleCountMatches": sample_count_ok,
        "requestsWithoutShow": unanswered,
        "supersededRequests": superseded,
        "warmRecallMs": summary,
        "thresholds": {"warmRecallP95Ms": WARM_RECALL_P95_BUDGET_MS},
        # A short sample count must fail even when the samples that did arrive were fast: one
        # capture out of ten expected is not evidence that recall meets the budget.
        "passed": bool(
            summary
            and sample_count_ok
            and summary["p95"] <= WARM_RECALL_P95_BUDGET_MS
        ),
        "note": (
            "warmRecallMs is window-shown minus recall-requested, so it measures local recall "
            "latency and includes no provider time. It is not process uptime. A sample count "
            "below expectedRecalls, or a request that produced no show, fails the run."
        ),
    }


def cold_report(
    run_dir: pathlib.Path, out_path: pathlib.Path, expected: int, binary: str
) -> dict:
    """Reports cold start as the first run and warm start as every run after it.

    The runs are kept in launch order. Sorting first and dropping the smallest value would discard
    whichever run happened to be fastest rather than the one that was actually cold.
    """
    def run_number(path: pathlib.Path) -> int:
        return int(path.stem.split("-")[1])

    per_run = []
    for path in sorted(run_dir.glob("run-*.log"), key=run_number):
        per_run.append(
            {
                "run": path.stem,
                "trayReadyMs": first_timestamp_of(read_events(path), TRAY_READY),
            }
        )

    cold = per_run[0]["trayReadyMs"] if per_run else None
    warm_values = [r["trayReadyMs"] for r in per_run[1:] if r["trayReadyMs"] is not None]
    missing = [r["run"] for r in per_run if r["trayReadyMs"] is None]
    observed = len(per_run) - len(missing)

    return {
        "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "binary": binary,
        "expectedRuns": expected,
        "observedRuns": observed,
        "sampleCountMatches": observed == expected,
        "runsMissingTrayReady": missing,
        "perRun": per_run,
        "coldStartMs": cold,
        "warmStartMs": (
            {"samples": len(warm_values), **summarize(warm_values)} if warm_values else None
        ),
        "thresholds": {"coldStartMs": COLD_START_BUDGET_MS},
        "passed": {
            # A run that never reported readiness is a failed sample, not a fast one.
            "sampleCount": observed == expected,
            "coldStart": cold is not None and cold <= COLD_START_BUDGET_MS,
        },
        "note": (
            "tray-ready measures process entry to an operable menu bar, which is what the "
            "cold-start threshold describes. Warm start is every run after the first, in launch "
            "order. Warm recall is measured separately with --recall-only."
        ),
    }


def main(argv: list[str]) -> int:
    if len(argv) < 5:
        print(__doc__, file=sys.stderr)
        return 2

    mode = argv[1]
    if mode == "recall":
        payload = recall_report(pathlib.Path(argv[2]), pathlib.Path(argv[3]), int(argv[4]))
    elif mode == "cold":
        if len(argv) < 6:
            print("cold mode needs <run-dir> <out-json> <expected-runs> <binary>", file=sys.stderr)
            return 2
        payload = cold_report(
            pathlib.Path(argv[2]), pathlib.Path(argv[3]), int(argv[4]), argv[5]
        )
    else:
        print(f"unknown mode: {mode}", file=sys.stderr)
        return 2

    with open(argv[3], "w", encoding="utf-8") as handle:
        json.dump(payload, handle, ensure_ascii=False, indent=2)
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
