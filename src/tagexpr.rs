// Copyright (c) 2020  Brendan Molloy <brendan@bbqsrc.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ### Tag expressions
//! 
//! You can read about tag expressions in the [Cucumber documentation](https://cucumber.io/docs/cucumber/api/#tag-expressions).
//! 
//! This implements the parsing apparatus for these expressions so that other crates like [cucumber_rust](https://github.com/bbqsrc/cucumber-rust)
//! may take advantage of them.
//! 
//! #### Usage
//! 
//! ```
//! let op: TagOperation = "@a and @b".parse()?;
//! ```

use pest::iterators::Pairs;
use std::str::FromStr;

pub(crate) mod parser {
    #[derive(Parser)]
    #[grammar = "tagexpr.pest"]
    pub struct TagExprParser;

    // This ensures that when the .pest file is changed during dev, a new build will occur.
    #[cfg(debug_assertions)]
    const _GRAMMAR: &str = include_str!("./tagexpr.pest");
}

use parser::Rule;

/// Re-exported `pest::Error` wrapped around the `Rule` type
pub type Error = pest::error::Error<Rule>;

impl FromStr for TagOperation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use pest::Parser;

        let pairs = parser::TagExprParser::parse(Rule::main, s)?;
        Ok(TagOperation::new(pairs))
    }
}

/// A parsed tree of operations for Gherkin tags. 
#[derive(Debug, Clone)]
pub enum TagOperation {
    And(Box<TagOperation>, Box<TagOperation>),
    Or(Box<TagOperation>, Box<TagOperation>),
    Not(Box<TagOperation>),
    Tag(String),
}

#[derive(Debug)]
struct OpBuilder {
    op: Option<TagOperation>,
    rule: Rule,
    is_not: bool,
}

impl std::default::Default for OpBuilder {
    fn default() -> OpBuilder {
        OpBuilder {
            op: None,
            rule: Rule::op_and,
            is_not: false,
        }
    }
}

impl OpBuilder {
    #[inline]
    fn rule(&mut self, rule: Rule) -> &mut Self {
        match rule {
            Rule::op_and | Rule::op_or => {
                self.rule = rule;
            }
            Rule::op_not => {
                self.is_not = true;
            }
            _ => panic!("{:?}", rule),
        };
        self
    }

    #[inline]
    fn op(&mut self, mut other: TagOperation) -> &mut Self {
        if self.is_not {
            self.is_not = false;
            other = TagOperation::Not(Box::new(other));
        }

        if self.op.is_none() {
            self.op = Some(other);
            return self;
        }

        match self.rule {
            Rule::op_and => {
                self.op = Some(TagOperation::And(
                    Box::new(self.op.take().unwrap()),
                    Box::new(other),
                ));
            }
            Rule::op_or => {
                self.op = Some(TagOperation::Or(
                    Box::new(self.op.take().unwrap()),
                    Box::new(other),
                ));
            }
            r => {
                panic!("Unhandled op rule: {:?}", r);
            }
        }

        self
    }

    #[inline]
    fn build(&mut self) -> TagOperation {
        self.op.clone().take().expect("Requires at least one op")
    }
}

impl TagOperation {
    fn new(pairs: Pairs<'_, Rule>) -> Self {
        let mut builder = OpBuilder::default();

        pairs.for_each(|pair| {
            let rule = pair.as_rule();

            match rule {
                Rule::tag => {
                    builder.op(TagOperation::Tag(pair.as_span().as_str().to_string()));
                }
                Rule::braced => {
                    let op = TagOperation::new(pair.into_inner());
                    builder.op(op);
                }
                Rule::op_not | Rule::op_and | Rule::op_or => {
                    builder.rule(rule);
                }
                Rule::EOI => {}
                r => panic!("Invalid rule: {:?}", r),
            }
        });

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn parse_tag_expr() {
        let foo =
            TagExprParser::parse(Rule::main, "@foo and @bar").unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
    }
    #[test]
    fn parse_tag_expr2() {
        let foo =
            TagExprParser::parse(Rule::main, "@foo or @bar").unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
    }

    #[test]
    fn parse_tag_expr3() {
        let foo = TagExprParser::parse(Rule::main, "@foo and not @bar")
            .unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
    }

    #[test]
    fn parse_tag_expr4() {
        assert!(TagExprParser::parse(Rule::main, "@foo not @bar").is_err());
    }

    #[test]
    fn parse_tag_expr5() {
        let foo = TagExprParser::parse(Rule::main, "(not @foo) and not (@haha or @bar)")
            .unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
        // let rule = foo.next().unwrap();
        let x = TagOperation::new(foo);
        println!("{:?}", x);
    }

    #[test]
    fn parse_tag_expr6() {
        let foo = TagExprParser::parse(Rule::main, "not @foo and not @haha or @bar")
            .unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
        let x = TagOperation::new(foo);
        println!("{:?}", x);
    }

    #[test]
    fn parse_tag_expr7() {
        let foo = TagExprParser::parse(Rule::main, "not (@a or @b) and (@c or not @d)")
            .unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
        let x = TagOperation::new(foo);
        println!("{:?}", x);
    }

    #[test]
    fn parse_tag_expr8() {
        let foo = TagExprParser::parse(Rule::main, "@a or @b and @c or not @d")
            .unwrap_or_else(|e| panic!("{}", e));
        println!("{:#?}", foo);
        let x = TagOperation::new(foo);
        println!("{:?}", x);
    }
}
