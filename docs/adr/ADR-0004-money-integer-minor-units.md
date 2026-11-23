# ADR-0004 — Money is integer minor units inside a `Money` newtype; never floats

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** fintecheng Precision handling: floats lose precision; minor units are the
  right storage; "don't assume it's always 2" decimals; serialize as string or minor-unit
  integer, never a bare JSON number (doc 06 §6.2).
- **Decision:** `Money { amount_minor: i64, currency: CurrencyCode }` in `core`;
  `CurrencyCode` validated against `data/currencies.toml`; no f64/f32 in money code
  (`clippy::float_arithmetic` denied in money crates); rounding explicit, at boundaries, with
  an explicit rounding account for residuals.
- **Invariants:** INV-MONEY-001.
- **Alternatives:** f64 (rejected — precision loss), BigDecimal everywhere (rejected —
  complexity; i128 minor units headroom is enough for personal finance, documented limit).
