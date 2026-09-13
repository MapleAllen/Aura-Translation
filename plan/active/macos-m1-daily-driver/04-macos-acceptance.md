# macOS Acceptance

Owner: implementation owner plus macOS target host
Dependencies: stages 1-3 landed; a real macOS desktop is available

## Allowed Files

- `scripts/release/measure-resources-macos.sh` (new)
- `scripts/release/measure-startup-macos.sh` (new)
- `scripts/release/mock-translate-server.py` (new)
- `scripts/release/accept-interaction-macos.sh` (new)
- `docs/macOS-Adaptation-Checklist.md`
- this plan folder

## Do Not Modify

- `scripts/release/*.ps1` (Windows acceptance scripts)
- `.github/workflows/*` unless a check genuinely needs wiring

## Rationale

The proposal's threshold table mixes measurable and unmeasurable rows. "Background CPU below 0.5%"
is not discriminating: a `changeCount()` call every 275ms is far below that budget even in the
current, unfixed implementation, so the number can pass while the defect remains. The measurable
form of the same requirement is a behavioural assertion: with automatic translation off, no
`NSPasteboard` frames appear in a sampled call stack. The scripts below produce that evidence.

## Implementation Tasks

1. `measure-resources-macos.sh`
   - sample CPU and memory for the main process and attributable WebKit processes
   - capture `sample <pid> <seconds>` output while automatic translation is off
   - assert and report the absence of `NSPasteboard` frames
   - write `artifacts/macos-trial/resources.json`
2. `mock-translate-server.py`
   - minimal OpenAI-compatible streaming endpoint that records the number of requests
3. `accept-interaction-macos.sh`
   - drive same-text hotkey, new-text hotkey, and Esc collapse against the mock server
   - assert the request count is unchanged for same-text recall/collapse and increases by one for
     new text
   - write `artifacts/macos-trial/interaction.json`
4. `measure-startup-macos.sh`
   - run with `AURA_TRACE=1`, parse the trace records, and report cold-start and warm-recall p95
   - write `artifacts/macos-trial/startup.json`
5. Record target-host evidence in `docs/macOS-Adaptation-Checklist.md` and `05-verification.md`,
   including device, macOS version, and text length, and keeping network time separate from local
   time.

## Measurement Definitions

| Dimension | How it is measured | Gate |
|---|---|---|
| First use | Fresh config, working credential, wall-clock to first translation | Target 3 minutes; record actual |
| Toggle | Mock request counter across visible, hidden, pinned, in-flight, and config-changed states | Same-text recall/collapse issues 0 requests |
| Warm recall | Hotkey event to interactive window, 30 samples | p95 target 200ms, local only |
| Collapse | Hotkey event to hidden, 30 samples | p95 target 100ms, local only |
| Cold start | Process launch to operable menu bar, 10 runs | Target 2s; record cold vs warm cache |
| Background | Automatic translation off, idle 10 minutes | Zero `NSPasteboard` frames; CPU reported, not gated |
| Memory | App plus attributable WebKit processes | Report baseline, post-translation, post-settings |
| Failure handling | Offline, 401, 429, first-byte timeout, stream stall | Each has a readable message and a retry entry |
| Desktop fit | Fullscreen app, two displays at different scaling, display unplug, dark mode | No lost or off-screen window |
| Daily use | 5 consecutive working days | Trial notes; not a CI gate |

## Edge Cases

- Sampling must exclude the dev server; only a release build counts.
- A local Ollama model is reported separately and never counted against the app memory budget.
- Localhost mock runs need proxy variables cleared, matching the existing Rust test note.
- If a threshold fails, record the measured value and the shortfall rather than rounding up.

## Known Limitations

- The five-day trial and the three unread first-time users are self-reported evidence. They are
  recorded but never treated as automated gates.
- Intel Macs and the minimum supported macOS version stay unverified until a host is available.

## Completion Evidence

Scripts added under `scripts/release/` (POSIX shell, verified against the bash 3.2 that ships with
macOS; the PowerShell helpers are untouched):

- `mock-translate-server.py`: OpenAI-compatible stub that records every request and streams a canned
  translation with proper HTTP/1.1 chunked framing. Verified by hand for the streaming path, the
  non-streaming readiness path, and counter reset.
- `accept-interaction-macos.sh`: starts the stub, writes an isolated config so the run never touches
  the real Keychain or history, and prints the step table for the counter-based contract with the
  expected count after each step. Driving a global hotkey from a script would need accessibility
  permissions the app does not request, so the steps stay interactive and the script refuses to
  pretend otherwise.
- `measure-resources-macos.sh`: samples CPU and memory, and captures `sample` output to report
  whether any `NSPasteboard` frame appears. The gate is the frame count, not the CPU number, because
  the proposal's CPU threshold cannot distinguish a fixed build from a broken one.
- `measure-startup-macos.sh`: launches the release build repeatedly with `AURA_TRACE=1`, parses the
  traces, and reports cold start plus warm-recall percentiles against the 2 s and 200 ms thresholds.

Target-host rows in `05-verification.md` remain open. Nothing in this slice is claimed as verified
without a real-host run.
