# Phase19 official plaintext document/compiler contract

The root Cucumber corpus at ../cucumber/tests/fixtures/gherkin43-corpus adopts all
50 good and12 bad .feature files at officialGherkin43.0.0 commit
3d70e2bb4e0f16a568ee539a5ce0b03fb247a777, preserving upstream CRLF checkout rules,
MIT license, Git blob identity and SHA256 manifest. Its per-fixture BDD was
written before implementation.

Document::parse is additive and returns Option<Feature>; Feature::parse keeps
its required-feature contract. Explicit latest table mode ignores trailing
unclosed text after complete cells, as the official extra_table_content fixture
requires. Latest vocabulary chooses longest step keywords across kinds and
accepts padded Feature preambles. SourceGraph's tagged/comment/table compiler
fixes live in the root consumer; parser production files are byte-synchronized
into its vendor directory. Existing legacy flags/defaults remain unchanged.

Phase-end root corpus62/236 PASS; initial60/62 was corrected only for padded
space/tab preambles, with focused2/8 then full confirmation. Affected local
`cargo test --test step_arguments` BDD73/241 also PASS. No generic/unit suite.
Root docs/compatibility/gherkin-corpus.md links full retained evidence, including
failed correction attempts. No code publication or extra dependency fork.

The finite corpus compares complete good AST/Pickle graphs and bad rejection;
error wording/position/count, Markdown and arbitrary grammar are not claimed.

## Phase25 runtime document adoption

`Document::parse_file_source(path, source, env)` adds path/error identity while
retaining optional Feature representation. Featureless parsing now rejects bare
language directives: the pinned official grammar requires a Feature header after
Language, confirmed by actual Gherkin43 oracle acceptance. Existing Feature APIs
and legacy defaults are unchanged. The Cucumber integration opts in through
`with_gherkin43`, captures exact empty-document generations and emits no synthetic
Feature/Pickle/case. Root focused BDD passed11/41 and the entire original corpus
regression passed62/236. Evidence is archived in cucumber under
`docs/compatibility/evidence/phase25-documents-{bdd,regression}.log`.
