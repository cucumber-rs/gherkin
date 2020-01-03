// Copyright (c) 2018  Brendan Molloy <brendan@bbqsrc.net>
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

#[doc(hidden)]
pub extern crate pest;

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate derive_builder;

mod parser;
pub mod tagexpr;

use pest::iterators::{Pair, Pairs};

/// A feature background
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
pub struct Background {
    /// The parsed steps from the background directive.
    pub steps: Vec<Step>,
    /// The `(line, col)` position the background directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

/// Examples for a scenario
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
pub struct Examples {
    /// The data table from the examples directive.
    pub table: Table,
    /// The tags for the examples directive if provided.
    #[builder(default)]
    pub tags: Option<Vec<String>>,
    /// The `(line, col)` position the examples directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

/// A feature
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
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
    pub scenarios: Vec<Scenario>,
    /// The rules for the feature.
    pub rules: Vec<Rule>,
    /// The tags for the feature if provided.
    #[builder(default)]
    pub tags: Option<Vec<String>>,
    /// The `(line, col)` position the feature directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

/// A rule, as introduced in Gherkin 6.
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
pub struct Rule {
    /// The name of the scenario.
    pub name: String,
    /// The parsed scenarios from the rule directive.
    pub scenarios: Vec<Scenario>,
    /// The tags for the rule directive if provided.
    #[builder(default)]
    pub tags: Option<Vec<String>>,
    /// The `(line, col)` position the rule directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

/// A scenario
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
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
    pub tags: Option<Vec<String>>,
    /// The `(line, col)` position the scenario directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

/// A scenario step
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
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
    /// The `(line, col)` position the step directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

/// The fundamental Gherkin step type after contextually handling `But` and `And`
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum StepType {
    Given,
    When,
    Then,
}

/// A data table
#[derive(Debug, Clone, Builder, PartialEq, Hash, Eq)]
pub struct Table {
    /// The headers of the data table.
    pub header: Vec<String>,
    /// The rows of the data table. Each row is always the same length as the `header` field.
    pub rows: Vec<Vec<String>>,
    /// The `(line, col)` position the table directive was found in the .feature file.
    #[builder(default)]
    pub position: (usize, usize),
}

impl StepType {
    pub fn as_str(&self) -> &str {
        match self {
            StepType::Given => "Given",
            StepType::When => "When",
            StepType::Then => "Then",
        }
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
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.raw_type, &self.value)
    }
}

fn parse_tags(outer_rule: Pair<'_, parser::Rule>) -> Vec<String> {
    let mut tags = vec![];

    for rule in outer_rule.into_inner() {
        if rule.as_rule() == parser::Rule::tag {
            let tag = rule.as_span().as_str().to_string();
            tags.push(tag);
        }
    }

    tags
}

impl<'a> std::convert::TryFrom<&'a str> for Feature {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Feature, Error> {
        use parser::*;
        use pest::Parser;

        let mut pairs = FeatureParser::parse(Rule::main, &s)?;
        let pair = pairs.next().expect("pair to exist");
        let inner_pair = pair.into_inner().next().expect("feature to exist");

        Ok(Feature::from(inner_pair))
    }
}

#[derive(Debug)]
pub enum TryFromError {
    Parsing(Error),
    Reading(std::io::Error),
}

impl<'a> std::convert::TryFrom<&'a std::path::Path> for Feature {
    type Error = TryFromError;

    fn try_from(p: &'a std::path::Path) -> Result<Feature, TryFromError> {
        let s = std::fs::read_to_string(p).map_err(TryFromError::Reading)?;
        Feature::try_from(&*s).map_err(TryFromError::Parsing)
    }
}

impl StepType {
    pub fn new_with_context(s: &str, context: Option<StepType>) -> Self {
        match (s, context) {
            ("Given", _) => StepType::Given,
            ("When", _) => StepType::When,
            ("Then", _) => StepType::Then,
            ("And", Some(v)) => v,
            ("But", Some(v)) => v,
            _ => panic!("Invalid input: {:?}", s),
        }
    }
}

impl Step {
    fn from_rule_with_context(
        outer_rule: Pair<'_, parser::Rule>,
        context: Option<StepType>,
    ) -> Self {
        let mut builder = StepBuilder::default();

        for rule in outer_rule.into_inner() {
            match rule.as_rule() {
                parser::Rule::step_kw => {
                    let span = rule.as_span();
                    let raw_type = span.as_str();
                    let ty = StepType::new_with_context(raw_type, context);
                    builder.ty(ty);
                    builder.position(span.start_pos().line_col());
                    builder.raw_type(raw_type.to_string());
                }
                parser::Rule::step_body => {
                    let value = rule.as_span().as_str().to_string();
                    builder.value(value);
                }
                parser::Rule::docstring => {
                    let r = rule
                        .into_inner()
                        .next()
                        .expect("docstring value")
                        .as_span()
                        .as_str();
                    let r = textwrap::dedent(r);
                    let docstring = r
                        .trim_end()
                        .trim_matches(|c| c == '\r' || c == '\n')
                        .to_string();
                    builder.docstring(Some(docstring));
                }
                parser::Rule::datatable => {
                    let datatable = Table::from(rule);
                    builder.table(Some(datatable));
                }
                parser::Rule::space | parser::Rule::nl | parser::Rule::EOI => (),
                _ => panic!("unhandled rule for Step: {:?}", rule),
            }
        }

        builder.build().expect("step to be built")
    }

    fn vec_from_rule(rule: Pair<'_, parser::Rule>) -> Vec<Step> {
        let mut steps: Vec<Step> = vec![];

        for pair in rule.into_inner() {
            if pair.as_rule() == parser::Rule::step {
                let s = Step::from_rule_with_context(pair, steps.last().map(|x| x.ty));
                steps.push(s);
            }
        }

        steps
    }
}

impl<'a> From<Pair<'a, parser::Rule>> for Rule {
    fn from(rule: Pair<'a, parser::Rule>) -> Self {
        let mut builder = RuleBuilder::default();
        let mut scenarios = vec![];

        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::rule_name => {
                    let span = pair.as_span();
                    builder.name(span.as_str().to_string());
                    builder.position(span.start_pos().line_col());
                }
                parser::Rule::scenario => {
                    let scenario = Scenario::from(pair);
                    scenarios.push(scenario);
                }
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                }
                _ => {}
            }
        }

        builder
            .scenarios(scenarios)
            .build()
            .expect("scenario to be built")
    }
}

impl<'a> From<Pair<'a, parser::Rule>> for Background {
    fn from(rule: Pair<'a, parser::Rule>) -> Self {
        let pos = rule.as_span().start_pos().line_col();
        Background {
            steps: Step::vec_from_rule(rule),
            position: pos,
        }
    }
}

impl<'a> From<Pair<'a, parser::Rule>> for Feature {
    fn from(rule: Pair<'a, parser::Rule>) -> Self {
        let mut builder = FeatureBuilder::default();
        let mut scenarios = vec![];
        let mut rules = vec![];

        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::feature_kw => {
                    builder.position(pair.as_span().start_pos().line_col());
                }
                parser::Rule::feature_body => {
                    builder.name(pair.as_span().as_str().to_string());
                }
                parser::Rule::feature_description => {
                    let description = textwrap::dedent(pair.as_span().as_str());
                    if description == "" {
                        builder.description(None);
                    } else {
                        builder.description(Some(description));
                    }
                }
                parser::Rule::background => {
                    builder.background(Some(Background::from(pair)));
                }
                parser::Rule::scenario => {
                    let scenario = Scenario::from(pair);
                    scenarios.push(scenario);
                }
                parser::Rule::rule => {
                    let rule = Rule::from(pair);
                    rules.push(rule);
                }
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                }
                _ => {}
            }
        }

        builder
            .scenarios(scenarios)
            .rules(rules)
            .build()
            .expect("feature to be built")
    }
}

impl<'a> From<Pair<'a, parser::Rule>> for Table {
    fn from(rule: Pair<'a, parser::Rule>) -> Self {
        let mut builder = TableBuilder::default();
        let mut rows = vec![];

        builder.position(rule.as_span().start_pos().line_col());

        fn row_from_inner(inner: Pairs<'_, parser::Rule>) -> Vec<String> {
            let mut rows = vec![];
            for pair in inner {
                if pair.as_rule() == parser::Rule::table_field {
                    rows.push(pair.as_span().as_str().trim().to_string());
                }
            }
            rows
        }

        let mut header = None;
        for pair in rule.into_inner() {
            if pair.as_rule() == parser::Rule::table_row {
                if header.is_none() {
                    header = Some(row_from_inner(pair.into_inner()));
                } else {
                    rows.push(row_from_inner(pair.into_inner()));
                }
            }
        }
        if rows.is_empty() {
            rows.push(header.take().unwrap());
        }
        if let Some(h) = header {
            builder.header(h);
        } else {
            let mut h = Vec::new();
            h.resize(rows[0].len(), String::new());
            builder.header(h);
        }

        builder.rows(rows).build().expect("table to be build")
    }
}

impl<'a> From<Pair<'a, parser::Rule>> for Examples {
    fn from(rule: Pair<'a, parser::Rule>) -> Self {
        let mut builder = ExamplesBuilder::default();
        builder.position(rule.as_span().start_pos().line_col());

        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::datatable => {
                    let table = Table::from(pair);
                    builder.table(table);
                }
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                }
                _ => {}
            }
        }

        builder.build().expect("examples to be built")
    }
}

impl<'a> From<Pair<'a, parser::Rule>> for Scenario {
    fn from(rule: Pair<'a, parser::Rule>) -> Self {
        let mut builder = ScenarioBuilder::default();

        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::scenario_name => {
                    let span = pair.as_span();
                    builder.name(span.as_str().to_string());
                    builder.position(span.start_pos().line_col());
                }
                parser::Rule::scenario_steps => {
                    builder.steps(Step::vec_from_rule(pair));
                }
                parser::Rule::examples => {
                    let examples = Examples::from(pair);
                    builder.examples(Some(examples));
                }
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                }
                _ => {}
            }
        }

        builder.build().expect("scenario to be built")
    }
}

/// Re-exported `pest::Error` wrapped around the `Rule` type
pub type Error = pest::error::Error<parser::Rule>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_e2e() {
        let s = include_str!("./test.feature");
        let _f = Feature::try_from(s);
        // println!("{:#?}", _f);
    }
}
