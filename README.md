Gherkin language for Rust
=========================

[![Documentation](https://docs.rs/gherkin/badge.svg)](https://docs.rs/gherkin)
[![CI](https://github.com/cucumber-rs/gherkin/workflows/CI/badge.svg?branch=main "CI")](https://github.com/cucumber-rs/gherkin/actions?query=workflow%3ACI+branch%3Amain)
[![Rust 1.46+](https://img.shields.io/badge/rustc-1.46+-lightgray.svg "Rust 1.46+")](https://blog.rust-lang.org/2020/08/27/Rust-1.46.0.html)
[![Unsafe Forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)

[Changelog](https://github.com/cucumber-rs/gherkin/blob/main/CHANGELOG.md)

A pure Rust implementation of the Gherkin (`.feature` file) language for the Cucumber testing framework.

If you want to run Cucumber tests in Rust, try [cucumber-rust](https://github.com/bbqsrc/cucumber-rust)!

## Usage

```toml
[dependencies]
gherkin = "0.11"
```

## Further information

For a detailed description of Gherkin usage, you can refer to upstream Cucumber documentation.

### Upstream documentation

1. for Cucumber **developers**:
   * [the Gherkin readme](https://github.com/cucumber/cucumber/blob/master/gherkin/README.md)
   * [the Gherkin contributing guide](https://github.com/cucumber/cucumber/blob/master/gherkin/CONTRIBUTING.md)
1. for Cucumber **users**:
   * [the Cucumber user documentation](https://cucumber.io/docs/cucumber/).
   * [the Gherkin user documentation](https://cucumber.io/docs/gherkin/).

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

[Original source](https://github.com/cucumber/cucumber/blob/master/gherkin/gherkin-languages.json) of `src/languages.json` is used under the [MIT license](https://github.com/cucumber/cucumber/blob/master/gherkin/LICENSE).
