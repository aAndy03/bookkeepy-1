# 11 — Live FX rates via the mirror sub-repo (points 2, 2.1–2.4; ADR-0011/0012)

## The rule

Bookkeepy 1 **never connects directly to any live-rates source**. Rates reach the app only as
immutable, checksummed packages published by the mirror sub-repo
**[aAndy03/bookkeepy-1-rates](https://github.com/aAndy03/bookkeepy-1-rates)**. The app always
works offline with whatever packages are already downloaded; "Download rates" re-syncs from
the mirror only. (ADR-0011 supersedes ADR-0003's network scope; doc 08.)

## The mirror sub-repo

- Automation (GitHub Actions on a schedule + manual dispatch, added in milestone M7) pulls
  from public sources — **first source: BNR**, cursurile pieței valutare XML
  (https://www.bnr.ro/23988-cursurile-pietei-valutare-in-format-xml , feed `nbrfxrates.xml`).
  More sources are added over time, each with its own `source.id`.
- It normalizes each fetch into a package, validates currency codes against ISO 4217
  (`data/currencies.toml` from the main repo — **2.1: rates are linked to currencies.toml**,
  RULE-DATA-002), appends it to `manifest.json`, and publishes. Packages are **immutable**:
  sources add, never rewrite history (**2.4**); corrections are `kind = "addendum"` packages.
- Format spec (SSOT, pinned by version): `bookkeepy-1-rates/docs/package-format.md`
  (`bookkeepy-rates-package@1`). Rates are decimal **strings**, directional, per
  `base_currency` (BNR quotes RON), with `multiplier` support; SHA-256 in the manifest;
  incremental packages per source per date (`kind = "incremental"`, `supersedes` chain) plus
  periodic `full` packages. Signing (minisign) is layered on later.

## App side (`rates` crate + `engine_rates`)

1. **Sync:** fetch `manifest.json` (small) from the mirror repo only → diff against installed
  `rates_packages` → download missing/incremental package files → verify SHA-256 → store
  read-only in `<data dir>/rates/packages/<source>/<date>.json` → import rows into
  `fx_rates` with `origin = "live"`, `source`, `package_id`, `fetched_at`. Never blocks the UI
  (RULE-ASYNC-001); progress events; resumable; runs on user action or the documented
  default-off periodic check.
2. **Storage:** `fx_rates(source, currency, base_currency, date, rate, multiplier, origin:
  live|manual, package_id, hash, created_at, note)`. Live rows are immutable app-side.
3. **Manual rates & adjustments:** user-entered or user-adjusted rows get `origin = "manual"`.
  They are ordinary user data: **deleting them is irreversible** (UI warns explicitly). When
  no suitable row exists, the default chain in `data/defaults.toml` applies (see doc 05 §5.3).
4. **Source choice (2.3/2.4):** each row keeps the source it came from; switching the user's
  primary source changes only *future* defaults/lookups — historical rows keep their computed
  rates and are only changed by explicit manual update. The rates UI shows all accumulated
  sources side by side and lets the user pick which rate a conversion or report uses.
5. **UI (F14):** rates browser (per currency pair, per source, with date + source badge),
  primary-source picker, manual rate editor, "adjust this live rate" (creates a manual
  override), "Download rates now", last-sync status, and a visible note that manual entries
  cannot be recovered once deleted.

## Offline behavior

With zero packages downloaded the app still works; conversions without any rate produce
`BK-FX-0002` (user-facing, copyable) prompting the user to download rates or enter a manual
one. The app never falls back to the network silently.

## Trust & automation (ADR-0023)

The mirror repo runs a scheduled + manual GitHub Actions workflow that fetches the
sources (BNR first), builds `bookkeepy-rates-package@1` packages, sha256-checksums them
and **ed25519-signs** each package (secret `RATES_SIGNING_KEY`; public key committed at
`keys/rates-ed25519.pub.pem`). The app pins that pubkey and refuses any package with a bad
or missing signature (`BK-RATES-0002`) before checksum/import. Packages stay immutable;
corrections are `kind = "addendum"` packages.

## History backfill (2005 → today, already in the mirror)

BNR publishes yearly archives (`curs.bnr.ro/files/xml/years/nbrfxratesYYYY.xml`). The
mirror repo **already contains** them as `kind: "archive"` packages
(`packages/bnr/archive-2005.json … archive-2026.json`, spec addition documented in the
mirror's `docs/package-format.md`), plus daily `kind: "full"` packages from the latest
quote feed. The **scheduled** job (`update-rates.yml`, banking days after the 13:00
update) fetches ONLY `https://curs.bnr.ro/nbrfxrates.xml`; archives are built by the
manual `backfill.yml`. So: history is in the repo now, automation only chases the latest.
