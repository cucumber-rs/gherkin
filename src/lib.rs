// Copyright (c) 2018-2023  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Gherkin parser for the Cucumber test framework.
//!
//! It is intended to parse the full gamut of Cucumber .feature files that exist in the wild,
//! as there is only a _de facto_ standard for these files.
//!
//! ### .feature file structure
//!
//! The basic structure of a feature file is:
//!
//! - Optionally one or more tags
//! - Optionally `#`-prefixed comments on their own line
//! - The feature definition
//! - An optional description
//! - An optional background
//! - One or more scenarios (also taggable), each including:
//!   - One or more steps
//!   - Optionally data tables or docstrings per step
//!   - Optionally examples, which can also be tagged
//! - One or more rules (also taggable), each including:
//!   - An optional background
//!   - One or more scenarios
//!
//! ### Unparsed elements
//!
//! Indentation and comments are ignored by the parser. Most other things can be accessed via
//! properties of the relevant struct.

#![doc(
    html_logo_url = "https://avatars.githubusercontent.com/u/91469139?s=128",
    html_favicon_url = "https://avatars.githubusercontent.com/u/91469139?s=256"
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "parser")]
mod keywords;
#[cfg(feature = "parser")]
mod parser;
#[cfg(feature = "parser")]
pub mod tagexpr;

#[cfg(feature = "parser")]
use std::path::Path;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    path::PathBuf,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "parser")]
use typed_builder::TypedBuilder;

#[cfg(feature = "parser")]
pub use self::parser::{EnvError, GherkinEnv};

#[cfg(feature = "parser")]
pub fn is_language_supported(lang: &str) -> bool {
    keywords::Keywords::get(lang).is_some()
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[cfg(feature = "juniper")]
#[cfg_attr(feature = "juniper", juniper::graphql_object)]
impl Span {
    pub fn start(&self) -> i32 {
        self.start as i32
    }

    pub fn end(&self) -> i32 {
        self.end as i32
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

#[cfg(feature = "juniper")]
#[cfg_attr(feature = "juniper", juniper::graphql_object)]
impl LineCol {
    pub fn line(&self) -> i32 {
        self.line as i32
    }

    pub fn col(&self) -> i32 {
        self.col as i32
    }
}

/// A feature background
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Background {
    /// The raw keyword used in the original source.
    pub keyword: String,
    /// The name of the background.
    pub name: String,
    /// The description of the background, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub description: Option<String>,
    /// The parsed steps from the background directive.
    pub steps: Vec<Step>,
    /// The `(start, end)` offset the background directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the background directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
}

/// Examples for a scenario
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Examples {
    /// The raw keyword used in the original source.
    pub keyword: String,
    /// The name of the examples.
    pub name: Option<String>,
    /// The description of the examples, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub description: Option<String>,
    /// The data table from the examples directive.
    pub table: Option<Table>,
    /// The tags for the examples directive if provided.
    #[cfg_attr(feature = "parser", builder(default))]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the examples directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the examples directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
}

/// A feature
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Feature {
    /// The raw keyword used in the original source.
    pub keyword: String,
    /// The name of the feature.
    pub name: String,
    /// The description of the feature, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub description: Option<String>,
    /// The background of the feature, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub background: Option<Background>,
    /// The scenarios for the feature.
    #[cfg_attr(feature = "parser", builder(default))]
    pub scenarios: Vec<Scenario>,
    /// The rules for the feature.
    #[cfg_attr(feature = "parser", builder(default))]
    pub rules: Vec<Rule>,
    /// The tags for the feature if provided.
    #[cfg_attr(feature = "parser", builder(default))]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the feature directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the feature directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
    /// The path supplied for the parsed `Feature`, if known.
    #[cfg_attr(feature = "parser", builder(default))]
    #[cfg_attr(feature = "juniper", graphql(skip))]
    pub path: Option<PathBuf>,
}

#[cfg(feature = "parser")]
impl Feature {
    #[inline]
    pub fn parse_path<P: AsRef<Path>>(path: P, env: GherkinEnv) -> Result<Feature, ParseFileError> {
        let mut s =
            std::fs::read_to_string(path.as_ref()).map_err(|e| ParseFileError::Reading {
                path: path.as_ref().to_path_buf(),
                source: e,
            })?;

        if !s.ends_with('\n') {
            // Add a new line at the end, because our parser is bad and we should feel bad.
            s.push('\n');
        }

        let mut feature =
            parser::gherkin_parser::feature(&s, &env).map_err(|e| ParseFileError::Parsing {
                path: path.as_ref().to_path_buf(),
                error: env
                    .fatal_error
                    .borrow_mut()
                    .take()
                    .or_else(|| env.last_error.borrow_mut().take()),
                source: ParseError {
                    position: LineCol {
                        line: e.location.line,
                        col: e.location.column,
                    },
                    expected: e.expected.tokens().collect(),
                },
            })?;

        feature.path = Some(path.as_ref().to_path_buf());
        Ok(feature)
    }

    #[inline]
    pub fn parse<S: AsRef<str>>(input: S, env: GherkinEnv) -> Result<Feature, ParseError> {
        use std::borrow::Cow;

        let input: Cow<'_, str> = match input.as_ref().ends_with('\n') {
            true => Cow::Borrowed(input.as_ref()),
            // Add a new line at the end, because our parser is bad and we should feel bad.
            false => Cow::Owned(format!("{}\n", input.as_ref())),
        };
        parser::gherkin_parser::feature(&input, &env).map_err(|e| ParseError {
            position: LineCol {
                line: e.location.line,
                col: e.location.column,
            },
            expected: e.expected.tokens().collect(),
        })
    }
}

impl PartialOrd for Feature {
    fn partial_cmp(&self, other: &Feature) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Feature {
    fn cmp(&self, other: &Feature) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

/// A rule, as introduced in Gherkin 6.
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Rule {
    /// The raw keyword used in the original source.
    pub keyword: String,
    /// The name of the scenario.
    pub name: String,
    /// The description of the rule, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub description: Option<String>,
    /// The background of the rule, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub background: Option<Background>,
    /// The parsed scenarios from the rule directive.
    pub scenarios: Vec<Scenario>,
    /// The tags for the rule directive if provided.
    #[cfg_attr(feature = "parser", builder(default))]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the rule directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the rule directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
}

/// A scenario
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Scenario {
    /// The raw keyword used in the original source.
    pub keyword: String,
    /// The name of the scenario.
    pub name: String,
    /// The description of the scenario, if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub description: Option<String>,
    /// The parsed steps from the scenario directive.
    pub steps: Vec<Step>,
    // The parsed examples from the scenario directive if found.
    #[cfg_attr(feature = "parser", builder(default))]
    pub examples: Vec<Examples>,
    /// The tags for the scenarios directive if provided.
    #[cfg_attr(feature = "parser", builder(default))]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the scenario directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the scenario directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
}

/// A scenario step
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Step {
    /// The raw keyword used in the original source, including `But` and `And`.
    pub keyword: String,
    /// The step type for the step after parsed in context.
    pub ty: StepType,
    /// The value of the step after the type.
    pub value: String,
    /// A docstring, if provided.
    #[cfg_attr(feature = "parser", builder(default))]
    pub docstring: Option<String>,
    /// A data table, if provided.
    #[cfg_attr(feature = "parser", builder(default))]
    pub table: Option<Table>,
    /// The `(start, end)` offset the step directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the step directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
}

impl Step {
    pub fn docstring(&self) -> Option<&String> {
        self.docstring.as_ref()
    }

    pub fn table(&self) -> Option<&Table> {
        self.table.as_ref()
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", &self.keyword, &self.value)
    }
}

/// The fundamental Gherkin step type after contextually handling `But` and `And`
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLEnum))]
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum StepType {
    Given,
    When,
    Then,
}

/// A data table
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "parser", derive(TypedBuilder))]
#[cfg_attr(feature = "juniper", derive(juniper::GraphQLObject))]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Table {
    /// The rows of the data table. Each row is always the same length as the first row.
    pub rows: Vec<Vec<String>>,
    /// The `(start, end)` offset the table directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub span: Span,
    /// The `(line, col)` position the table directive was found in the .feature file.
    #[cfg_attr(feature = "parser", builder(default))]
    pub position: LineCol,
}

impl Table {
    pub fn row_width(&self) -> usize {
        self.rows.get(0).map(Vec::len).unwrap_or_default()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Error at {}:{}: {expected:?}", .position.line, .position.col)]
pub struct ParseError {
    position: LineCol,
    expected: HashSet<&'static str>,
}

#[cfg(feature = "parser")]
#[derive(Debug, thiserror::Error)]
pub enum ParseFileError {
    #[error("Could not read path: {path}")]
    Reading {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not parse feature file: {path}")]
    Parsing {
        path: PathBuf,
        error: Option<parser::EnvError>,
        #[source]
        source: ParseError,
    },
}
