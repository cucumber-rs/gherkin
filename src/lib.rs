// Copyright (c) 2018  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate derive_builder;

mod parser;

#[derive(Debug, Clone, Copy)]
pub enum StepType {
    Given,
    When,
    Then
}

impl StepType {
    pub fn as_str(&self) -> &str {
        match self {
            StepType::Given => "Given",
            StepType::When => "When",
            StepType::Then => "Then"
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub position: (usize, usize)
}

#[derive(Debug, Clone, Builder)]
pub struct Step {
    pub ty: StepType,
    pub raw_type: String,
    pub value: String,
    #[builder(default)]
    pub docstring: Option<String>,
    #[builder(default)]
    pub table: Option<Table>,
    pub position: (usize, usize)
}

impl Step {
    pub fn docstring(&self) -> Option<&String> {
        match &self.docstring {
            Some(v) => Some(&v),
            None => None
        }
    }

    pub fn to_string(&self) -> String {
        format!("{} {}", &self.raw_type, &self.value)
    }
}

#[derive(Debug, Clone, Builder)]
pub struct Background {
    pub steps: Vec<Step>,
    pub position: (usize, usize)
}

#[derive(Debug, Clone, Builder)]
pub struct Examples {
    pub table: Table,
    #[builder(default)]
    pub tags: Option<Vec<String>>,
    pub position: (usize, usize)
}

#[derive(Debug, Clone, Builder)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
    #[builder(default)]
    pub examples: Option<Examples>,
    #[builder(default)]
    pub tags: Option<Vec<String>>,
    pub position: (usize, usize)
}

#[derive(Debug, Clone, Builder)]
pub struct Feature {
    pub name: String,
    #[builder(default)]
    pub background: Option<Background>,
    pub scenarios: Vec<Scenario>,
    #[builder(default)]
    pub tags: Option<Vec<String>>,
    pub position: (usize, usize)
}

fn parse_tags<'a>(outer_rule: pest::iterators::Pair<'a, parser::Rule>) -> Vec<String> {
    let mut tags = vec![];

    for rule in outer_rule.into_inner() {
        match rule.as_rule() {
            parser::Rule::tag => {
                let tag = rule.clone().into_span().as_str().to_string();
                tags.push(tag);
            },
            _ => {}
        }
    }

    tags
}

impl Feature {
    pub fn try_from<'a>(s: &'a str) -> Result<Feature, Error> {
        use pest::Parser;
        use parser::*;

        let mut pairs = FeatureParser::parse(Rule::main, &s)?;
        let pair = pairs.next().expect("pair to exist");
        let inner_pair = pair.into_inner().next().expect("feature to exist");

        Ok(Feature::from(inner_pair))
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
            _ => panic!("Invalid input: {:?}", s)
        }
    }
}

// https://github.com/bbqsrc/textwrap/blob/master/src/lib.rs#L900
// License: MIT
#[doc(hidden)]
fn dedent(s: &str) -> String {
    let mut prefix = String::new();

    // We first search for a non-empty line to find a prefix.
    for line in s.lines() {
        let whitespace = line.chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();
        // Check if the line had anything but whitespace
        if whitespace.len() < line.len() {
            prefix = whitespace;
            break;
        }
    }

    // Filter out all whitespace-only lines
    let lines = s.lines().filter(|l| !l.chars().all(|c| c.is_whitespace()));

    // We then continue looking through the remaining lines to
    // possibly shorten the prefix.
    for line in lines {
        let whitespace = line.chars()
            .zip(prefix.chars())
            .take_while(|&(a, b)| a == b)
            .map(|(_, b)| b)
            .collect::<String>();
        // Check if we have found a shorter prefix
        if whitespace.len() < prefix.len() {
            prefix = whitespace;
        }
    }

    // We now go over the lines a second time to build the result.
    let mut result = s.lines()
        .map(|line| {
            if line.starts_with(&prefix) && line.chars().any(|c| !c.is_whitespace()) {
                line.split_at(prefix.len()).1
            } else {
                ""
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    // Reappend missing newline if found
    if s.ends_with("\n") {
        result.push('\n');
    }

    result
}

impl Step {
    pub fn from_rule_with_context<'a>(outer_rule: pest::iterators::Pair<'a, parser::Rule>, context: Option<StepType>) -> Self {
        let mut builder = StepBuilder::default();

        for rule in outer_rule.into_inner() {
            match rule.as_rule() {
                parser::Rule::step_kw => {
                    let span = rule.clone().into_span();
                    let raw_type = span.as_str();
                    let ty = StepType::new_with_context(raw_type, context);
                    builder.ty(ty);
                    builder.position(span.start_pos().line_col());
                    builder.raw_type(raw_type.to_string());
                },
                parser::Rule::step_body => {
                    let value = rule.clone().into_span().as_str().to_string();
                    builder.value(value);
                },
                parser::Rule::docstring => {
                    let r = rule.into_inner()
                            .next().expect("docstring value")
                            .into_span().as_str();
                    let r = dedent(r);
                    let docstring = r
                        .trim_right()
                        .trim_matches(|c| c == '\r' || c == '\n')
                        .to_string();
                    builder.docstring(Some(docstring));
                }
                parser::Rule::datatable => {
                    let datatable = Table::from(rule);
                    builder.table(Some(datatable));
                }
                _ => panic!("unhandled rule for Step: {:?}", rule)
            }
        }
        
        builder.build().expect("step to be built")
    }

    pub fn vec_from_rule<'a>(rule: pest::iterators::Pair<'a, parser::Rule>) -> Vec<Step> {
        let mut steps: Vec<Step> = vec![];

        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::step => {
                    let s = Step::from_rule_with_context(pair, steps.last().map(|x| x.ty));
                    steps.push(s);
                },
                _ => {}
            }
        }

        steps
    }
}

impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Background {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let pos = rule.clone().into_span().start_pos().line_col();
        Background {
            steps: Step::vec_from_rule(rule),
            position: pos
        }
    }
}

impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Feature {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let mut builder = FeatureBuilder::default();
        let mut scenarios = vec![];
        
        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::feature_kw => {
                    builder.position(pair.clone().into_span().start_pos().line_col());
                },
                parser::Rule::feature_body => {
                    builder.name(pair.clone().into_span().as_str().to_string());
                },
                parser::Rule::background => {
                    builder.background(Some(Background::from(pair)));
                },
                parser::Rule::scenario => {
                    let scenario = Scenario::from(pair);
                    scenarios.push(scenario);
                },
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                },
                _ => {}
            }
        }

        builder
            .scenarios(scenarios)
            .build()
            .expect("feature to be built")
    }
}


impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Table {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let mut builder = TableBuilder::default();
        let mut rows = vec![];

        builder.position(rule.clone().into_span().start_pos().line_col());

        fn row_from_inner<'a>(inner: pest::iterators::Pairs<'a, parser::Rule>) -> Vec<String> {
            let mut rows = vec![];
            for pair in inner {
                match pair.as_rule() {
                    parser::Rule::table_field => {
                        rows.push(pair.clone().into_span().as_str().trim().to_string());
                    },
                    _ => {}
                }
            }
            rows
        }
        
        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::table_header => {
                    builder.header(row_from_inner(pair.into_inner()));
                 },
                parser::Rule::table_row => {
                    rows.push(row_from_inner(pair.into_inner()));
                }
                _ => {}
            }
        }

        builder
            .rows(rows)
            .build().expect("table to be build")
    }
}

impl<'a> From<&'a str> for Feature {
    fn from(s: &'a str) -> Self {
        Feature::try_from(s).unwrap()
    }
}

impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Examples {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let mut builder = ExamplesBuilder::default();
        builder.position(rule.clone().into_span().start_pos().line_col());
        
        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::datatable => {
                    let table = Table::from(pair);
                    builder.table(table);
                }
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                },
                _ => {}
            }
        }

        builder.build().expect("examples to be built")
    }
}

impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Scenario {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let mut builder = ScenarioBuilder::default();
        
        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::scenario_name => {
                    let span = pair.clone().into_span();
                    builder.name(span.as_str().to_string());
                    builder.position(span.start_pos().line_col());
                },
                parser::Rule::scenario_steps => { builder.steps(Step::vec_from_rule(pair)); }
                parser::Rule::examples => {
                    let examples = Examples::from(pair);
                    builder.examples(Some(examples));
                }
                parser::Rule::tags => {
                    let tags = parse_tags(pair);
                    builder.tags(Some(tags));
                },
                _ => {}
            }
        }

        builder.build().expect("scenario to be built")
    }
}

pub type Error<'a> = pest::Error<'a, parser::Rule>;

pub fn error_position<'a>(error: &Error<'a>) -> (usize, usize) {
    match error {
        pest::Error::ParsingError {
            pos,
            positives: _,
            negatives: _
        } => pos.line_col(),
        pest::Error::CustomErrorPos {
            pos,
            message: _
        } => pos.line_col(),
        _ => (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e() {
        let s = include_str!("./test.feature");
        let _f = Feature::from(s);
        println!("{:#?}", _f);
    }
}

