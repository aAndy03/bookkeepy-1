# ADR-0002 — License: GPL-3.0-or-later

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** Bookkeepy is open source and distributed through GitHub; the user requires
  "GNU v3 and later".
- **Decision:** `license = "GPL-3.0-or-later"` in every crate; `LICENSE` = GPLv3 text;
  repo created with the `gpl-3.0` license. Users may choose any later version.
- **Consequences:** all dependencies must be GPLv3-compatible (enforced by `cargo deny`);
  any AGPL dependency would force source disclosure on *network use* — we have none, but we
  still ban AGPL to keep the promise simple.
- **Alternatives:** MIT/Apache — rejected: user explicitly chose GPLv3+ to keep derivatives
  free.
