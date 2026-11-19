# Native Gherkin43 Markdown API

`gherkin::markdown::MarkdownDocument::parse(source, default_language)` returns a
typed source graph independent of Messages or JavaScript. `to_feature()` and
`to_feature_at(path)` project existing native execution structures. Default
classic Feature/Document APIs are unchanged. Fallback Markdown Feature titles
have no keyword in source metadata and project to an empty native keyword.

This alternate Gherkin matcher uses header/bullet prefixes, backtick tags,
Markdown table-separator rules and variable backtick fences; it does not render
CommonMark or parse fenced classic Gherkin blocks. Source locations, argument
order, media and delimiter metadata survive projection separately.

Root Cucumber phase28a BDD passed21 scenarios/78 steps against all five untouched
official43 Markdown fixture graphs and fifteen live pinned-oracle boundaries.
Classic regressions passed62/236,11/41,19/80. Evidence resides in
`../cucumber/docs/compatibility/evidence/phase28a-markdown-{bdd,regression}.log`.
The pinned fixture manifest is under `../cucumber/tests/fixtures/gherkin43-markdown`.

This proves the adopted finite corpus/boundaries, not all arbitrary Markdown
states or exact official diagnostic wording/recovery. Runtime source selection
and Messages source media type wiring are the next integration phase. No Node
runtime dependency, new fork, push or publication was introduced.

## Phase28b runtime integration

Cucumber now selects Plain(default), Markdown or Auto source syntax explicitly,
retains exact typed source snapshots and emits the actual source/error media.
A private ParseError detail plus additive `with_message` constructor preserves
existing public error variants and legacy formatting while retaining positioned
Markdown diagnostics. Root phase28b BDD passed14 scenarios/54 steps; evidence is
`../cucumber/docs/compatibility/evidence/phase28b-markdown-runtime-bdd.log`.
