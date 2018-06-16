#[derive(Parser)]
#[grammar = "feature.pest"]
pub struct FeatureParser;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("./feature.pest"); // relative to this file

#[cfg(test)]
mod tests {
    use pest::Parser;
    use super::FeatureParser;
    use super::Rule;

    #[test]
    fn parse_step() {
        let _pairs = FeatureParser::parse(Rule::step, "Given you disappoint me\n").unwrap_or_else(|e| panic!("{}", e));
        
        // for pair in pairs {
        //     for inner_pair in pair.into_inner() {
        //         // let span = inner_pair.clone().into_span();
        //         // println!("{:?} {}", inner_pair.as_rule(), span.as_str());
        //     }
        // }
    }

    #[test]
    fn parse_scenario() {
        let s = r#"Scenario: You are walking through the forest
Given you fear Shia LeBoeuf
When you encounter Shia LeBoeuf
Then attempt to kill Shia LeBoeuf
"#;
        let pairs = FeatureParser::parse(Rule::scenario, &s).unwrap_or_else(|e| panic!("{}", e));
        
        for pair in pairs {
            // println!("<{:?}>", pair.as_rule());
            for inner_pair in pair.into_inner() {
                // println!("  <{:?}>", inner_pair.as_rule());

                for inin_pair in inner_pair.into_inner() {
                    // println!("    <{:?}>", inin_pair.as_rule());

                    for _ininin_pair in inin_pair.into_inner() {
                        // let span = ininin_pair.clone().into_span();
                        // println!("       <{:?}> {}", ininin_pair.as_rule(), span.as_str());
                    }
                }
            }
        }
    }

    #[test]
    fn parse_docstring() {
        let s = r#""""
This is a docstring
"""
"#;
        let _pairs = FeatureParser::parse(Rule::docstring, &s).unwrap_or_else(|e| panic!("{}", e));
    }

    #[test]
    fn parse_table_row() {
        let s = r#"| first | second | third |"#;
        let pairs = FeatureParser::parse(Rule::table_row, &s).unwrap_or_else(|e| panic!("{}", e)).next().unwrap().into_inner();
        
        let mut c = 0usize;
        for _pair in pairs {
            // println!("{:?}", pair.clone().into_span().as_str());
            c += 1;
        }
        assert!(c == 3, "{} != 3", c);
    }

    #[test]
    fn parse_datatable() {
        let s = r#"| first | second | third |
| a thingo | another thingo | final thingo |
| a thingo | another thingo | final thingo |
"#;
        let _pairs = FeatureParser::parse(Rule::datatable, &s).unwrap_or_else(|e| panic!("{}", e));
    }
}
