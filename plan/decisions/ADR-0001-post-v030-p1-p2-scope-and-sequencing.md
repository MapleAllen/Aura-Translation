# ADR-0001: Post-v0.3.0 P1/P2 scope and sequencing

Date: 2026-06-11
Status: Accepted for coordination

## Context

The repository now has module-level current-state and future-phase documents for Config And Secrets, Runtime Readiness, Translation Profiles, Translation History, and Aura Guard. The user requested that work now begin on `P1` and `P2`, but the repository did not contain literal `P1`/`P2` labels or a pre-existing coordination plan naming those priorities.

The coordination skill requires a repository-owned execution contract on `main`, one active task owner at a time, and explicit scope before implementation starts.

## Decision

For this coordination plan:

- `P1` means trust, diagnosability, and readiness hardening.
  - Structured error routing for config/profile/history load failures
  - Provider-probe caching and invalidation
  - Tray readiness badge/tooltip parity
- `P2` means operator workflow depth for saved state and recall.
  - Per-profile independent system-stored API keys with legacy fallback
  - Translation-history search and filters
  - Retry with original provider/model/base URL via a one-shot override path

Implementation remains sequential on `main`, with one shared-source stage for `P1`, target-host verification, then one shared-source stage for `P2`, then final target-host verification.

## Consequences

- The first implementation work should target the modules named in `plan/active/post-v030-p1-p2-foundation-and-workflows/`.
- The following roadmap items are explicitly deferred out of this plan:
  - config schema versioning
  - window-placement validation
  - Linux keyring support
  - profile reorder, import/export, quick-switch
  - history export
  - Aura Guard future phases
- If the user intended different `P1`/`P2` meanings, this ADR must be amended before any source-code lock is assigned.
