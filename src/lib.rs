// Copyright (c) 2018-2020  Brendan Molloy <brendan@bbqsrc.net>
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
//!   - One or more scenarios
//!
//! ### Unparsed elements
//!
//! Indentation and comments are ignored by the parser. Most other things can be accessed via
//! properties of the relevant struct.

mod parser;
pub mod tagexpr;

// Re-export for convenience
pub use peg::error::ParseError;
pub use peg::str::LineCol;

use typed_builder::TypedBuilder;

use std::path::Path;

/// A feature background
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Background {
    /// The parsed steps from the background directive.
    pub steps: Vec<Step>,
    /// The `(start, end)` offset the background directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
}

/// Examples for a scenario
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Examples {
    /// The data table from the examples directive.
    pub table: Table,
    /// The tags for the examples directive if provided.
    #[builder(default)]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the examples directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
}

/// A feature
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Feature {
    /// The name of the feature.
    pub name: String,
    /// The description of the feature, if found.
    #[builder(default)]
    pub description: Option<String>,
    /// The background of the feature, if found.
    #[builder(default)]
    pub background: Option<Background>,
    /// The scenarios for the feature.
    #[builder(default)]
    pub scenarios: Vec<Scenario>,
    /// The rules for the feature.
    #[builder(default)]
    pub rules: Vec<Rule>,
    /// The tags for the feature if provided.
    #[builder(default)]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the feature directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
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
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Rule {
    /// The name of the scenario.
    pub name: String,
    /// The parsed scenarios from the rule directive.
    pub scenarios: Vec<Scenario>,
    /// The tags for the rule directive if provided.
    #[builder(default)]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the rule directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
}

/// A scenario
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Scenario {
    /// The name of the scenario.
    pub name: String,
    /// The parsed steps from the scenario directive.
    pub steps: Vec<Step>,
    // The parsed examples from the scenario directive if found.
    #[builder(default)]
    pub examples: Option<Examples>,
    /// The tags for the scenarios directive if provided.
    #[builder(default)]
    pub tags: Vec<String>,
    /// The `(start, end)` offset the scenario directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
}

/// A scenario step
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Step {
    /// The step type for the step after parsed in context.
    pub ty: StepType,
    /// The original raw step type, including `But` and `And`.
    pub raw_type: String,
    /// The value of the step after the type.
    pub value: String,
    /// A docstring, if provided.
    #[builder(default)]
    pub docstring: Option<String>,
    /// A data table, if provided.
    #[builder(default)]
    pub table: Option<Table>,
    /// The `(start, end)` offset the step directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
}

/// The fundamental Gherkin step type after contextually handling `But` and `And`
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum StepType {
    Given,
    When,
    Then,
}

/// A data table
#[derive(Debug, Clone, TypedBuilder, PartialEq, Hash, Eq)]
pub struct Table {
    /// The rows of the data table. Each row is always the same length as the first row.
    pub rows: Vec<Vec<String>>,
    /// The `(start, end)` offset the table directive was found in the .feature file.
    #[builder(default)]
    pub span: (usize, usize),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseFileError {
    #[error("Could not read path: {0}")]
    Reading(std::path::PathBuf, #[source] std::io::Error),

    #[error("Could not parse feature file: {0}")]
    Parsing(
        std::path::PathBuf,
        #[source] peg::error::ParseError<peg::str::LineCol>,
    ),
}

impl Feature {
    #[inline]
    pub fn parse_path<P: AsRef<Path>>(path: P) -> Result<Feature, ParseFileError> {
        let s = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ParseFileError::Reading(path.as_ref().to_path_buf(), e))?;
        parser::gherkin_parser::feature(&s, &Default::default())
            .map_err(|e| ParseFileError::Parsing(path.as_ref().to_path_buf(), e))
    }

    #[inline]
    pub fn parse<S: AsRef<str>>(input: S) -> Result<Feature, ParseError<LineCol>> {
        parser::gherkin_parser::feature(input.as_ref(), &Default::default())
    }
}

impl Step {
    pub fn docstring(&self) -> Option<&String> {
        match &self.docstring {
            Some(v) => Some(&v),
            None => None,
        }
    }

    pub fn table(&self) -> Option<&Table> {
        match &self.table {
            Some(v) => Some(&v),
            None => None,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.raw_type, &self.value)
    }
}
