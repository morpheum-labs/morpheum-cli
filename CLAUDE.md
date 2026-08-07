<!-- morpheum-claude-framework v2026-08-07 — shared blocks synced by sync.sh; edit prose freely -->
# morpheum-cli

The official Morpheum CLI (`morpheum` binary): a thin wrapper over the Rust `morpheum-sdk`.
clap command tree → dispatcher → per-module `tx/` and `query/` implementations, with
keyring-backed key storage and gRPC transport.

**This repo is PUBLIC on GitHub.** Everything committed here is public content.

## Layout

- `src/cli.rs` (clap tree) → `src/dispatcher.rs` → `src/tx/*`, `src/query/*` (one file per
  chain module) — follow this shape for new commands
- `src/keyring.rs` — OS-keyring key storage (`keyring` + `secrecy`)
- `docs/` — `two-layers-architecture.md`, `command-structure.md` — keep current
- Every chain module is behind a cargo feature; `default = []`, `modules` enables all

## Commands `[host]` (builds on the host; needs sibling checkouts)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --features modules
```

## Invariants

- **Thin wrapper**: business logic, signing, and encoding live in the SDK; this repo does
  argument parsing, transport wiring, output formatting, and key management. If a change
  needs real logic, it belongs upstream.
- Secret material stays in the OS keyring behind `secrecy` types — never print, log, or
  export it; `unsafe_code = "forbid"` stays.
- New module commands ship with their cargo feature wired into `modules` and an
  `assert_cmd` integration test.
- CI must check out the **transitive** sibling closure (a dependency reached via
  `morpheum-primitives` once broke it) — keep `.github/workflows/ci.yml`'s list complete
  when deps change.

<!-- framework:begin ripple -->
## Cross-repo ripple

- Depends on siblings (11 path deps): seven `../morpheum-sdk/crates/*` crates,
  `../morpheum-signing` (native), `../morpheum-proto`, `../morpheum-primitives`.
- Dependents: `orchestrator` e2e suites drive the built `morpheum` binary (each module
  suite has a `cli/` crate).
- Most changes here originate upstream (sdk/proto); when the SDK surface moves, this repo
  moves in the same PR batch. CI clones the transitive sibling closure — keep its checkout
  list in `.github/workflows/ci.yml` complete.
<!-- framework:end ripple -->

## Verification

- CI = fmt + workspace clippy `-D warnings` + workspace tests (`--all-features`). Run the
  same three locally; `tests/integration.rs` exercises the binary via `assert_cmd`.
