// Copyright (c) 2020-2026  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

/// The words that start each part of a feature file in one language.
///
/// Step keywords keep their trailing space where the language has one, such as
/// `"Given "`, and `"* "` appears among the step keywords of every language.
///
/// ```
/// let english = gherkin::Keywords::get("en").unwrap();
/// assert!(english.given.contains(&"Given "));
/// assert!(english.and.contains(&"And "));
///
/// let formal = gherkin::Keywords::get("formal").unwrap();
/// assert_eq!(formal.feature, ["Section"]);
///
/// assert!(gherkin::Keywords::get("no-such-language").is_none());
/// ```
#[derive(Debug, Clone)]
pub struct Keywords<'a> {
    /// Words that start a feature, such as `Feature`.
    pub feature: &'a [&'a str],
    /// Words that start a background, such as `Background`.
    pub background: &'a [&'a str],
    /// Words that start a rule, such as `Rule`.
    pub rule: &'a [&'a str],
    /// Words that start a scenario, such as `Scenario` or `Example`.
    pub scenario: &'a [&'a str],
    /// Words that start a scenario outline, such as `Scenario Outline`.
    pub scenario_outline: &'a [&'a str],
    /// Words that start a table of examples, such as `Examples`.
    pub examples: &'a [&'a str],
    /// Words that start a `Given` step.
    pub given: &'a [&'a str],
    /// Words that start a `When` step.
    pub when: &'a [&'a str],
    /// Words that start a `Then` step.
    pub then: &'a [&'a str],
    /// Words that start an `And` step, which continues the step before it.
    pub and: &'a [&'a str],
    /// Words that start a `But` step, which continues the step before it.
    pub but: &'a [&'a str],
}

impl<'a> Keywords<'a> {
    /// Returns the keywords of the language named in a `# language:` line, such as `en`,
    /// `fr` or `formal`, or `None` if the language isn't supported.
    pub fn get(key: &str) -> Option<Keywords<'a>> {
        let result = include!(concat!(env!("OUT_DIR"), "/match.gen.rs"));

        if let Some(result) = result {
            return Some(result);
        }

        Some(match key {
            "formal" => FORMAL_SPEC_KEYWORDS,
            _ => return None,
        })
    }

    pub(crate) fn all(&self) -> Vec<&'a str> {
        let mut v = [
            self.feature,
            self.background,
            self.rule,
            self.scenario,
            self.scenario_outline,
            self.examples,
            self.given,
            self.when,
            self.then,
            self.and,
            self.but,
        ]
        .iter()
        .flat_map(|s| s.iter().map(Deref::deref))
        .collect::<Vec<_>>();

        v.sort_unstable();

        v
    }

    pub(crate) fn excluded_feature(&'a self) -> Vec<&'a str> {
        [
            self.background,
            self.rule,
            self.scenario,
            self.scenario_outline,
        ]
        .concat()
    }

    pub(crate) fn excluded_rule(&'a self) -> Vec<&'a str> {
        [self.background, self.scenario, self.scenario_outline].concat()
    }

    pub(crate) fn excluded_background(&'a self) -> Vec<&'a str> {
        [
            self.scenario,
            self.scenario_outline,
            self.given,
            self.when,
            self.then,
            self.and,
            self.but,
        ]
        .concat()
    }

    pub(crate) fn excluded_scenario(&'a self) -> Vec<&'a str> {
        [
            self.scenario,
            self.scenario_outline,
            self.given,
            self.when,
            self.then,
            self.and,
            self.but,
        ]
        .concat()
    }

    pub(crate) fn excluded_scenario_outline(&'a self) -> Vec<&'a str> {
        [
            self.scenario,
            self.scenario_outline,
            self.given,
            self.when,
            self.then,
            self.and,
            self.but,
        ]
        .concat()
    }

    pub(crate) fn excluded_examples(&'a self) -> Vec<&'a str> {
        let mut r = [
            self.scenario,
            self.scenario_outline,
            self.given,
            self.when,
            self.then,
            self.and,
            self.but,
        ]
        .concat();
        r.push("|");
        r
    }
}

impl Default for Keywords<'_> {
    fn default() -> Self {
        EN
    }
}

const FORMAL_SPEC_KEYWORDS: Keywords<'static> = Keywords {
    feature: &["Section"],
    background: &["Context"],
    rule: &["Rule"],
    scenario: &["Proof", "Evidence"],
    scenario_outline: &["Demonstration"],
    examples: &["Examples"],
    given: &["Given"],
    when: &["When"],
    then: &["Then"],
    and: &["*", "And"],
    but: &["But"],
};

include!(concat!(env!("OUT_DIR"), "/keywords.gen.rs"));
