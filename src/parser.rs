// Copyright (c) 2020  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::RefCell;

use crate::tagexpr::TagOperation;
use crate::{Background, Feature, Rule, Scenario, Step, StepType, Table, Examples};

struct Keywords<'a> {
    feature: &'a [&'a str],
    background: &'a [&'a str],
    rule: &'a [&'a str],
    scenario: &'a [&'a str],
    scenario_outline: &'a [&'a str],
    examples: &'a [&'a str],
    given: &'a [&'a str],
    when: &'a [&'a str],
    then: &'a [&'a str],
    and: &'a [&'a str],
    but: &'a [&'a str],
}

impl<'a> Keywords<'a> {
    pub fn all(&self) -> Vec<&'a str> {
        let mut v = vec![];

        for x in [
            self.feature,
            self.background,
            self.rule,
            self.scenario,
            self.rule,
            self.scenario_outline,
            self.examples,
            self.given,
            self.when,
            self.then,
            self.and,
            self.but,
        ]
        .iter()
        {
            v.append(&mut x.to_vec());
        }

        v
    }
}

const DEFAULT_KEYWORDS: Keywords<'static> = Keywords {
    feature: &["Feature"],
    background: &["Background"],
    rule: &["Rule"],
    scenario: &["Scenario", "Example"],
    scenario_outline: &["Scenario Outline", "Scenario Template"],
    examples: &["Examples"],
    given: &["Given"],
    when: &["When"],
    then: &["Then"],
    and: &["*", "And"],
    but: &["But"],
};

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

const SV_KEYWORDS: Keywords<'static> = Keywords {
    feature: &["Egenskap"],
    background: &["Bakgrund"],
    rule: &["Regel"],
    scenario: &["Scenario", "Exempel"],
    scenario_outline: &["Abstrakt Scenario"],
    examples: &["Exempel"],
    given: &["Givet"],
    when: &["När"],
    then: &["Så"],
    and: &["*", "Och"],
    but: &["Men"],
};

pub struct GherkinEnv {
    keywords: RefCell<Keywords<'static>>,
    last_step: RefCell<Option<StepType>>,
    last_keyword: RefCell<Option<String>>,
}

impl GherkinEnv {
    pub fn set_language(&self, language: &str) -> Result<(), &'static str> {
        let keywords = match language {
            "formal" => FORMAL_SPEC_KEYWORDS,
            "sv" => SV_KEYWORDS,
            "en" => DEFAULT_KEYWORDS,
            _ => return Err("Error: requested language not supported"),
        };

        *self.keywords.borrow_mut() = keywords;

        Ok(())
    }

    fn keywords(&self) -> std::cell::Ref<Keywords<'static>> {
        self.keywords.borrow()
    }

    fn set_keyword(&self, kw: String) {
        *self.last_keyword.borrow_mut() = Some(kw);
    }

    fn clear_keyword(&self) {
        *self.last_keyword.borrow_mut() = None;
    }

    fn last_keyword(&self) -> std::cell::Ref<Option<String>> {
        self.last_keyword.borrow()
    }

    fn take_keyword(&self) -> String {
        self.last_keyword.borrow_mut().take().unwrap()
    }

    fn set_last_step(&self, ty: StepType) {
        *self.last_step.borrow_mut() = Some(ty);
    }

    fn clear_last_step(&self) {
        *self.last_step.borrow_mut() = None;
    }

    fn last_step(&self) -> Option<StepType> {
        self.last_step.borrow().clone()
    }
}

impl Default for GherkinEnv {
    fn default() -> Self {
        GherkinEnv {
            keywords: RefCell::new(DEFAULT_KEYWORDS),
            last_step: RefCell::new(None),
            last_keyword: RefCell::new(None),
        }
    }
}

peg::parser! { pub(crate) grammar gherkin_parser(env: &GherkinEnv) for str {

rule _() = quiet!{[' ']*}
rule __() = quiet!{[' ']+}
rule nl() = quiet!{"\n" comment()*}
rule eof() = quiet!{![_]}
rule nl_eof() = quiet!{(nl() / [' '])+ / eof()}
rule comment() = quiet!{"#" $((!nl()[_])*) nl()}

rule keyword1(list: &[&'static str]) -> &'static str
    = input:$([_]*<
        {list.iter().map(|x| x.len()).min().unwrap()},
        {list.iter().map(|x| x.len()).max().unwrap()}
    >) {?
        // println!("Input: {} {:?}", &input, &list);
        match list.iter().find(|x| input.starts_with(**x)) {
            Some(v) => {
                env.set_keyword(v.to_string());
                // println!("Found: {}", &v);
                Err("success")
            },
            None => {
                // println!("Unfound: {}", &input);
                env.clear_keyword();
                Err("hacks")
            }
        }
    }

rule keyword0(list: &[&'static str]) -> usize
    = keyword1(list)? {?
        match env.last_keyword().as_ref() {
            Some(v) => Ok(v.len()),
            None => Err("no match")
        }
    }

pub(crate) rule keyword(list: &[&'static str]) -> &'static str
    = len:keyword0(list) [_]*<{len}> {
        let kw = env.take_keyword();
        list.iter().find(|x| **x == &*kw).unwrap()
    }

rule language_directive() -> ()
    = "# language: " l:$(['a'..='z']+) _ nl() {?
        env.set_language(l)
    }

rule docstring() -> String
    = "\"\"\"" n:$((!"\"\"\""[_])*) "\"\"\"" nl_eof() {
        textwrap::dedent(n)
    }

rule table_cell() -> &'input str
    = "|" _ !(nl() / eof()) n:$((!"|"[_])*) { n }

pub(crate) rule table_row() -> Vec<String>
    = n:(table_cell() ** _) _ "|" _ nl_eof() {
        n.into_iter()
            .map(str::trim)
            .map(str::to_string)
            .collect()
    }

pub(crate) rule table0() -> Vec<Vec<String>>
    = _ d:(table_row() ++ _) {
        if d.is_empty() {
            d
        } else {
            let len = d[0].len();
            d.into_iter().map(|mut x| { x.truncate(len); x }).collect()
        }
    }

pub(crate) rule table() -> Table
    = pa:position!() t:table0() pb:position!() {
        Table::builder()
            .span((pa, pb))
            .rows(t)
            .build()
    }

pub(crate) rule step() -> Step
    = pa:position!() k:keyword((env.keywords().given)) __ n:$((!['\n'][_])+) pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {
        env.set_last_step(StepType::Given);
        Step::builder().ty(StepType::Given)
            .raw_type(k.to_string())
            .value(n.to_string())
            .table(t)
            .docstring(d)
            .span((pa, pb))
            .build()
    }
    / pa:position!() k:keyword((env.keywords().when)) __ n:$((!['\n'][_])+) pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {
        env.set_last_step(StepType::When);
        Step::builder().ty(StepType::When)
            .raw_type(k.to_string())
            .value(n.to_string())
            .table(t)
            .docstring(d)
            .span((pa, pb))
            .build()
    }
    / pa:position!() k:keyword((env.keywords().then)) __ n:$((!['\n'][_])+) pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {
        env.set_last_step(StepType::Then);
        Step::builder().ty(StepType::Then)
            .raw_type(k.to_string())
            .value(n.to_string())
            .table(t)
            .docstring(d)
            .span((pa, pb))
            .build()
    }
    / pa:position!() k:keyword((env.keywords().and)) __ n:$((!['\n'][_])+) pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {?
        match env.last_step() {
            Some(v) => {
                Ok(Step::builder().ty(v)
                    .raw_type(k.to_string())
                    .value(n.to_string())
                    .table(t)
                    .docstring(d)
                    .span((pa, pb))
                    .build())
            }
            None => {
                Err("given, when or then")
            }
        }
    }
    / pa:position!() k:keyword((env.keywords().but)) __ n:$((!['\n'][_])+) pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {?
        match env.last_step() {
            Some(v) => {
                Ok(Step::builder().ty(v)
                    .raw_type(k.to_string())
                    .value(n.to_string())
                    .table(t)
                    .docstring(d)
                    .span((pa, pb))
                    .build())
            }
            None => {
                Err("given, when or then")
            }
        }
    }

pub(crate) rule steps() -> Vec<Step>
    = s:(step() ** _) {
        env.clear_last_step();
        s
    }

rule background() -> Background
    = _ pa:position!()
      keyword((env.keywords().background)) ":" _ nl_eof()
      s:steps()?
      pb:position!()
    {
        Background::builder()
            .steps(s.unwrap_or(vec![]))
            .span((pa, pb))
            .build()
    }

rule any_directive() -> &'static str
    = k:keyword((&*env.keywords().all())) {
        // println!("Found directive: {}", &k);
        k
    }

rule description_line() -> &'input str
    = _ !"@" !any_directive() _ n:$((!['\n'][_])+) nl_eof() { n }

rule description() -> Option<String>
    = d:(description_line() ** _) {
        let d = d.join("\n");
        if d.trim() == "" {
            None
        } else {
            Some(d)
        }
    }

rule examples() -> Examples
    = _
      t:tags()
      _
      pa:position!()
      keyword((env.keywords().examples)) ":" _ nl_eof()
      tb:table()
      pb:position!()
    {
        Examples::builder()
            .tags(t)
            .table(tb)
            .span((pa, pb))
            .build()
    }

rule scenario() -> Scenario
    = _
      t:tags()
      _
      pa:position!()
      keyword((env.keywords().scenario)) ":" _ n:$((!['\n'][_])+) _ nl_eof()
      s:steps()?
      e:examples()?
      pb:position!()
    {
        Scenario::builder()
            .name(n.to_string())
            .tags(t)
            .steps(s.unwrap_or(vec![]))
            .examples(e)
            .span((pa, pb))
            .build()
    }
    / _
      t:tags()
      _
      pa:position!()
      keyword((env.keywords().scenario_outline)) ":" _ n:$((!['\n'][_])+) _ nl_eof()
      s:steps()?
      e:examples()?
      pb:position!()
    {
        Scenario::builder()
            .name(n.to_string())
            .tags(t)
            .steps(s.unwrap_or(vec![]))
            .examples(e)
            .span((pa, pb))
            .build()
    }

rule tag_char() -> &'input str
    = s:$([_]) {?
        let x = s.chars().next().unwrap();
        if x.is_alphanumeric() || x == '_' || x == '-' {
            Ok(s)
        } else {
            Err("tag character")
        }
    }

pub(crate) rule tag() -> String
    = "@" s:tag_char()+ { s.join("") }

pub(crate) rule tags() -> Vec<String>
    = t:(tag() ** ([' ']+)) _ nl() { t }
    / { vec![] }

rule rule_() -> Rule
    = _
      t:tags()
      _
      pa:position!()
      keyword((env.keywords().rule)) ":" _ n:$((!['\n'][_])+) _ nl_eof()
      s:scenarios()?
    //   e:examples()?
      pb:position!()
    {
        Rule::builder()
            .name(n.to_string())
            .tags(t)
            .scenarios(s.unwrap_or(vec![]))
            .span((pa, pb))
            .build()
    }

rule rules() -> Vec<Rule>
    = _ r:(rule_() ** _)? { r.unwrap_or(vec![]) }

pub(crate) rule scenarios() -> Vec<Scenario>
    = _ s:(scenario() ** _)? { s.unwrap_or(vec![]) }

pub rule feature() -> Feature
    = _ language_directive()? nl()*
      t:tags() nl()*
      pa:position!()
      keyword((env.keywords().feature)) ":" _ n:$((!['\n'][_])+) _ nl()+
      d:description()? nl()*
      b:background()? nl()*
      s:scenarios() nl()*
      r:rules() pb:position!()
      nl()*
    {
        Feature::builder()
            .tags(t)
            .name(n.to_string())
            .description(d.flatten())
            .background(b)
            .scenarios(s)
            .rules(r)
            .span((pa, pb))
            .build()
    }

pub rule tag_operation() -> TagOperation = precedence!{
    x:@ _ "and" _ y:(@) { TagOperation::And(Box::new(x), Box::new(y)) }
    x:@ _ "or" _ y:(@) { TagOperation::Or(Box::new(x), Box::new(y)) }
    "not" _ x:(@) { TagOperation::Not(Box::new(x)) }
    --
    t:tag() { TagOperation::Tag(t) }
    "(" t:tag_operation() ")" _ { t }
}

}}

#[cfg(test)]
mod test {
    use super::*;

    const FOO: &str = r#"# language: formal
@hot-stuff
Section: 4.2. The thing we care about
A description just jammed in here for no reason
@lol @a @rule     @with-spaces
Rule: All gubbins must be placed in the airlock

@bad_idea
Evidence: A gubbins in an airlock
    Given a gubbins
    """
    That's a gubbins
    and that is
    and so is that
    """
    When a gubbins is forced into this weird corner
    | a | b | c |
    | 1 | 2 | 3 |
    | 4 | 5 | 6 |
    Then a gubbins is proven to be in an airlock
"#;
    #[test]
    fn smoke() {
        let env = GherkinEnv::default();
        assert!(gherkin_parser::feature(FOO, &env).is_ok());
    }

    #[test]
    fn smoke2() {
        let env = GherkinEnv::default();
        let d = env!("CARGO_MANIFEST_DIR");
        let s = std::fs::read_to_string(format!("{}/tests/test.feature", d)).unwrap();
        assert!(gherkin_parser::feature(&s, &env).is_ok());
    }
}
