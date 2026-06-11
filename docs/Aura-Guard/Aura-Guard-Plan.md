# Aura Guard Plan

## Objective

Aura Guard will become a transparent, precise, and user-configurable credential screening layer. When all phases are complete, users will see a brief notification whenever a clipboard change is blocked, be able to define their own allow-list patterns, and benefit from an entropy-based scorer that reduces false positives on long non-credential strings. The keyword list will extend beyond English, and additional vendor token formats will be detected without requiring changes to the Rust source.

## Design Principles

- **Silent is the default**: Aura Guard must not interrupt the user's workflow with modal dialogs or persistent alerts; any notification must be brief and non-blocking.
- **False negatives are acceptable; false positives on real text are not**: it is better to miss an unusual token format than to block legitimate text the user wants to translate.
- **All heuristics are opt-out, not opt-in**: users should be able to disable individual checks via config rather than having to enable screening one rule at a time.
- **No network calls during screening**: the detection must be synchronous and offline; sending clipboard content to a remote classifier is not acceptable.
- **Guard state is visible in Settings**: users must always be able to see and toggle `aura_guard_enabled` from Settings.

## Phase 1: Block Reason Notification — NOT STARTED

Status: **Not Started**

Goals:

- Surface the block reason to the user as a brief, dismissible tray notification or inline bubble message.
- Give users visibility into why Aura mode did not trigger on a clipboard change.

Remaining features:

- Emit a `aura-guard-blocked` event from the Aura mode clipboard handler when `detect_sensitive_clipboard` returns `Some`.
- Include the `reason` string in the event payload.
- Handle `aura-guard-blocked` in `NotificationCenter.svelte` or the translation bubble to show a brief auto-dismissing toast.
- Add a user setting to suppress block notifications (default: show).

## Phase 2: Configurable Allow-List — NOT STARTED

Status: **Not Started**

Goals:

- Let users define regex patterns or literal prefixes that are always passed through without screening.
- Prevent Aura Guard from blocking content the user has explicitly marked as safe.

Remaining features:

- Add `aura_guard_allow_patterns: Vec<String>` to `AppConfig` (default: empty).
- Before running any heuristic in `detect_sensitive_clipboard`, check whether the text matches any allow-list pattern; if it does, return `None` immediately.
- Patterns are matched as literal substrings by default; prefix them with `regex:` to enable regex matching.
- Add an allow-list editor to the Aura Guard section of Settings.
- Validate regex patterns at save time and reject malformed patterns with a user-facing error.

## Phase 3: Entropy-Based Scorer — NOT STARTED

Status: **Not Started**

Goals:

- Replace the blob heuristic with a Shannon entropy scorer that better distinguishes high-entropy credential strings from low-entropy but long natural language text.
- Reduce false positives on long base64-encoded content that is not a credential.

Remaining features:

- Implement a `shannon_entropy(text: &str) -> f32` helper that computes per-character entropy over the ASCII printable range.
- Replace `looks_like_secret_blob` with a combined check: charset filter AND entropy ≥ a configurable threshold (default: 4.0 bits/char).
- Add `aura_guard_entropy_threshold: f32` to `AppConfig` with a default of `4.0`.
- Expose the threshold slider in Settings for advanced users.
- Retain the length ≥ 24 and no-whitespace preconditions from the existing heuristic.

## Phase 4: Extended Token Format Detection — NOT STARTED

Status: **Not Started**

Goals:

- Detect additional vendor-specific credential formats without requiring Rust source changes.
- Support a data-driven pattern registry that can be updated independently of the app binary.

Remaining features:

- Define a `TokenPattern` struct: `{ prefix: Option<String>, keywords: Vec<String>, description: String }`.
- Load a built-in list of token patterns at compile time (expanding the current hard-coded lists).
- Add detection for: AWS session tokens (`ASIA`, `AROA`), Azure SAS tokens (`?sv=`), GCP service account JSON (`"type": "service_account"`), npm tokens (`npm_`), PyPI tokens (`pypi-`).
- Allow users to add custom token patterns via `aura_guard_allow_patterns` (from Phase 2) or a separate `aura_guard_block_patterns` list.

## Implementation Rules

- Do not perform any I/O inside `detect_sensitive_clipboard`; the function must remain synchronous and side-effect free.
- Do not block empty or whitespace-only text; that check is an early return before any heuristic runs.
- Do not add heuristics that require parsing external formats (JSON, YAML, etc.); the guard must not fail if the clipboard contains malformed structured data.
- Do not surface block reasons as modal dialogs; use a transient, auto-dismissing notification only.

## Open Questions

- **What is the right default notification duration for block toasts (Phase 1)?** Too short (< 2s) may go unnoticed; too long (> 5s) may become annoying during clipboard-heavy workflows.
- **Should allow-list patterns apply before or after the heuristics?** Applying them first is the most user-friendly approach (explicit allow always wins), but it means a user could accidentally bypass all screening. Applying after the heuristics would make the allow-list a "false positive correction" tool rather than a general bypass.
- **What entropy threshold is right for Phase 3?** 4.0 bits/char is a common heuristic for random strings, but real API keys from vendors like AWS (AKIA + 16 chars) may have lower entropy than a random base64 string. Empirical testing on a corpus of known-safe and known-credential texts is needed.
