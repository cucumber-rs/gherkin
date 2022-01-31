`gherkin` crate changelog
=========================

All user visible changes to `gherkin` crate will be documented in this file. This project uses [Semantic Versioning 2.0.0].




## [0.12.0] · 2022-??-??
[0.12.0]: /../../tree/v0.12.0

[Diff](/../../compare/v0.11.1...v0.12.0)

### Added

- Support for text after `Background` and `Examples` ([#32])

[#32]: /../../pull/32




## [0.11.1] · 2021-12-08
[0.11.1]: /../../tree/v0.11.1

[Diff](/../../compare/v0.11.0...v0.11.1)

### Fixed

- Allowed keywords in `Feature`s `Description`. ([#30], [cucumber#175])
- Allowed characters in `Tag`s. ([#30], [cucumber#174])
- Comments on the same line with `Tag`s. ([#30])
- `Tag`s requiring whitespaces between them. ([#30])
- [Escaping][0111-1] in `TagOperation`. ([#30])

[#30]: /../../pull/30
[cucumber#174]: https://github.com/cucumber-rs/cucumber/issues/174
[cucumber#175]: https://github.com/cucumber-rs/cucumber/issues/175
[0111-1]: https://github.com/cucumber/tag-expressions/tree/6f444830b23bd8e0c5a2617cd51b91bc2e05adde#escaping




## [0.11.0] · 2021-12-06
[0.11.0]: /../../tree/v0.11.0

[Diff](/../../compare/v0.10.2...v0.11.0)

### BC Breaks

- Renamed crate from `gherkin_rust` to just `gherkin`. ([d8803b80])
- Yank [0.10.2] version, as it's appeared to be backwards incompatible.

[d8803b80]: /../../commit/d8803b808eb5bd2684b9dc7c868a9637a0398100




## [0.10.2] · 2021-11-29
[0.10.2]: /../../tree/v0.10.2

[Diff](/../../compare/v0.10.1...v0.10.2)

### Added

- Support of multiple `Examples` in `Scenario Outline`. ([#29])

[#29]: /../../pull/29




[Semantic Versioning 2.0.0]: https://semver.org
