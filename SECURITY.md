# Security policy — Bookkeepy 1

Bookkeepy is offline-first: it never sends your financial data anywhere (doc 08). The
only network connections are app updates from this repo's Releases and rates packages
from the `aAndy03/bookkeepy-1-rates` mirror — both signature/checksum verified.

## Reporting a vulnerability

**Please do not open a public issue for security reports.** Use GitHub's private
vulnerability reporting: this repo's **Security tab → "Report a vulnerability"**
(Security Advisories). You will get a response within **7 days**.

Include, when possible: affected version/build (About screen or a copied
`bookkeepy-diagnostic@1` bundle), OS, steps to reproduce, and impact. Diagnostic bundles
contain build info + error codes only — never financial data.

## Scope

In scope: the app binaries from Releases, the updater/rates download paths, the storage
format (data integrity/tamper-evidence), the rates mirror pipeline. Out of scope:
threats that require physical access to the unlocked machine or a compromised OS.

## Safe harbor

We will not pursue legal action against good-faith research that follows this policy
(no data exfiltration, no social engineering, reasonable rate limits).
