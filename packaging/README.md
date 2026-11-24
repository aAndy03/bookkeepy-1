# Packaging — installers / uninstallers (item 12 support, M0.5)

Bookkeepy 1 ships as signed installers built from the GitHub Releases of
`aAndy03/bookkeepy-1`. Release asset wire format: **docs/08 "Release wire format v1"**.

## Invariants

- **INV-APP-001 — uninstall never deletes user data.** The uninstaller removes only the
  app binaries/registration. The data tree (app data dir, doc 05 §5.2: `profiles.toml` +
  `profiles/<id>/…`) is left untouched. The uninstall wizard offers an **opt-in** checkbox
  "Also delete my data", **unchecked by default**; when checked it requires typing the
  profile name(s) and shows a final export offer first (F17 full export, doc 05 §5.6 —
  the export explains the data can be reused in other apps and that reinstalling finds the
  data unaltered if not deleted).
- **Single instance** (doc 10 round-2 item 1): a second launch focuses the existing window.
- **Seamless updates** (INV-DATA-001): updates never touch the data tree; migrations are
  in-app, transactional, and reversible-by-backup (doc 05 §5.4).

## Per-OS plan (activates with M3 app crate)

| OS | Installer | Uninstaller | Notes |
|---|---|---|---|
| Windows | MSI via WiX (or NSIS) | MSI Remove | registry uninstall entry; data in `%APPDATA%\Bookkeepy 1\data` |
| macOS | `.dmg` + signed `.app` (notarization) | drag-to-trash removes app only | data in `~/Library/Application Support/Bookkeepy 1/data` |
| Linux | `.AppImage` + `.deb`/`.rpm` | package remove / AppImage delete | data in `~/.local/share/bookkeepy-1/data` |

Signing keys live outside the repo (CI secrets); unsigned builds are marked as such in the
About screen. Auto-update checks only GitHub Releases of the main repo (RULE-NET-001) and
applies via the wire format in doc 08.
