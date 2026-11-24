# Contributing to Bookkeepy 1

Bookkeepy is a long-term, GPL-3.0-or-later project. The architecture, rules and ADRs live
in [`docs/`](docs/README.md) — read `docs/04-conventions.md` and
`docs/10-process-roadmap.md` first; they are the single source of truth.

## Setup (once)

```bash
rustup toolchain install 1.100.0   # rust-toolchain.toml pins it anyway
cargo xtask doctor                  # env self-check
cargo xtask hook install            # pre-commit gate (fmt + governors + clippy)
```

## Before pushing / opening a PR

GitHub Actions spend minutes **only on releases** — routine checks run locally:

```bash
cargo xtask check                   # fmt, clippy -D warnings, tests (nextest if installed),
                                    # governors, bench sanity, cargo deny check
```

Then follow the PR template checklist (`.github/pull_request_template.md`): rule/ADR
traceability, docs propagation (RULE-SYS-001), error codes registered in
`data/errors.toml` **and** every locale, dependencies declared on first use + rationale in
`docs/07-dependencies.md`, CHANGELOG entry, SPDX header (automatic via `xtask new-crate`
for new files; `govern lints` checks).

New crates: `cargo xtask new-crate <name>` (registers in the workspace + code registry).
New ADRs: `cargo xtask new-adr <slug>`.

## Rules of the road

- Backend logic is coded in this repo; UI work is delegated to the frontend agent via
  short prompts (RULE-PROC-002) — see `docs/04-conventions.md` §4.13.
- Never block the user (RULE-UX-001); every user-facing error has a BK-code + a message in
  every shipped locale (RULE-ERR-002/003).
- No network code outside `bookkeepy-updater` / `bookkeepy-rates` (RULE-NET-001,
  enforced by `govern net`).
- Money is integer minor units only; no floats in money code (INV-MONEY-001).

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
