# Explicit classic lexical semantics

`GherkinEnv::with_lexical_semantics(LexicalSemantics::Gherkin43)` selects
ECMAScript token whitespace and trimming with Unicode scalar token columns.
`Legacy` remains the default for every constructor, including `new_latest`,
which selects vocabulary independently. Byte spans retain original offsets;
no rewritten source is parsed. Table-cell trimming is unchanged.

The root Cucumber `with_gherkin43` convenience opts into this additional policy;
separate builders remain available. Its immutable source snapshots carry the
policy through source conversion and actual Examples-row selection. Acceptance
is authored first in root `tests/features/lexical_semantics/lexical.feature`,
with whole AST/Pickle differential comparison and real native handlers.
Implementation is complete; phase-end BDD is pending. No unit suite or generic
check has run, and nothing has been published. Astral table-cell UTF16 column
alignment is explicitly deferred to Phase38b.


Root primary BDD result:25 scenarios/110 steps passed, warning-free at that
boundary. The first24-case runtime attempt exposed Unicode preamble/tag padding;
its correction and a PEG syntax correction are retained in root evidence. The
25th feature-first case covers independently selected Legacy vocabulary with
Gherkin43 lexical semantics. Existing sibling BDD regressions remain separately
coordinated; this result is the root integration target, not a unit-suite claim.


Affected verification completed: the sibling's focused BDD command
`cargo test --test step_arguments` passed73 scenarios/241 steps; root's25 affected
BDD targets passed408 scenarios/1,791 steps. Logs are retained under root
`docs/compatibility/evidence/phase38-parser-affected-bdd.log` and
`phase36-37-38-affected-bdd.log`. No unit suite was run.
