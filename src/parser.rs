// Copyright (c) 2020-2022  Brendan Molloy <brendan@bbqsrc.net>
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
    was_escaped: RefCell<bool>,
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
        let mut line_offsets = self.line_offsets.borrow_mut();
        if !line_offsets.contains(&offset) {
            line_offsets.push(offset);
        }
    }

    fn position(&self, offset: usize) -> LineCol {
        let line_offsets = self.line_offsets.borrow();
        let line = line_offsets
            .iter()
            .position(|x| x > &offset)
            .unwrap_or(line_offsets.len());

        let col = offset - line_offsets[line - 1] + 1;

        LineCol { line, col }
    }

    fn escaped(&self) -> bool {
        *self.was_escaped.borrow()
    }

    fn set_escaped(&self, v: bool) {
        *self.was_escaped.borrow_mut() = v;
    }
}

impl Default for GherkinEnv {
    fn default() -> Self {
        GherkinEnv {
            keywords: RefCell::new(Keywords::default()),
            last_error: RefCell::new(None),
            fatal_error: RefCell::new(None),
            last_step: RefCell::new(None),
            last_keyword: RefCell::new(None),
            line_offsets: RefCell::new(vec![0]),
            was_escaped: RefCell::new(false),
        }
    }
}

peg::parser! { pub(crate) grammar gherkin_parser(env: &GherkinEnv) for str {

rule _() = quiet!{[' ' | '\t']*}
rule __() = quiet!{[' ' | '\t' | '\r' | '\n']*}

rule nl0() = quiet!{"\r"? "\n"}
rule nl() = quiet!{nl0() p:position!() comment()* {
    env.increment_nl(p);
}}
rule eof() = quiet!{![_]}
rule nl_eof() = quiet!{(nl() / [' ' | '\t'])+ / eof()}
rule comment() = quiet!{[' ' | '\t']* "#" $((!nl0()[_])*) nl_eof()}
rule not_nl() -> &'input str = n:$((!nl0()[_])+) { n }

rule keyword1(list: &[&str]) -> &'input str
    = input:$([_]*<
        {list.iter().map(|x| x.chars().count()).min().unwrap()},
        {list.iter().map(|x| x.chars().count()).max().unwrap()}
    >) {?
        // println!("Input: {}", &input);
        // println!("Looking for: {}", list.join(","));
        if let Some(v) = list.iter().find(|x| input.starts_with(**x)) {
            env.set_keyword((*v).to_string());
            // println!("Found: {}", &v);
            Err("success")
        } else {
            // println!("Unfound: {}", &input);
            env.clear_keyword();
            env.set_last_error(EnvError::UnknownKeyword(input.into()));
            Err("unknown keyword")
        }
    }

rule keyword0(list: &[&str]) -> usize
    = keyword1(list)? {?
        match env.last_keyword().as_ref() {
            Some(v) => Ok(v.chars().count()),
            None => Err("no match")
        }
    }

pub(crate) rule keyword<'a>(list: &[&'a str]) -> &'a str
    = comment()* len:keyword0(list) [_]*<{len}> {
        let kw = env.take_keyword();
        <&[&str]>::clone(&list).iter().find(|x| **x == &*kw).unwrap()
    }

rule language_directive() -> ()
    = _ "#" _ "language" _ ":" _ l:$(not_nl()+) _ nl() {?
        env.set_language(l)
    }

rule docstring() -> String
    = "\"\"\"" n:$((!"\"\"\"" (nl() / [_]))*) "\"\"\"" nl_eof() {
        textwrap::dedent(n)
    }
    / "```" n:$((!"```"(nl() / [_]))*) "```" nl_eof() {
        textwrap::dedent(n)
    }

rule table_cell() -> &'input str
    = "|" _ !(nl0() / eof()) n:$((!("|" / nl0())[_])*) { n }

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
    = comment()* pa:position!() k:keyword((env.keywords().given)) _ n:not_nl() pb:position!() _ nl_eof() _
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
    / pa:position!() k:keyword((env.keywords().when)) _ n:not_nl() pb:position!() _ nl_eof() _
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
    / pa:position!() k:keyword((env.keywords().then)) _ n:not_nl() pb:position!() _ nl_eof() _
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
    / pa:position!() k:keyword((env.keywords().and)) _ n:not_nl() pb:position!() _ nl_eof() _
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
    / pa:position!() k:keyword((env.keywords().but)) _ n:not_nl() pb:position!() _ nl_eof() _
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
      k:keyword((env.keywords().background)) ":" _ n:not_nl()? nl_eof()
      d:description((&env.keywords().excluded_background()))? nl()*
      s:steps()?
      pb:position!()
    {
        Background::builder()
            .keyword(k.into())
            .name(n.map(str::to_string))
            .description(d.flatten())
            .steps(s.unwrap_or_default())
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule any_directive() -> &'static str
    = k:keyword((&*env.keywords().all())) {
        k
    }

rule description_line(excluded: &[&str]) -> &'input str
    = _
      !"@" !keyword((excluded))
      _ n:not_nl() nl_eof()
    {
        n
    }

rule description(excluded: &[&str]) -> Option<String>
    = d:(description_line(excluded) ** _) {
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
      k:keyword((env.keywords().examples)) ":" _ n:not_nl()? nl_eof()
      d:description((&env.keywords().excluded_examples()))? nl()*
      tb:table()?
      pb:position!()
    {
        Examples::builder()
            .keyword(k.into())
            .name(n.map(str::to_owned))
            .description(d.flatten())
            .tags(t)
            .table(tb.unwrap_or_default())
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
      k:keyword((env.keywords().scenario)) ":" _ n:not_nl()? _ nl_eof()
      d:description((&env.keywords().excluded_scenario()))? nl()*
      s:steps()?
      e:examples()*
      pb:position!()
    {
        Scenario::builder()
            .keyword(k.into())
            .name(n.unwrap_or_default().to_string())
            .description(d.flatten())
            .tags(t)
            .steps(s.unwrap_or_default())
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
      k:keyword((env.keywords().scenario_outline)) ":" _ n:not_nl()? _ nl_eof()
      d:description((&env.keywords().excluded_scenario_outline()))? nl()*
      s:steps()?
      e:examples()*
      pb:position!()
    {
        Scenario::builder()
            .keyword(k.into())
            .name(n.unwrap_or_default().to_string())
            .description(d.flatten())
            .tags(t)
            .steps(s.unwrap_or_default())
            .examples(e)
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule tag_char() -> &'input str
    = s:$([_]) {?
        let x = s.chars().next().unwrap();
        if !x.is_whitespace() && x != '@' {
            Ok(s)
        } else {
            Err("tag character")
        }
    }

pub(crate) rule tag() -> String
    = "@" s:tag_char()+ { s.join("") }

rule tag_in_expr_char() -> Option<&'input str>
    = s:$([_]) {?
        let x = s.chars().next().unwrap();
        if !env.escaped() && x == '\\' {
            env.set_escaped(true);
            Ok(None)
        } else if env.escaped() {
            env.set_escaped(false);
            if "\\() ".contains(x) {
                Ok(Some(s))
            } else {
                Err("escaped non-reserved char")
            }
        } else if !x.is_whitespace() && !"@()\\".contains(x) {
            Ok(Some(s))
        } else {
            Err("tag character")
        }
    }

pub(crate) rule tag_in_expr() -> String
    = "@" s:tag_in_expr_char()+ {?
        if env.escaped() {
            env.set_escaped(false);
            Err("escaped end of line")
        } else {
            Ok(s.into_iter().flatten().collect())
        }
    }

pub(crate) rule tags() -> Vec<String>
    = t:(tag() ** __) _ nl()* { t }
    / { vec![] }

rule rule_() -> Rule
    = _
      t:tags()
      _
      pa:position!()
      k:keyword((env.keywords().rule)) ":" _ n:not_nl()? _ nl_eof()
      d:description((&env.keywords().excluded_rule()))? nl()*
      b:background()? nl()*
      s:scenarios()? nl()*
    //   e:examples()?
      pb:position!()
    {
        Rule::builder()
            .keyword(k.into())
            .name(n.unwrap_or_default().to_string())
            .description(d.flatten())
            .tags(t)
            .background(b)
            .scenarios(s.unwrap_or_default())
            .span(Span { start: pa, end: pb })
            .position(env.position(pa))
            .build()
    }

rule rules() -> Vec<Rule>
    = _ r:(rule_() ** _)? { r.unwrap_or_default() }

pub(crate) rule scenarios() -> Vec<Scenario>
    = _ s:(scenario() ** _)? { s.unwrap_or_default() }

pub(crate) rule feature() -> Feature
    = _ language_directive()?
      nl()*
      t:tags()
      pa:position!()
      k:keyword((env.keywords().feature)) ":" _ n:not_nl()? _ nl_eof()
      d:description((&env.keywords().excluded_feature()))? nl()*
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
                .name(n.unwrap_or_default().to_string())
                .description(d.flatten())
                .background(b)
                .scenarios(s)
                .rules(r)
                .span(Span { start: pa, end: pb })
                .position(env.position(pa))
                .build())
        }
    }

pub(crate) rule features() -> Vec<Feature>
    = __ comment()* f:(feature() ** __ )? __ comment()* { f.unwrap_or_default() }

pub(crate) rule tag_operation() -> TagOperation = precedence!{
    x:@ _ "and" _ y:(@) { TagOperation::And(Box::new(x), Box::new(y)) }
    x:@ _ "or" _ y:(@) { TagOperation::Or(Box::new(x), Box::new(y)) }
    "not" _ x:(@) { TagOperation::Not(Box::new(x)) }
    --
    t:tag_in_expr() { TagOperation::Tag(t) }
    "(" t:tag_operation() ")" _ { t }
}

}}

#[cfg(test)]
mod test {
    use crate::ast_checker;

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
        assert!(feature.scenarios[0].steps[0].position.line != 0);
    }

    #[test]
    fn correct_line_numbers() {
        let env = GherkinEnv::default();
        let input = r#"
# language: en
Feature: Basic functionality
        here's some text
     really
@tag
Scenario: Hello
  Given a step
  Then a step
@tag
Scenario: Hello
  Given a step

  And more

# comment
Rule: rule
    @tag
    Scenario Outline: Hello
        Given <step>
        """
        Doc String
        """

    Examples:
        | step |
        | 1    |
        | 2    |


    @tag
Rule: rule
    #comment
    Scenario: Hello
        Given a step
"#;
        let feature = gherkin_parser::feature(input, &env).unwrap();
        assert_eq!(feature.scenarios.len(), 2);
        assert!(feature.description.is_some());
        assert_eq!(feature.position.line, 3);
        assert_eq!(feature.scenarios[0].position.line, 7);
        assert_eq!(feature.scenarios[0].steps[0].position.line, 8);
        assert_eq!(feature.scenarios[0].steps[1].position.line, 9);
        assert_eq!(feature.scenarios[1].position.line, 11);
        assert_eq!(feature.scenarios[1].steps[0].position.line, 12);
        assert_eq!(feature.scenarios[1].steps[1].position.line, 14);
        assert_eq!(feature.rules[0].position.line, 17);
        assert_eq!(feature.rules[0].position.line, 17);
        assert_eq!(feature.rules[0].scenarios[0].position.line, 19);
        assert_eq!(feature.rules[0].scenarios[0].steps[0].position.line, 20);
        assert_eq!(feature.rules[0].scenarios[0].examples[0].position.line, 25);
        assert_eq!(
            feature.rules[0].scenarios[0].examples[0]
                .table
                .position
                .line,
            26,
        );
        assert_eq!(
            feature.rules[0].scenarios[0].examples[0].table.rows.len(),
            3,
        );
        assert_eq!(feature.rules[1].position.line, 32);
        assert_eq!(feature.rules[1].scenarios[0].position.line, 34);
        assert_eq!(feature.rules[1].scenarios[0].steps[0].position.line, 35);
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

    #[test]
    fn empty_feature() {
        let env = GherkinEnv::default();
        let input = " \n\t  \t\n\n ";
        let feature = gherkin_parser::feature(input, &env);
        assert!(feature.is_err());
    }

    #[test]
    fn one_feature() {
        let env = GherkinEnv::default();
        let input = r#"Feature: Basic functionality
        "#;
        let features = gherkin_parser::features(input, &env).unwrap();
        assert_eq!(features.len(), 1);
    }

    #[test]
    fn no_feature() {
        let env = GherkinEnv::default();
        let input = " \n\t  \t\n\n ";
        let features = gherkin_parser::features(input, &env).unwrap();
        assert_eq!(features.len(), 0);
    }

    #[test]
    fn multiple_features() {
        let env = GherkinEnv::default();
        let input = r#"Feature: Basic functionality
        here's some text
        really
Scenario: Hello
  Given a step

Feature: Another"#;

        // let features = gherkin_parser::features(input, &env);
        // println!("{:?}", features.unwrap_err());
        // println!("{:?}", env.last_error);
        // panic!();

        let features = gherkin_parser::features(input, &env).unwrap();
        assert_eq!(features.len(), 2);
    }

    #[test]
    fn fixture() {
        // We cannot handle missing features very well yet
        let skip = ["empty.feature", "incomplete_feature_3.feature"];
        let mut failed = 0;

        let d = env!("CARGO_MANIFEST_DIR");
        let files = std::fs::read_dir(format!("{}/tests/fixtures/data/good/", d)).unwrap();
        for file in files {
            let file = file.unwrap();
            let filename = file.file_name();
            let filename = filename.to_str().unwrap();
            if filename.ends_with(".feature") {
                if skip.contains(&filename) {
                    continue;
                }
                let res = std::panic::catch_unwind(|| {
                    let env = GherkinEnv::default();
                    let input = std::fs::read_to_string(format!(
                        "{}/tests/fixtures/data/good/{}",
                        d, filename
                    ))
                    .unwrap();
                    let feature = gherkin_parser::feature(&input, &env).unwrap();
                    let fixture = std::fs::read_to_string(format!(
                        "{}/tests/fixtures/data/good/{}.ast.ndjson",
                        d, filename
                    ))
                    .unwrap();
                    println!("{:#?}", feature);
                    ast_checker::check_ast(feature, &fixture);
                });
                if res.is_err() {
                    failed += 1;
                    println!("{}", filename);
                }
            }
        }

        if failed != 0 {
            panic!("{} fixtures have failed", failed);
        }
    }
}
