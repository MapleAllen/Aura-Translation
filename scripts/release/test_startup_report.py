#!/usr/bin/env python3
"""Regression tests for the macOS startup trace analysis.

Run with:
    python3 -m unittest discover -s scripts/release -p 'test_*.py'

The analysis in `startup_report.py` has silently reported wrong numbers twice: raw process uptime
presented as recall latency, and the warm sample set dropping the fastest run instead of the first.
Both are arithmetic, so they are pinned here instead of by launching the app.
"""

from __future__ import annotations

import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import startup_report  # noqa: E402


def trace(event: str, t_ms: int) -> str:
    return f'[aura][trace] {{"event":"{event}","t_ms":{t_ms}}}\n'


class RecallReportTests(unittest.TestCase):
    def write_log(self, directory: pathlib.Path, lines: list[str]) -> pathlib.Path:
        path = directory / "recall.log"
        path.write_text("".join(lines), encoding="utf-8")
        return path

    def test_recall_latency_is_a_delta_not_process_uptime(self) -> None:
        # The hotkey is pressed ten seconds after launch and the window appears 18ms later. The
        # previous implementation collected the raw `t_ms`, so this reported roughly 10000ms.
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            lines: list[str] = []
            for index in range(3):
                base = 10000 + index * 20000
                lines += [
                    trace("setup-start", 100),
                    trace("tray-ready", 150),
                    trace("recall-requested", base),
                    trace("window-shown:translation-recall", base + 18),
                ]
            log = self.write_log(directory, lines)

            report = startup_report.recall_report(log, directory / "out.json", 3)

            self.assertEqual(report["observedRecalls"], 3)
            self.assertEqual(report["warmRecallMs"]["p95"], 18)
            self.assertLess(report["warmRecallMs"]["max"], 100)
            self.assertTrue(report["passed"])

    def test_an_incomplete_sample_set_fails_even_when_fast(self) -> None:
        # One captured recall out of ten expected, and it was quick. A short sample count must not
        # be reported as a pass.
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            log = self.write_log(
                directory,
                [
                    trace("recall-requested", 5000),
                    trace("window-shown:translation-recall", 5010),
                ],
            )

            report = startup_report.recall_report(log, directory / "out.json", 10)

            self.assertEqual(report["observedRecalls"], 1)
            self.assertFalse(report["sampleCountMatches"])
            self.assertFalse(report["passed"])

    def test_a_recall_that_exceeds_the_budget_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            lines: list[str] = []
            for _ in range(4):
                lines += [
                    trace("recall-requested", 2000),
                    trace("window-shown:translation-recall", 2350),
                ]
            log = self.write_log(directory, lines)

            report = startup_report.recall_report(log, directory / "out.json", 4)

            self.assertEqual(report["warmRecallMs"]["p95"], 350)
            self.assertFalse(report["passed"])

    def test_a_superseded_request_is_reported(self) -> None:
        # Two requests, one show. The report has to distinguish "the user pressed again before the
        # window appeared" from "the window never appeared at all".
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            log = self.write_log(
                directory,
                [
                    trace("recall-requested", 5000),
                    trace("recall-requested", 6000),
                    trace("window-shown:translation-recall", 6012),
                ],
            )

            report = startup_report.recall_report(log, directory / "out.json", 2)

            self.assertEqual(report["observedRecalls"], 1)
            # The first request was superseded by the second; the second produced the single show.
            self.assertEqual(report["supersededRequests"], 1)
            self.assertEqual(report["requestsWithoutShow"], 0)
            self.assertFalse(report["passed"])

    def test_a_request_with_no_show_is_reported(self) -> None:
        # The window never appeared, which is what a failed show looks like in the trace now that
        # the completion marker is only emitted on success.
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            log = self.write_log(directory, [trace("recall-requested", 5000)])

            report = startup_report.recall_report(log, directory / "out.json", 1)

            self.assertEqual(report["observedRecalls"], 0)
            self.assertEqual(report["requestsWithoutShow"], 1)
            self.assertFalse(report["passed"])

    def test_no_recalls_at_all_is_not_a_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            log = self.write_log(directory, [trace("tray-ready", 150)])

            report = startup_report.recall_report(log, directory / "out.json", 5)

            self.assertIsNone(report["warmRecallMs"])
            self.assertFalse(report["passed"])


class ColdReportTests(unittest.TestCase):
    def write_runs(self, directory: pathlib.Path, times: list[int]) -> pathlib.Path:
        run_dir = directory / "runs"
        run_dir.mkdir()
        for index, t_ms in enumerate(times, start=1):
            (run_dir / f"run-{index}.log").write_text(
                trace("setup-start", max(t_ms - 20, 0)) + trace("tray-ready", t_ms),
                encoding="utf-8",
            )
        return run_dir

    def test_cold_is_the_first_run_and_warm_is_every_run_after_it(self) -> None:
        # Sorting first and dropping the smallest would discard whichever run happened to be
        # fastest. With 131/106/127 the warm set must be 106 and 127, not 127 and 131.
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            run_dir = self.write_runs(directory, [131, 106, 127])

            report = startup_report.cold_report(run_dir, directory / "out.json", 3, "/bin/true")

            self.assertEqual(report["coldStartMs"], 131)
            self.assertEqual(report["warmStartMs"]["samples"], 2)
            self.assertEqual(report["warmStartMs"]["min"], 106)
            self.assertEqual(report["warmStartMs"]["max"], 127)
            self.assertTrue(report["passed"]["sampleCount"])
            self.assertTrue(report["passed"]["coldStart"])

    def test_a_missing_run_fails_the_sample_count(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            run_dir = self.write_runs(directory, [131])
            # A second run whose log never recorded readiness.
            (run_dir / "run-2.log").write_text(trace("setup-start", 90), encoding="utf-8")

            report = startup_report.cold_report(run_dir, directory / "out.json", 2, "/bin/true")

            self.assertEqual(report["observedRuns"], 1)
            self.assertFalse(report["sampleCountMatches"])
            self.assertFalse(report["passed"]["sampleCount"])
            self.assertEqual(report["runsMissingTrayReady"], ["run-2"])

    def test_a_cold_start_over_budget_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            run_dir = self.write_runs(directory, [2600, 140, 150])

            report = startup_report.cold_report(run_dir, directory / "out.json", 3, "/bin/true")

            self.assertEqual(report["coldStartMs"], 2600)
            self.assertFalse(report["passed"]["coldStart"])

    def test_malformed_lines_are_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            run_dir = self.write_runs(directory, [120])
            with (run_dir / "run-1.log").open("a", encoding="utf-8") as handle:
                handle.write("not a trace line\n")
                handle.write('[aura][trace] {"event":"tray-ready","t_ms":not-a-number}\n')

            report = startup_report.cold_report(run_dir, directory / "out.json", 1, "/bin/true")

            self.assertEqual(report["coldStartMs"], 120)

    def test_the_report_is_written_to_disk(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            run_dir = self.write_runs(directory, [120, 110])
            out = directory / "nested" / "out.json"
            out.parent.mkdir()

            exit_code = startup_report.main(
                ["startup_report.py", "cold", str(run_dir), str(out), "2", "/bin/true"]
            )

            self.assertEqual(exit_code, 0)
            payload = json.loads(out.read_text(encoding="utf-8"))
            self.assertEqual(payload["coldStartMs"], 120)


if __name__ == "__main__":
    unittest.main()
