# Contributing

Thanks for taking the time to improve Aura Translation.

## Development Setup

Install dependencies and start the Tauri app:

```bash
npm install
npm run tauri dev
```

Run checks before opening a pull request:

```bash
npm run check
npm test
```

For backend changes, also run the Rust tests when practical:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Pull Requests

- Keep changes focused on one product area or bug.
- Update README or `docs/` when user-facing behavior changes.
- Add or update tests for behavior changes.
- Do not commit generated output, local release evidence, `.env` files, API keys, or OS metadata.
- For platform-specific behavior, state what was verified on the target OS and what remains unverified.

## Product Boundaries

- Aura is local-first. Avoid introducing hosted services or telemetry without an explicit design discussion.
- Source-app paste-back is not in the supported product scope. Result usage should stay explicit through copy actions unless the scope is reopened.
- API keys should use the OS credential store by default. Plaintext storage is only an explicit fallback path.
