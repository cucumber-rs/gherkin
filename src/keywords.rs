// Copyright (c) 2020-2021  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

#[derive(Debug, Clone)]
pub(crate) struct Keywords<'a> {
    pub feature: &'a [&'a str],
    pub background: &'a [&'a str],
    pub rule: &'a [&'a str],
    pub scenario: &'a [&'a str],
    pub scenario_outline: &'a [&'a str],
    pub examples: &'a [&'a str],
    pub given: &'a [&'a str],
    pub when: &'a [&'a str],
    pub then: &'a [&'a str],
    pub and: &'a [&'a str],
    pub but: &'a [&'a str],
}

impl<'a> Keywords<'a> {
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

    pub fn all(&self) -> Vec<&'a str> {
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

    pub fn exclude_in_description(&self) -> Vec<&'a str> {
        let mut v = [
            self.feature,
            self.background,
            self.rule,
            self.scenario,
            self.scenario_outline,
            self.examples,
        ]
        .iter()
        .flat_map(|s| s.iter().map(Deref::deref))
        .collect::<Vec<_>>();

        v.sort_unstable();

        v
    }
}

impl<'a> Default for Keywords<'a> {
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
