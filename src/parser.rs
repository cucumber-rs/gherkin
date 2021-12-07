// Copyright (c) 2020-2021  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::RefCell;

use crate::{keywords::Keywords, tagexpr::TagOperation};
use crate::{Background, Examples, Feature, LineCol, Rule, Scenario, Span, Step, StepType, Table};

#[derive(Debug)]
pub struct GherkinEnv {
    keywords: RefCell<Keywords<'static>>,
    pub(crate) last_error: RefCell<Option<EnvError>>,
    pub(crate) fatal_error: RefCell<Option<EnvError>>,
    last_step: RefCell<Option<StepType>>,
    last_keyword: RefCell<Option<String>>,
    line_offsets: RefCell<Vec<usize>>,
}

#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("Requested language '{0}' is not supported.")]
    UnsupportedLanguage(String),

    #[error("Unknown keyword: '{0}'.")]
    UnknownKeyword(String),

    #[error("Inconsistent cell count")]
    InconsistentCellCount(Vec<Vec<String>>),
}

impl GherkinEnv {
    pub fn new(language: &str) -> Result<Self, EnvError> {
        let keywords = Keywords::get(language)
            .ok_or_else(|| EnvError::UnsupportedLanguage(language.into()))?;

        Ok(Self {
            keywords: RefCell::new(keywords),
            ..Default::default()
        })
    }

    pub fn set_language(&self, language: &str) -> Result<(), &'static str> {
        let keywords = Keywords::get(language).ok_or_else(|| {
            self.set_fatal_error(EnvError::UnsupportedLanguage(language.into()));
            "Unsupported language"
        })?;

        *self.keywords.borrow_mut() = keywords;

        Ok(())
    }

    fn assert_no_error(&self) -> Result<(), &'static str> {
        if self.fatal_error.borrow().is_some() {
            return Err("fatal error");
        }

        Ok(())
    }

    fn set_fatal_error(&self, error: EnvError) {
        if self.fatal_error.borrow().is_some() {
            return;
        }

        *self.fatal_error.borrow_mut() = Some(error);
    }

    fn set_last_error(&self, error: EnvError) {
        *self.last_error.borrow_mut() = Some(error);
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
        *self.last_step.borrow()
    }

    fn increment_nl(&self, offset: usize) {
        self.line_offsets.borrow_mut().push(offset);
    }

    fn position(&self, offset: usize) -> LineCol {
        let line_offsets = self.line_offsets.borrow();
        let line = line_offsets
            .iter()
            .position(|x| x > &offset)
            .unwrap_or_else(|| line_offsets.len());

        let col = offset - line_offsets[line - 1] + 1;

        LineCol { line, col }
    }
}

impl Default for GherkinEnv {
    fn default() -> Self {
        GherkinEnv {
            keywords: RefCell::new(Default::default()),
            last_error: RefCell::new(None),
            fatal_error: RefCell::new(None),
            last_step: RefCell::new(None),
            last_keyword: RefCell::new(None),
            line_offsets: RefCell::new(vec![0]),
        }
    }
}

peg::parser! { pub(crate) grammar gherkin_parser(env: &GherkinEnv) for str {

rule _() = quiet!{[' ' | '\t']*}
rule __() = quiet!{[' ' | '\t']+}

rule nl0() = quiet!{"\r"? "\n"}
rule nl() = quiet!{nl0() p:position!() comment()* {
    env.increment_nl(p);
}}
rule eof() = quiet!{![_]}
rule nl_eof() = quiet!{(nl() / [' ' | '\t'])+ / eof()}
rule comment() = quiet!{[' ' | '\t']* "#" $((!nl0()[_])*) nl_eof()}
rule not_nl() -> &'input str = n:$((!nl0()[_])+) { n }

rule keyword1(list: &[&'static str]) -> &'static str
    = input:$([_]*<
        {list.iter().map(|x| x.chars().count()).min().unwrap()},
        {list.iter().map(|x| x.chars().count()).max().unwrap()}
    >) {?
        // println!("Input: {}", &input);
        match list.iter().find(|x| input.starts_with(**x)) {
            Some(v) => {
                env.set_keyword((*v).to_string());
                // println!("Found: {}", &v);
                Err("success")
            },
            None => {
                // println!("Unfound: {}", &input);
                env.clear_keyword();
                env.set_last_error(EnvError::UnknownKeyword(input.into()));
                Err("unknown keyword")
            }
        }
    }

rule keyword0(list: &[&'static str]) -> usize
    = keyword1(list)? {?
        match env.last_keyword().as_ref() {
            Some(v) => Ok(v.chars().count()),
            None => Err("no match")
        }
    }

pub(crate) rule keyword(list: &[&'static str]) -> &'static str
    = comment()* len:keyword0(list) [_]*<{len}> {
        let kw = env.take_keyword();
        list.iter().find(|x| **x == &*kw).unwrap()
    }

rule language_directive() -> ()
    = "#" _ "language:" _ l:$(not_nl()+) _ nl() {?
        env.set_language(l)
    }

rule docstring() -> String
    = "\"\"\"" n:$((!"\"\"\""[_])*) "\"\"\"" nl_eof() {
        textwrap::dedent(n)
    }
    / "```" n:$((!"```"[_])*) "```" nl_eof() {
        textwrap::dedent(n)
    }

rule table_cell() -> &'input str
    = "|" _ !(nl0() / eof()) n:$((!"|"[_])*) { n }

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
    = pa:position!() t:table0() pb:position!() {?
        if !t.is_empty() && t.iter().skip(1).any(|x| x.len() != t[0].len()) {
            env.set_fatal_error(EnvError::InconsistentCellCount(t));
            Err("inconsistent table row sizes")
        } else {
            Ok(Table::builder()
                .span(Span { start: pa, end: pb })
                .position(env.position(pa))
                .rows(t)
                .build())
        }
    }

pub(crate) rule step() -> Step
    = comment()* pa:position!() k:keyword((env.keywords().given)) __ n:not_nl() pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {
        env.set_last_step(StepType::Given);
        Step::builder().ty(StepType::Given)
            .keyword(k.to_string())
            .value(n.to_string())
            .table(t)
            .docstring(d)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }
    / pa:position!() k:keyword((env.keywords().when)) __ n:not_nl() pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {
        env.set_last_step(StepType::When);
        Step::builder().ty(StepType::When)
            .keyword(k.to_string())
            .value(n.to_string())
            .table(t)
            .docstring(d)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }
    / pa:position!() k:keyword((env.keywords().then)) __ n:not_nl() pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {
        env.set_last_step(StepType::Then);
        Step::builder().ty(StepType::Then)
            .keyword(k.to_string())
            .value(n.to_string())
            .table(t)
            .docstring(d)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }
    / pa:position!() k:keyword((env.keywords().and)) __ n:not_nl() pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {?
        match env.last_step() {
            Some(v) => {
                Ok(Step::builder().ty(v)
                    .keyword(k.to_string())
                    .value(n.to_string())
                    .table(t)
                    .docstring(d)
                    .span(Span { start: pa, end: pb })
                    .position(env.position(pa))
                    .build())
            }
            None => {
                Err("given, when or then")
            }
        }
    }
    / pa:position!() k:keyword((env.keywords().but)) __ n:not_nl() pb:position!() _ nl_eof() _
      d:docstring()? t:table()?
    {?
        match env.last_step() {
            Some(v) => {
                Ok(Step::builder().ty(v)
                    .keyword(k.to_string())
                    .value(n.to_string())
                    .table(t)
                    .docstring(d)
                    .span(Span { start: pa, end: pb })
                    .position(env.position(pa))
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
    = comment()* _ pa:position!()
      k:keyword((env.keywords().background)) ":" _ nl_eof()
      s:steps()?
      pb:position!()
    {
        Background::builder()
            .keyword(k.into())
            .steps(s.unwrap_or_else(Vec::new))
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule any_directive() -> &'static str
    = k:keyword((&*env.keywords().all())) {
        k
    }

rule description_line() -> &'input str
    = _ !"@" !any_directive() _ n:not_nl() nl_eof() { n }

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
    = comment()*
      _
      t:tags()
      _
      pa:position!()
      k:keyword((env.keywords().examples)) ":" _ nl_eof()
      tb:table()
      pb:position!()
    {
        Examples::builder()
            .keyword(k.into())
            .tags(t)
            .table(tb)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule scenario() -> Scenario
    = comment()*
      _
      t:tags()
      _
      pa:position!()
      k:keyword((env.keywords().scenario)) ":" _ n:not_nl() _ nl_eof()
      s:steps()?
      e:examples()*
      pb:position!()
    {
        Scenario::builder()
            .keyword(k.into())
            .name(n.to_string())
            .tags(t)
            .steps(s.unwrap_or_else(Vec::new))
            .examples(e)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }
    / comment()*
      _
      t:tags()
      _
      pa:position!()
      k:keyword((env.keywords().scenario_outline)) ":" _ n:not_nl() _ nl_eof()
      s:steps()?
      e:examples()*
      pb:position!()
    {
        Scenario::builder()
            .keyword(k.into())
            .name(n.to_string())
            .tags(t)
            .steps(s.unwrap_or_else(Vec::new))
            .examples(e)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule tag_char() -> &'input str
    = s:$([_]) {?
        let x = s.chars().next().unwrap();
        if x.is_alphanumeric() || "_-.#".contains(x) {
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
      k:keyword((env.keywords().rule)) ":" _ n:not_nl() _ nl_eof()
      b:background()? nl()*
      s:scenarios()? nl()*
    //   e:examples()?
      pb:position!()
    {
        Rule::builder()
            .keyword(k.into())
            .name(n.to_string())
            .tags(t)
            .background(b)
            .scenarios(s.unwrap_or_else(Vec::new))
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule rules() -> Vec<Rule>
    = _ r:(rule_() ** _)? { r.unwrap_or_else(Vec::new) }

pub(crate) rule scenarios() -> Vec<Scenario>
    = _ s:(scenario() ** _)? { s.unwrap_or_else(Vec::new) }

pub(crate) rule feature() -> Feature
    = _ language_directive()?
      nl()*
      t:tags() nl()*
      pa:position!()
      k:keyword((env.keywords().feature)) ":" _ n:not_nl() _ nl()+
      d:description()? nl()*
      b:background()? nl()*
      s:scenarios() nl()*
      r:rules() pb:position!()
      nl()*
    {?
        if let Err(e) = env.assert_no_error() {
            Err(e)
        } else {
            Ok(Feature::builder()
                .keyword(k.into())
                .tags(t)
                .name(n.to_string())
                .description(d.flatten())
                .background(b)
                .scenarios(s)
                .rules(r)
                .span(Span { start: pa, end: pb })
                .position(env.position(pa))
                .build())
        }
    }

pub(crate) rule tag_operation() -> TagOperation = precedence!{
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

    const FOO: &str = "# language: formal\r\n
@hot-stuff
Section: 4.2. The thing we care about
A description just jammed in here for no reason
@lol @a @rule     @with-spaces
Rule: All gubbins must be placed in the airlock

@bad_idea
Evidence: A gubbins in an airlock
    Given a gubbins
    \"\"\"
    That's a gubbins
    and that is
    and so is that
    \"\"\"
    When a gubbins is forced into this weird corner
    | a | b | c |
    | 1 | 2 | 3 |
    | 4 | 5 | 6 |
    Then a gubbins is proven to be in an airlock
";

    // From Gherkin 6 documentation
    const RULE_WITH_BACKGROUND: &str = "
Feature: Overdue tasks
  Let users know when tasks are overdue, even when using other
  features of the app

  Rule: Users are notified about overdue tasks on first use of the day
    Background:
      Given I have overdue tasks

    Example: First use of the day
      Given I last used the app yesterday
      When I use the app
      Then I am notified about overdue tasks

    Example: Already used today
      Given I last used the app earlier today
      When I use the app
      Then I am not notified about overdue tasks
";

    const DOCSTRING: &str = r#"
Feature: Meow

Scenario: Meow
  Given meow
    """
    Docstring life!
    """
"#;

    const DOCSTRING2: &str = r#"
Feature: Meow

Scenario: Meow
  Given meow
    ```
    Docstring life!
    ```
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

    #[test]
    fn rule_with_background() {
        let env = GherkinEnv::default();
        assert!(
            gherkin_parser::feature(RULE_WITH_BACKGROUND, &env).is_ok(),
            "RULE_IN_BACKGROUND was not parsed correctly!"
        );
    }

    #[test]
    fn docstring() {
        let env = GherkinEnv::default();
        assert!(
            gherkin_parser::feature(DOCSTRING, &env).is_ok(),
            "DOCSTRING was not parsed correctly!"
        );
    }

    #[test]
    fn docstring2() {
        let env = GherkinEnv::default();
        assert!(
            gherkin_parser::feature(DOCSTRING2, &env).is_ok(),
            "DOCSTRING2 was not parsed correctly!"
        );
    }

    #[test]
    fn feature_name_and_scenario() {
        let env = GherkinEnv::default();
        let input = r#"Feature: Basic functionality
        here's some text
        really
Scenario: Hello
  Given a step"#;
        let feature = gherkin_parser::feature(input, &env).unwrap();
        println!("{:#?}", feature);
        assert_eq!(feature.scenarios.len(), 1);
        assert!(feature.description.is_some());
        assert!(feature.scenarios[0].steps[0].position.line != 0)
    }

    #[test]
    fn feature_only() {
        let env = GherkinEnv::default();
        let input = r#"Feature: Basic functionality
        "#;
        let feature = gherkin_parser::feature(input, &env).unwrap();
        println!("{:#?}", feature);
        assert_eq!(feature.scenarios.len(), 0);
        assert!(feature.description.is_none());
    }
}
