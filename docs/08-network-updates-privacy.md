# 08 — Network policy, updates, rates, privacy (ADR-0011)

## Offline-first: the hard rule, v2 (RULE-NET-001)

- Bookkeepy 1 is **mainly offline**. The app's only internet connections are:
  1. **App updates** — GitHub Releases of `aAndy03/bookkeepy-1` (updater crate), SHA-256
     verified, user-approved install. (F12)
  2. **Rates packages** — `manifest.json` + package files from `aAndy03/bookkeepy-1-rates`
     only (rates crate, doc 11). The app **never** connects to BNR or any other rate source
     directly.
- Exactly two crates may contain HTTP code (`updater`, `rates`), each with one client
  (`ureq` + rustls), each restricted to its allowlisted host(s). `xtask govern deps`
  (RULE-NET-001) fails the build if any other crate depends on an HTTP stack, and
  `govern net` scans for hardcoded hosts outside the allowlist. CI includes an offline smoke
  test (app fully functional with networking disabled).
- Everything else — no telemetry, no analytics, no crash upload, no bank APIs, no cloud.

## Updater behavior (F12)

User clicks **Check for updates** (or the default-off weekly check) → latest release of the
main repo is compared (semver) → release notes shown → on explicit approval the asset is
downloaded, SHA-256-verified against the published checksum, then installed. Never
auto-installs, never runs unverified code. Releases are the only distribution channel.

## Rates sync (F14, doc 11)

User clicks **Download rates** (or default-off periodic check) → fetch manifest from the
mirror repo only → download missing incrementals → verify SHA-256 → import as
`origin = "live"`. Fully resumable; works offline afterwards.

## Local system context — collected, never transmitted (point 10)

The app collects **locally only** the data it needs to function correctly on this machine:

| Data | Why | Where |
|---|---|---|
| Displays: count, resolution, scale/DPI, viewport | window opens at the right size/position on the right monitor | `system_context` table |
| GPU adapter / renderer backend | choose correct gpui render path; perf budgets | `system_context` |
| CPU model/cores, RAM total | perf budgets contextualized to this machine (RULE-PERF-001) | `system_context` |
| OS + version, app version, build | diagnostics, fatal-error reports, upgrade path | `system_context` / audit |
| Timezone, locale | correct dates/times, number formats | `settings` |
| Network state (online/offline, interface type) | know whether update/rates sync is possible | in-memory only |
| Minimal usage stats (which views opened, when) | tune prefetch (hot/warm/cold, doc 03) | `usage_stats` table |

Rules: all of it stays on the device (never sent anywhere — the only egress is the two
allowlisted syncs above, which carry no personal data); the **first-run setup dialog
discloses the collection**; **Settings → Privacy → "Data we collect"** lists every item,
its purpose, and lets the user view/clear it (usage stats and system context are deletable;
deleting them never breaks the app — defaults apply). Logs are local-only; verbose
diagnostics that could contain amounts/payees are opt-in and local-only.

## Privacy guarantees (user-facing statement)

- Your financial data never leaves your device. No accounts, no cloud, no sync, no trackers.
- The data file is plain SQLite in a folder you choose (inside the user-profile data dir by
  default, with OS file permissions) — you can read, copy, back up, delete it. (App lock /
  encryption: parked future feature, doc 10.)
- Uninstall removes only the app — never your data (INV-APP-001, ADR-0016); the uninstaller's
  "also delete my data" checkbox is unchecked by default and preceded by a full-export offer.
- Erasure: delete the data folder/file; inside the app, PII (payees, notes, attachments) is
  separable and deletable without rewriting the immutable ledger (doc 06 GDPR adaptation).
- Profiles (F18, ADR-0018) share nothing: the app-level dir holds only the profile registry
  (id/name/path), never financial data; each profile's data root is independent.

## Threat model (documented; mitigations = current posture + parked features)

- **Local attacker / stolen device:** OS file permissions on the data dir now; app lock +
  at-rest encryption parked (doc 10, M11).
- **Tampering with the books:** SHA-256 hash chain over postings + audit; `verify_chain()` on
  demand and in the self-check job (ADR-0016); tamper → BK-coded fatal surfaced for reporting.
- **Malware reading open files:** single-instance lock (ADR-0016) + no secrets stored.
- **Network:** allowlist only (updates + rates mirror); offline smoke test in CI.
