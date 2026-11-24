# ADR-0023 — Rates packages are ed25519-signed; app pins the mirror's public key

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** the rates mirror repo (`aAndy03/bookkeepy-1-rates`) publishes packages the
  app downloads (doc 11, `bookkeepy-rates-package@1`). TLS to GitHub + SHA-256 in the
  package manifest protects transport, but the app should verify **authorship** of the
  data, not just its integrity in transit — a compromised mirror repo (or a hijacked
  release) must not be able to feed the app poisoned rates.
- **Decision:**
  1. The mirror's CI signs every package with an **ed25519** key (minisign-compatible
     signature file `<package>.sig`); the private key lives only as a repo secret
     (`RATES_SIGNING_KEY`), never in the repo.
  2. The app pins the mirror's **public key** as primary data (`data/rates-trust.toml`,
     added at M7 with the rates engine) and refuses any package whose signature does not
     verify (`BK-RATES-0002`), before any checksum/import step.
  3. SHA-256 checksums remain as a second layer (fast integrity inside the package).
  4. The mirror CI may run on GitHub Actions on a schedule (owner-approved: it is fast
     even with many sources) — minutes are spent there, not in the main repo.
- **Consequences:** easier — strong end-to-end authenticity for rates; rotation = new
  key in `data/rates-trust.toml` (versioned, updatable by app updates). Harder — key
  rotation procedure must be documented (doc 11) and old packages re-signed or
  grandfathered by key id.
- **Rules/invariants introduced:** INV-RATES-001 (no unsigned package is ever imported);
  error code BK-RATES-0002.
- **Alternatives considered:** TLS-only (rejected — doesn't bind data to the mirror's
  identity); cosign/sigstore (rejected — heavy for a desktop app; ed25519 + pinned key is
  sufficient and auditable).
