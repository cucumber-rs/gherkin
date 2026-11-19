# Gherkin 43 step argument checkpoint

Date: 2026-11-17.

Fork: https://github.com/GQAdonis/gherkin (created for this work from
https://github.com/cucumber-rs/gherkin).
Local checkout: `/Users/gqadonis/Projects/references/gherkin`.
Base commit: `4c30e3a48a0226bf907053a0a9e738e91cf06727`.
Crate version: `gherkin 0.16.0`, Rust minimum `1.88`.
No implementation commits have been pushed.

## Bounded contract

The official grammar baseline is
https://github.com/cucumber/gherkin/blob/v43.0.0/gherkin.berp:
`StepArg` permits a data table followed by an optional doc string, or a doc
string followed by an optional data table. Each argument kind occurs at most
once on a step.

The existing Rust AST already has separate optional `table` and `docstring`
fields. The change shares a bounded argument parser across the Given, When,
Then, And, and But branches, preserving the AST API and existing zero/single
argument forms. Wildcard steps use the existing keyword machinery. Repeated
argument kinds remain unconsumed and are rejected by full feature parsing.

## Changes

- `src/parser.rs`: shared argument rule allowing both orders.
- `tests/compatibility/step_arguments.feature`: acceptance scenarios,
  authored before implementation.
- `tests/step_arguments.rs`: dedicated Cucumber BDD harness with real parser
  assertions, retained argument values, escaped table cell, and step boundary
  assertions.
- `Cargo.toml`: `step_arguments` test target with `harness = false`; includes
  its source and feature in package contents.

Acceptance contract: 18 scenarios (12 combinations of argument order and
Given/When/Then/And/But/*; 3 zero/single argument forms; 3 duplicate-kind
rejections). No unit tests, compilation, lint, or acceptance execution occurred
before completion of this bounded implementation.

## Execution checkpoint

Completed at phase end by the parent agent: 18 scenarios passed, 69 steps
passed. A Rust 2018 panic-format warning in the harness was corrected and this
focused target rerun successfully without the warning. The associated local
cucumber runner also passed 20 scenarios / 115 steps using this parser via an
explicit Cargo patch. Evidence and continuation are recorded in
`../cucumber/docs/compatibility/PHASE1.md`. Focused command from this checkout:

```sh
cargo test --test step_arguments
```

This selects the dedicated Cucumber BDD executable only; do not replace it with
an unqualified `cargo test` or a unit suite. The Cucumber fork can also exercise
this parser through a local Cargo patch to this checkout. The observed results verify only the bounded argument-order contract.

## Deliberate limits

This is not a claim of complete Gherkin 43 conformance. Official fixture corpus
coverage, doc-string media type/source-location fidelity, Messages AST/Pickle
serialization, and language catalogue synchronization remain separate work.


The native `gherkin 0.16` doc-string representation retains its leading and
trailing line breaks (for this fixture: `"\nreusable payload\n"`). This bounded
argument-order change preserves that representation. Acceptance assertions check
the exact string without trimming. Normalization to official Gherkin doc-string
content semantics is a known deferred conformance gap, requiring its own
compatibility decision and acceptance scenarios.

## Phase 6 prerequisite: literal table backslashes

Authored six additional BDD cases before changing the parser: unknown escapes,
a trailing backslash before table-cell padding, recognized pipe/backslash/newline
escapes, and a mixed Unicode case. Source is constructed inside a Given step so
the outer harness's registry parser does not pre-parse the values under test.
The Then steps assert exact rows and following-step separation.

The source-level gap was found while reading the independently pinned official
Gherkin 43 `escaped_pipes.feature` and its expected AST (curated under
`../cucumber/tests/fixtures/gherkin43`). Unknown escapes remain literal under that
contract. The existing recognized-escape alternatives are unchanged; the ordinary
cell-character fallback now permits a backslash when those alternatives do not
match. An immediately adjacent `\|` remains a recognized escaped pipe; a trailing
literal backslash is represented before cell padding, as exercised by acceptance.

The modified `src/parser.rs` was copied byte-for-byte into Cucumber's
`vendor/gherkin/src/parser.rs`; its provenance and SHA256SUMS entry were updated.
No other parser source was edited by this prerequisite task.

Phase-end command: `cargo test --test step_arguments`, exit 0. Observed
**1 feature, 24 scenarios passed, 93 steps passed** (18 prior scenarios plus six
new exact-value escape scenarios). No skipped acceptance steps or compiler
warnings were emitted. Evidence:
`../cucumber/docs/compatibility/evidence/phase6-table-escapes-bdd.log`.
No other test suite, lint/check, or premature build ran. Official whole-fixture
AST/Pickle comparison remains with the separately implemented Phase 6 converter;
these results establish only the bounded native table-escape behavior.

## Phase 6a source-preservation API

Added `Feature::parse_file_source(path, source, env)` so a runner can capture
and parse identical bytes while retaining file-path errors. `parse_path` now
shares that implementation. Native AST/API behavior remains otherwise unchanged.
The Cucumber SourceStore phase passed 5 BDD scenarios /20 steps, covering deleted
or changed input, absent EOF newline, outline pre-expansion source, configured
French dialect, and immutable snapshot generations. See
`../cucumber/docs/compatibility/PHASE6-SOURCES.md`. Vendor lib.rs synchronized.
