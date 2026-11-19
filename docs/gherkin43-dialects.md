# Explicit Gherkin 43 dialect catalogue

Baseline: official cucumber/gherkin v43.0.0, commit
`3d70e2bb4e0f16a568ee539a5ce0b03fb247a777`, file `gherkin-languages.json`.
Git blob: `d84cbf7f929e8a3fddd42792c02a32ca6c3b4fa0`.
SHA256: `84c50533cce6f2fbb1c1e1eca14811000a69ef70b5b564075b50bdfa7333c329`.
Immutable source: https://raw.githubusercontent.com/cucumber/gherkin/3d70e2bb4e0f16a568ee539a5ce0b03fb247a777/gherkin-languages.json
Upstream MIT notice is retained as `src/LICENSE-GHERKIN`.

## Explicit compatibility policy

The original `src/languages.json` (77 dialects plus the local formal extension)
remains unchanged and remains the policy of `GherkinEnv::new`, `Default`, and
`is_language_supported`. `GherkinEnv::new_latest` and
`is_latest_language_supported` explicitly select the official 80-dialect
Gherkin 43 catalogue. Language headers retain the selected policy. The `formal`
extension is not part of the official catalogue.

`src/languages-43.json` preserves the exact downloaded bytes. Build output creates
independent keyword tables. Latest-mode arrays prefer longer matching aliases,
following the official step-keyword ordering documented in
https://github.com/cucumber/gherkin/blob/v43.0.0/javascript/src/compareStepKeywords.ts
and GherkinClassicTokenMatcher.ts. Legacy map ordering remains unchanged.

## Bounded differences

The three new dialects are Amharic (`amh`), Belarusian (`be`), and Malayalam
(`ml`). Twenty-one existing dialect entries differ: `af`, `da`, `en-old`,
`en-tx`, `eo`, `fa`, `ga`, `gj`, `it`, `ja`, `ka`, `ko`, `ne`, `nl`, `ru`,
`sr-Cyrl`, `ta`, `tr`, `uz`, `vi`, `zh-CN`. English is unchanged.
The exact before/after fields are in `gherkin43-language-differences.json`.
Changes include localized Rule keywords, the Texan Rule trailing-space fix,
new Turkish aliases, whitespace corrections, and upstream alias removals.
The explicit policy prevents removals from silently changing existing users.

## Acceptance contract and boundary

Twenty-six BDD scenarios were authored before production changes: one for each
changed/new dialect, one retaining an obsolete Persian feature alias under both
legacy constructors while rejecting it under the official policy, and one
covering dialect availability. The 24 vocabulary scenarios exercise 182 fixed
source fixtures derived from the pinned external vocabulary: all 181 added or
changed non-wildcard keyword forms plus one retained Serbian alias covering its
reordering/deduplication. Fixtures are authored data, not runtime queries against
the implementation catalogue. Then steps assert exact keyword, step type, text,
Rule/background/scenario/examples structure and table values.

All vocabulary fixtures begin with a language header and use a latest-mode
English environment, exercising catalogue retention across header switches.
Existing argument/escape scenarios remain in the same focused BDD executable.
No claim is made about every grammar production or localized Messages conversion.
The native source files are synced into Cucumber's vendor snapshot with hashes.

Phase-end command: `cargo test --test step_arguments`, exit 0 on 2026-11-17.
Observed **1 feature, 50 scenarios passed, 172 steps passed**, with no skipped
acceptance steps or compiler warnings. This includes 24 previously adopted
argument/escape scenarios and 26 new dialect-policy scenarios. All 182 fixed
keyword-source cases passed exact structure/text assertions.

Evidence: `../cucumber/docs/compatibility/evidence/gherkin43-dialects-bdd.log`.
No failed attempts or production corrections were needed after this phase's
execution began. No unit suite, generic cargo test, lint/check, or premature
build ran. Cucumber-level catalogue selection/capture integration remains a
separate parent-owned BDD phase; no claim is made here about its outcome.
