# Translation Engine Plan

## Objective

Evolve the Translation Engine into a stable, provider-agnostic streaming service that can tolerate transient failures, preserve partial output on cancellation, and remain testable as provider rules expand. The current architecture already supports shared HTTP infrastructure, request-scoped cancellation, and provider-specific header policies; the next work is focused on observability, richer usage data, and tighter regression coverage around edge cases.

## Design Principles

- **Event payloads are the public contract.** `translation-chunk`, `translation-done`, `translation-error`, and `translation-retry` must remain request-scoped.
- **Transport concerns stay backend-side.** Retry policy, header policy, and SSE parsing belong in Rust, not in the UI shell.
- **Cancellation preserves user value.** Cancelling should keep partial translated text available whenever data already arrived.
- **Provider differences are additive.** New providers should extend header and base-URL policy without forking the stream parser.
- **Malformed streams fail loudly.** Invalid SSE payloads should emit a terminal error instead of being swallowed.

---

## Phase 1: Streaming Translation MVP - DONE

Status: **Done**

Goals:

- Ship end-to-end streaming translation over OpenAI-compatible chat completions.

Completed work:

- Implemented `translate_text`.
- Implemented SSE line parsing with incremental chunk emission.
- Added `translation-chunk`, `translation-done`, and `translation-error` events.
- Added source-language auto-detect prompt construction.

---

## Phase 2: Shared Client & Cancellation - DONE

Status: **Done**

Goals:

- Remove per-request setup waste and support cancellation.

Completed work:

- Reused a shared `reqwest::Client` via Tauri state.
- Added `CancellationRegistry`.
- Added `request_id`-scoped cancellation with `cancel_translate`.
- Preserved partial text by emitting `translation-done` on cancel.

---

## Phase 3: Retry & Provider Policies - DONE

Status: **Done**

Goals:

- Support transient-failure retries and provider-specific auth behavior.

Completed work:

- Added retry handling for transport errors and HTTP 5xx.
- Added `translation-retry` events.
- Added provider-specific headers for DeepSeek, OpenRouter, and Ollama.
- Added trailing-slash-safe completion URL building.

---

## Phase 4: Parser & Provider Regression Coverage - PARTIAL

Status: **Partial**

Goals:

- Lock down SSE parsing and provider behavior with repeatable tests.

Completed work:

- Added parser tests for valid chunks, `[DONE]`, ignored comment lines, malformed JSON, and CRLF-normalized input.
- Added provider header tests for OpenRouter and DeepSeek behavior.
- Added request tests confirming Ollama omits auth headers.
- Added retry-path coverage using sequenced mock responses.
- Added cancellation coverage ensuring `Done` emits without an error.
- Added malformed-stream coverage ensuring terminal error emission.

Remaining features:

- Add coverage for multi-line payload fragments split across several TCP chunks.
- Add coverage for stream termination without `[DONE]` when some chunks were already emitted.
- Add coverage for registry cleanup behavior at the command layer, not only inside the stream core.

---

## Phase 5: Usage & Cost Observability - NOT STARTED

Status: **Not Started**

Goals:

- Surface request usage metrics and cost-related diagnostics.

Remaining features:

- Parse provider usage payloads from terminal SSE chunks where available.
- Emit a request-scoped usage event after completion.
- Store recent usage records in backend state.
- Expose usage history to the frontend.

---

## Phase 6: Configurable Runtime Policy - NOT STARTED

Status: **Not Started**

Goals:

- Move hardcoded translation-runtime knobs into config when it improves UX or operations.

Remaining features:

- Make retry policy configurable.
- Make temperature configurable.
- Decide whether provider-specific timeout policy belongs in config or remains backend-defined.

## Implementation Rules

- Do not remove `request_id` from any public translation event.
- Do not retry HTTP 4xx responses.
- Do not treat malformed SSE JSON as ignorable noise.
- Do not rebuild the shared HTTP client per request.
- Do not leak provider-specific header rules into the frontend.

## Open Questions

- **Usage source of truth:** Should token accounting rely only on provider-reported usage fields, or should the app estimate usage when a provider omits them?
- **Retry UX contract:** Should the backend eventually include retry delay metadata in the event payload so the UI can communicate wait duration more precisely?
