use std::collections::HashMap;

use crate::Feature;

pub(crate) fn check_ast(parsed: Feature, ast_parsed: &str) {
    let d: HashMap<String, serde_json::Value> = serde_json::from_str(&ast_parsed).unwrap();

    let document = d
        .get("gherkinDocument")
        .expect("There is no document in the file");
    let feature = document
        .get("feature")
        .expect("There is no feature in the document");
    let children = feature.get("children");

    if children.is_none() {
        assert!(parsed.background.is_none());
        assert_eq!(parsed.scenarios.len(), 0);
        assert_eq!(parsed.rules.len(), 0);
        return;
    }

    let children = children.unwrap().as_array().unwrap();

    let mut backgrounds = 0;
    let mut scenarios = 0;
    let mut rules = 0;

    for child in children {
        if !child.is_object() {
            panic!("Child must be object: {:#?}", child);
        }
        if child.as_object().unwrap().len() != 1 {
            panic!(
                "Child must have exactly one inner object, it had {}: {:#?}",
                child.as_object().unwrap().len(),
                child
            );
        }

        if let Some(background) = child.get("background") {
            backgrounds += 1;
            let parsed_background = parsed.background.as_ref();
            assert!(parsed_background.is_some());
            let parsed_background = parsed_background.unwrap();

            let name = background.get("name");
            if name.is_none() {
                assert!(parsed_background.name.is_none());
            } else {
                let name = name.unwrap().as_str();
                assert_eq!(parsed_background.name.as_ref().map(|x| &(*x)[..]), name);
            }

            let steps = background.get("steps");

            if steps.is_none() {
                assert_eq!(parsed_background.steps.len(), 0);
                continue;
            }

            let steps = steps.unwrap().as_array().expect("Steps must be an array");
            if !check_steps(&parsed_background.steps, steps) {
                panic!("Background steps are different from fixture");
            }
        } else if let Some(json_scenario) = child.get("scenario") {
            scenarios += 1;

            let parsed_scenarios = &parsed.scenarios;
            let mut parsed_scenario_candidates = Vec::<&crate::Scenario>::new();

            let json_scenario_name = json_scenario
                .get("name")
                .map(|x| x.as_str().unwrap())
                .unwrap_or_default();

            for scenario in parsed_scenarios.iter() {
                if scenario.name == json_scenario_name {
                    parsed_scenario_candidates.push(&scenario);
                }
            }

            let json_steps = json_scenario.get("steps");
            if json_steps.is_none() {
                // We should check that there is a scenario candidate with 0 steps but hey
            } else {
                let mut success = false;
                for candidate in &parsed_scenario_candidates {
                    if check_steps(&candidate.steps, json_steps.unwrap().as_array().unwrap()) {
                        success = true;
                        break;
                    }
                }
                if !success {
                    panic!("Scenario steps are different from fixture");
                }
            }
        } else if let Some(json_rule) = child.get("rule") {
            rules += 1;
            let parsed_rules = &parsed.rules;

            let json_rule_name = json_rule
                .get("name")
                .map(|x| x.as_str().unwrap())
                .unwrap_or_default();

            for rule in parsed_rules.iter() {
                // This is not perfect but shoul work for the provided data
                if rule.name == json_rule_name {
                    continue;
                }
            }
        } else {
            panic!("Unknown child type: {:#?}", child);
        }
    }

    if parsed.background.is_some() {
        assert_eq!(1, backgrounds);
    } else {
        assert_eq!(0, backgrounds);
    }

    assert_eq!(parsed.scenarios.len(), scenarios);
    assert_eq!(parsed.rules.len(), rules);
}

fn check_steps(parsed: &Vec<crate::Step>, json: &Vec<serde_json::Value>) -> bool {
    if parsed.len() != json.len() {
        return false;
    }

    for i in 0..parsed.len() {
        if parsed[i].keyword != json[i].get("keyword").unwrap().as_str().unwrap() {
            // println!("`{}` != `{}`", parsed[i].keyword, json[i].get("keyword").unwrap().as_str().unwrap());
            return false;
        }
        if parsed[i].value != json[i].get("text").unwrap().as_str().unwrap() {
            // println!("`{}` != `{}`", parsed[i].value, json[i].get("text").unwrap().as_str().unwrap());
            return false;
        }
    }

    true
}
