# References

## fintecheng.txt

`fintecheng.txt` — **Fintech Engineering Handbook: Patterns for building software that
handles money**, by Voytek Pitula (verbatim import supplied by the project owner).

We use this handbook as the authority for how Bookkeepy 1 handles money. It is **quoted
verbatim** wherever a principle is integrated, and **adapted to the offline app** in
[../docs/06-fintech-principles.md](../docs/06-fintech-principles.md) — Bookkeepy never does
live transactions or anything over the internet, so online-only topics (webhooks, payment
rails, KYC/AML, custody, trading) are explicitly not adopted; their offline equivalents are
defined there (e.g. reconciliation against imported bank statements).
