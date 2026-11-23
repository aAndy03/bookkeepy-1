# 08 — Network policy, updates, privacy (ADR-0003)

## Offline-first: the hard rule (RULE-NET-001)

- Bookkeepy 1 is **mainly offline**. The only internet connection the app ever makes is to
  **download app updates** (and, later, only if the user explicitly enables it, other
  user-approved fetches we define as we go).
- Exactly one crate (`updater`) may contain HTTP code, with exactly one client (`ureq` +
  rustls). `cargo xtask check-deps` fails the build if any other crate depends on an HTTP
  stack. There is no analytics, no telemetry, no crash upload, no "check for news", no
  license server.
- The rest of the app must compile and run with the network cable pulled. CI includes an
  offline smoke test.

## Updater behavior (F12)

1. Only when the user clicks **Check for updates** (plus an optional, default-off,
   once-a-week automatic check — clearly documented in settings).
2. Query the GitHub Releases API of this repository (`aAndy03/bookkeepy-1`) for the latest
   release; compare with the running version using semver.
3. If newer: show release notes (changelog) and the asset list for the user's OS/arch.
4. On explicit user approval: download the asset, **verify its SHA-256 against the
   checksum published in the release** (SHA256SUMS artifact / release body), then hand
   off to the installer. Never auto-install; never run anything unverified.
5. Releases are the only distribution channel (README). Later phases add signing
   (minisign/signify) — until then installers are unsigned and the README says so.

## Privacy guarantees (user-facing statement)

- Your financial data never leaves your device. Bookkeepy 1 has no accounts, no cloud, no
  sync.
- The app contains no trackers. Logs are written locally only and never transmitted; verbose
  diagnostics that could contain amounts/payees are opt-in and local-only.
- The data file is plain SQLite (or encrypted if you enable the app lock) in a folder you
  choose — you can read it, copy it, back it up, delete it.
- Erasure: delete the data folder/file; inside the app, PII is separable and deletable
  without rewriting the immutable ledger (doc 06 §6.4 GDPR adaptation).
