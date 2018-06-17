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

#[derive(Debug, Clone, Builder)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>
}

#[derive(Debug, Clone, Builder)]
pub struct Step {
    pub ty: StepType,
    pub value: String,
    #[builder(default)]
    pub docstring: Option<String>,
    #[builder(default)]
    pub table: Option<Table>
}

#[derive(Debug, Clone, Builder)]
pub struct Background {
    pub steps: Vec<Step>
}

#[derive(Debug, Clone, Builder)]
pub struct Examples {
    pub table: Table,
    #[builder(default)]
    pub tags: Option<Vec<String>>
}

#[derive(Debug, Clone, Builder)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
    #[builder(default)]
    pub examples: Option<Examples>,
    #[builder(default)]
    pub tags: Option<Vec<String>>
}

#[derive(Debug, Clone, Builder)]
pub struct Feature {
    pub name: String,
    #[builder(default)]
    pub background: Option<Background>,
    pub scenarios: Vec<Scenario>,
    #[builder(default)]
    pub tags: Option<Vec<String>>
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

impl Step {
    pub fn from_rule_with_context<'a>(outer_rule: pest::iterators::Pair<'a, parser::Rule>, context: Option<StepType>) -> Self {
        let mut builder = StepBuilder::default();

        for rule in outer_rule.into_inner() {
            match rule.as_rule() {
                parser::Rule::step_kw => {
                    let ty = StepType::new_with_context(rule.clone().into_span().as_str(), context);
                    builder.ty(ty);
                },
                parser::Rule::step_body => {
                    let value = rule.clone().into_span().as_str().to_string();
                    builder.value(value);
                },
                parser::Rule::docstring => {
                    let docstring = rule.into_inner().next().expect("docstring value")
                        .into_span().as_str().trim().to_string();
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
        Background {
            steps: Step::vec_from_rule(rule)
        }
    }
}

impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Feature {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let mut builder = FeatureBuilder::default();
        let mut scenarios = vec![];
        
        for pair in rule.into_inner() {
            match pair.as_rule() {
                parser::Rule::feature_body => { builder.name(pair.clone().into_span().as_str().to_string()); },
                parser::Rule::background => { builder.background(Some(Background::from(pair))); },
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
        use pest::Parser;
        use parser::*;

        let mut pairs = FeatureParser::parse(Rule::main, &s)
            .unwrap_or_else(|e| panic!("{}", e));

        Feature::from(pairs.next()
            .expect("pair to exist")
            .into_inner()
            .next()
            .expect("feature to exist"))
    }
}

impl<'a> From<pest::iterators::Pair<'a, parser::Rule>> for Examples {
    fn from(rule: pest::iterators::Pair<'a, parser::Rule>) -> Self {
        let mut builder = ExamplesBuilder::default();
        
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
                parser::Rule::scenario_name => { builder.name(pair.clone().into_span().as_str().to_string()); },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e() {
        let s = include_str!("./test.feature");
        let _f = Feature::from(s);
        // println!("{:?}", f);
    }
}