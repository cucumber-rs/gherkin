# Explicit Gherkin 43 table sizing

Phase 15 adds an independent opt-in policy: `TableSemantics::Gherkin43` via
`GherkinEnv::with_table_semantics`. Default, `new`, and `new_latest` retain
`TableSemantics::Legacy`, including truncation of later rows longer than the first.
Legacy shorter rows continue to fail as before. Strict mode preserves all parsed
cells and rejects any subsequent row with a different cell count, for step tables
and Examples tables. No public AST shape changes are required.

## External contract and acceptance

Pinned upstream: cucumber/gherkin v43.0.0, commit
`3d70e2bb4e0f16a568ee539a5ce0b03fb247a777`.
[AstBuilder.ensureCellCount](https://github.com/cucumber/gherkin/blob/v43.0.0/javascript/src/AstBuilder.ts#L120)
rejects both longer and shorter rows. Exact source/error fixtures and their MIT
notice are retained in `tests/fixtures/gherkin43-tables`.

The `@phase15` acceptance scenarios were authored before production edits.
They isolate shorter/longer rows in step and Examples tables, check preservation
of valid values, preserve six legacy constructor/kind combinations, and require
the typed inconsistent-cell-count diagnostic for the published invalid fixture.
The native parser remains fail-fast; official multi-error aggregation and exact
error positions/messages are outside this phase. Table escaping, whitespace,
trailing text and empty table grammar are unchanged.

Phase-end BDD evidence (2026-11-17): `cargo test --test step_arguments`
passed on the first run: 73 scenarios and 241 steps, including all 13 new
Phase15 scenarios (39 steps). Retained log in the Cucumber checkout:
`docs/compatibility/evidence/phase15-tables-parser-bdd.log`. No unit suites,
generic tests, lint or build checks ran in this phase.
