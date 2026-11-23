# 04 — Conventions: how every piece of Bookkeepy 1 code is shaped

Goal: additions, refactors, subtractions and deprecations are easy and safe. Everything is
standardized, patterned, pipelined, grouped, owned, contextualized, API'd, validated and
engine'd. Rules have IDs (`RULE-<AREA>-<NNN>`); invariants have IDs (`INV-<AREA>-<NNN>`).
Governors (later: `cargo xtask …`, CI) enforce the mechanical ones; tests enforce the
invariants.

## 4.1 Primary data (RULE-DATA-001, ADR-0008)

- Sets of primary data (constants/variables the app is built on) live as data files in
  `data/*.toml`, e.g. `data/currencies.toml` (ISO 4217). **No such list is ever hardcoded
  in source code.**
- They are compiled into the binary via `include_str!` by the `primary-data` crate, which
  parses once at startup into typed, validated structures; invalid primary data fails at
  startup, loudly.
- Planned primary data: `currencies.toml` (done), `countries.toml`, `languages.toml`,
  `category-presets.toml` (default chart of categories), `date-formats.toml`,
  `default-accounts.toml` (suggested starter accounts).
- Changing a primary data file is a data change, not a code change: it requires a
  changelog entry and, if the schema changes, an ADR. Files carry their source and
  publication date in a `[source]` header.
- User-level customization (their own categories etc.) is *not* primary data; it lives in
  the user's database.

## 4.2 Crate anatomy (RULE-ARCH-002)

Every domain crate has the same skeleton, so any contributor knows where things live:

```
crates/<domain>/
├── README.md            what this crate owns, its invariants, its ADR links
├── src/
│   ├── lib.rs           pub mod api; everything else is private
│   ├── api.rs           THE public API: pub fns / pub trait + DTOs from contracts; the only import other crates use
│   ├── model.rs         domain types (smart constructors; invalid states unrepresentable)
│   ├── engine_<name>.rs business state machines / core logic ("engine_" prefix)
│   ├── pipeline_<name>.rs multi-step flows as staged, resumable pipelines ("pipeline_" prefix)
│   ├── validate.rs      RULE-VAL-001 boundary validation for every input
│   ├── error.rs         thiserror error enum for the crate
│   └── events.rs        domain events emitted by the crate
```

- `api.rs` is the crate's contract. Internal refactors never change it without an ADR.
- Engines are pure-ish: they take validated input + a storage handle, return `Result<_, DomainError>`.
- Pipelines are staged: each stage is a function `StageInput -> Result<StageOutput>`;
  progress is persisted so a pipeline can resume after a crash (fintecheng "full
  resumability", doc 06).

## 4.3 Validation (RULE-VAL-001)

- All external input (user input, imported files, settings) is validated at the boundary,
  in `validate.rs`, before any engine runs. Engines additionally assert preconditions.
- Prefer making invalid states unrepresentable (smart constructors, newtypes, enums) over
  runtime errors — fintecheng: invariants enforced "by construction" first (doc 06).

## 4.4 Errors (RULE-ERR-001/002)

- Each crate defines a `thiserror` error enum; `?` propagates; the binary converts to
  `anyhow` at the edge.
- No `unwrap()`/`expect()`/`panic!` outside tests (`clippy::unwrap_used/expect_used = deny`);
  `dbg!` and `todo!` denied workspace-wide; `unsafe_code = forbid`.
- Errors are actionable: they say what failed, which input, and what to fix.

## 4.5 Money (INV-MONEY-001, ADR-0004)

- Money is **never** `f64`/`f32`. Amounts are `i64` minor units inside a `Money` newtype
  pairing amount + `CurrencyCode` (fintecheng precision handling, doc 06).
- Serialized money is a string or integer minor units — never a bare JSON number
  (fintecheng: a bare JSON number reintroduces floating point at the edge).
- `clippy::float_arithmetic` is denied in `core` and all money-touching crates.
- Currency codes are validated against `data/currencies.toml` at the boundary
  (fintecheng: "controlled currency set… never accept arbitrary currency codes").

## 4.6 Events & ownership (RULE-EVT-001)

- Every state change emits a domain event from `contracts` (what happened, when, who/what
  triggered it, why — the audit-trail fields, doc 06).
- A local in-process event bus dispatches to subscribers: `audit` (append-only log),
  `search` (tantivy write-through), `ui` (cx.notify). Subscribers must be idempotent.
- Ownership: one crate owns each table/entity set; other crates go through its `api`.

## 4.7 UI conventions (RULE-UI-001…004, ADR-0007)

- UI uses **all** gpui-kit styled components (catalog-first); view code follows the
  vendored skills in `skills/gpui-kit/` (family conventions, recipes) and
  `skills/gpui-kit-design-guides/`.
- `ui` depends only on `app-services::api` + `contracts` (RULE-UI-004); presenters map
  domain → view models; no storage/engine internals leak into views.
- One module per screen/feature inside `ui/src/views/<feature>/`; shared widgets in
  `ui/src/widgets/`.

## 4.8 Naming

- Crates: `bookkeepy-<domain>`; modules as in 4.2; engines `engine_*`, pipelines
  `pipeline_*`, view modules `views/*`, presenters `presenters/*`.
- IDs: UUIDv7 for entities; import dedup uses explicit import-scoped idempotency keys
  (fintecheng idempotency: "prefer explicit keys").
- Time: all timestamps stored as UTC instants (i64 epoch millis) + the user's value-date
  separately (value vs booking time, doc 06); rendered in the user's locale/timezone.

## 4.9 Documentation & decisions

- Every public item has doc comments (`missing_docs = warn` → fix to deny in CI).
- Every crate has a README; the docs index is `docs/README.md`.
- Architecture-affecting decisions are ADRs in `docs/adr/` (immutable once accepted;
  supersede, never edit). Rule changes need an ADR too.
- `CHANGELOG.md` follows Keep a Changelog; every user-visible change gets an entry.

## 4.10 Git & CI hygiene (RULE-PROC-001)

- `main` is protected: no direct pushes, PRs only, required CI (doc 09), conventional
  commit messages (`feat:`, `fix:`, `refactor:`, `docs:`, `adr:` …) referencing the issue.
- Branch naming: `<type>/<short-name>`; PRs link an issue; ADRs land with the PR they
  describe.

## 4.11 Change checklist (for every PR)

1. Does it fit an existing crate's ownership (4.2)? If not → ADR first.
2. New/changed rule or invariant? → update this doc + ID registry; governor if mechanical.
3. Primary data touched? → 4.1 process.
4. Validation at the boundary (4.3)? Errors per 4.4? Money per 4.5?
5. Events emitted (4.6)? Search/audit updated?
6. Tests: unit + property/golden as applicable (doc 09); docs updated; CHANGELOG entry.
