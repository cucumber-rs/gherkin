# Contribution readiness

The phase47 parser repair at source commit `08dbfafc373769fa7cc58602e98179cfa63b14e0`
passed fresh focused acceptance on 2026-11-19, Rust1.88/Darwinarm64:

- Dedicated parser acceptance: **73 scenarios / 241 steps**, exit0.
- Legacy/explicit43 policy, public diagnostics and fork/vendor identity:
  **3 behavior scenarios plus3 matching controls /18 outer steps**.

[Original parser output](evidence/readiness47-parser/0001-stdout.txt),
[command receipt](evidence/readiness47-parser/0001-process.json) and the six
case/control receipts are retained together. Earlier full corpus/Markdown
consumer evidence remains historical; this is no universal-conformance claim.

Internal independent reviewer `/root/gherkin_postproof_review` examined the
entire delivered parser diff and current repairs. Exact model UNKNOWN; no
external or cross-model certification. No material component-review blocker
was found. Public diagnostics, opt-in semantics, provenance/licenses and dedicated
acceptance packaging were retained; CI/Makefile remain unchanged. Its nonblocking
finding was stale pending-proof documentation, corrected here and in README.

This is component-scoped review readiness. Shared Cucumber phase47 remains open
with separate acceptance/dependency findings, so no whole-program cleanliness,
KBD closure, merge-readiness or upstream acceptance is claimed. Existing fork
PR1 and upstream PR68/issue67 remain the contribution routes. Draft-state changes
are separately coordinated at the owner's direction; maintainers own upstream
acceptance. No registry publication or workflow change is included.
