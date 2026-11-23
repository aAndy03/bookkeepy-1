# Skills (vendored)

Skill files offered by gpui-kit, downloaded as instructed — used whenever we build the
Bookkeepy 1 interface.

| Skill | Source | Contents |
|---|---|---|
| `gpui-kit/` | `github.com/longbridge/gpui-kit` @ `59d2accb…` → `skills/gpui-kit/` | `SKILL.md` + `references/` (coding-guides, conventions, recipes, usage, and the `gpui/` API references) |
| `gpui-kit-design-guides/` | same repo → `skills/gpui-kit-design-guides/` | `SKILL.md` + `references/design-guides.md` — the normative design guides for gpui-kit desktop apps |

- Docs: [GPUI Kit Documentation](https://gpui-kit.com/docs/) · components:
  [Component catalog](https://gpui-kit.com/component/).
- When working on UI code: read `gpui-kit/SKILL.md` first, then the relevant
  `references/` file for the component family; verify against the component's own page in
  the catalog. Design questions → `gpui-kit-design-guides/`.
- `skills-lock.json` records the exact upstream commit the files were vendored from; bump
  it deliberately when refreshing.
