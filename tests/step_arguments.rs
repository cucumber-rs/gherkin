//! Focused behavior acceptance for Gherkin 43 step argument ordering.
use cucumber::{given, then, when, World as _};

#[derive(Debug, Default, cucumber::World)]
struct ArgumentWorld {
    source: String,
    table_parsed: Option<Result<gherkin::Feature, gherkin::ParseFileError>>,
    parsed: Option<Result<gherkin::Feature, gherkin::ParseError>>,
    dialect_cases: Vec<serde_json::Value>,
    dialect_results: Vec<Result<gherkin::Feature, gherkin::ParseError>>,
    languages: Vec<String>,
    availability: Vec<(bool, bool, bool, bool)>,
    formal_availability: (bool, bool),
    official_ast: Option<serde_json::Value>,
}

#[given(expr = "a feature whose {string} step has {string} arguments")]
fn feature(world: &mut ArgumentWorld, keyword: String, arguments: String) {
    let mut source = format!("Feature: Shared arguments\n  Scenario: Preserve values\n    {keyword} a payload\n");
    for argument in arguments.split('-') {
        match argument {
            "doc" => source.push_str("      \"\"\"\n      reusable payload\n      \"\"\"\n"),
            "table" => source.push_str("      | name | value |\n      | input | a\\|b |\n"),
            "none" => {},
            _ => panic!("Unknown argument kind: {}", argument),
        }
    }
    source.push_str("    Then a separate step\n");
    world.source = source;
}

#[when("the feature is parsed")]
fn parse(world: &mut ArgumentWorld) {
    world.parsed = Some(gherkin::Feature::parse(&world.source, Default::default()));
}

fn payload(world: &ArgumentWorld) -> &gherkin::Step {
    let feature = world.parsed.as_ref().expect("parser was invoked")
        .as_ref().expect("feature should parse");
    &feature.scenarios[0].steps[0]
}

#[then("parsing succeeds with both argument values on the same step")]
fn both(world: &mut ArgumentWorld) {
    let step = payload(world);
    assert_eq!(step.value, "a payload");
    assert_eq!(step.docstring.as_deref(), Some("\nreusable payload\n"));
    assert_eq!(step.table.as_ref().expect("table retained").rows,
        vec![vec!["name", "value"], vec!["input", "a|b"]]);
}

#[then(expr = "parsing succeeds with {string} arguments")]
fn fewer(world: &mut ArgumentWorld, arguments: String) {
    let step = payload(world);
    assert_eq!(step.docstring.is_some(), arguments == "doc");
    assert_eq!(step.table.is_some(), arguments == "table");
    if arguments == "doc" {
        assert_eq!(step.docstring.as_deref(), Some("\nreusable payload\n"));
    }
    if arguments == "table" {
        assert_eq!(step.table.as_ref().unwrap().rows,
            vec![vec!["name", "value"], vec!["input", "a|b"]]);
    }
}

#[then("the following step remains separate")]
fn next_step(world: &mut ArgumentWorld) {
    let feature = world.parsed.as_ref().unwrap().as_ref().unwrap();
    let steps = &feature.scenarios[0].steps;
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[1].value, "a separate step");
    assert!(steps[1].docstring.is_none());
    assert!(steps[1].table.is_none());
}

#[then("parsing fails")]
fn rejected(world: &mut ArgumentWorld) {
    assert!(world.parsed.as_ref().expect("parser was invoked").is_err(),
        "Repeated arguments must not be silently accepted: {}", world.source);
}

#[given(expr = "a feature with the {string} table cell encoding")]
fn table_encoding(world: &mut ArgumentWorld, encoding: String) {
    // Construct source here so the outer harness's registry parser never needs
    // to parse the table escapes that the changed local parser is accepting.
    let encoded = match encoding.as_str() {
        "unknown escape" => r"\o\q",
        "trailing slash" => r"tail\",
        "escaped pipe" => r"a\|b",
        "escaped slash" => r"a\\b",
        "escaped newline" => r"a\nb",
        "mixed escapes" => r"\o\nø\\\|tail\",
        _ => panic!("Unknown encoding: {}", encoding),
    };
    world.source = format!(
        "Feature: Literal table content\n  Scenario: Preserve values\n    Given a payload\n      | value |\n      | {} |\n    Then a separate step\n",
        encoded,
    );
}

#[then(expr = "the {string} table cell value is preserved exactly")]
fn table_value(world: &mut ArgumentWorld, encoding: String) {
    let expected = match encoding.as_str() {
        "unknown escape" => "\\o\\q",
        "trailing slash" => "tail\\",
        "escaped pipe" => "a|b",
        "escaped slash" => "a\\b",
        "escaped newline" => "a\nb",
        "mixed escapes" => "\\o\nø\\|tail\\",
        _ => panic!("Unknown encoding: {}", encoding),
    };
    let step = payload(world);
    assert_eq!(step.value, "a payload");
    assert_eq!(step.table.as_ref().expect("table retained").rows,
        vec![vec!["value"], vec![expected]]);
    assert!(step.docstring.is_none());
}

#[given(expr = "the published Gherkin 43 vocabulary cases for {string}")]
fn published_vocabulary(world: &mut ArgumentWorld, language: String) {
    let cases: Vec<serde_json::Value> = serde_json::from_str(
        include_str!("fixtures/gherkin43-dialect-cases.json")
    ).unwrap();
    world.dialect_cases = cases.into_iter()
        .filter(|case| case["language"].as_str() == Some(language.as_str()))
        .collect();
    assert!(!world.dialect_cases.is_empty(), "No published cases for {}", language);
}

#[when("those features are parsed with the Gherkin 43 catalogue")]
fn parse_published_vocabulary(world: &mut ArgumentWorld) {
    world.dialect_results = world.dialect_cases.iter().map(|case| {
        // Starting in English deliberately exercises # language switching while
        // retaining the explicitly selected catalogue, including new dialects.
        gherkin::Feature::parse(case["source"].as_str().unwrap(),
            gherkin::GherkinEnv::new_latest("en").unwrap())
    }).collect();
}

#[then("every published keyword retains its intended structure and text")]
fn assert_published_vocabulary(world: &mut ArgumentWorld) {
    assert_eq!(world.dialect_results.len(), world.dialect_cases.len());
    for (case, result) in world.dialect_cases.iter().zip(&world.dialect_results) {
        let feature = result.as_ref().unwrap_or_else(|error| panic!(
            "Official vocabulary source failed: {}\n{:?}", case["source"], error));
        let kind = case["kind"].as_str().unwrap();
        let keyword = case["keyword"].as_str().unwrap();
        assert_eq!(feature.name, "Portable accounts");
        let scenario = if kind == "rule" {
            assert_eq!(feature.rules.len(), 1, "{}", case["source"]);
            assert!(feature.scenarios.is_empty());
            assert_eq!(feature.rules[0].keyword, keyword);
            assert_eq!(feature.rules[0].name, "Account policy");
            assert_eq!(feature.rules[0].scenarios.len(), 1);
            &feature.rules[0].scenarios[0]
        } else {
            assert!(feature.rules.is_empty());
            assert_eq!(feature.scenarios.len(), 1, "{}", case["source"]);
            &feature.scenarios[0]
        };
        assert_eq!(scenario.name, "Account state");
        assert_eq!(scenario.steps.last().unwrap().value, "account remains", "{}", case["source"]);
        match kind {
            "feature" => assert_eq!(feature.keyword, keyword),
            "background" => {
                let background = feature.background.as_ref().unwrap();
                assert_eq!(background.keyword, keyword);
                assert_eq!(background.name, "Shared context");
                assert_eq!(background.steps.len(), 1);
                assert_eq!(background.steps[0].value, "account exists");
            }
            "rule" => {},
            "scenario" | "scenarioOutline" => assert_eq!(scenario.keyword, keyword),
            "examples" => assert_eq!(scenario.examples[0].keyword, keyword),
            "given" | "when" | "then" | "and" | "but" => {
                let step = scenario.steps.last().unwrap();
                assert_eq!(step.keyword, keyword, "{}", case["source"]);
                let expected_type = match kind {
                    "when" => gherkin::StepType::When,
                    "then" => gherkin::StepType::Then,
                    _ => gherkin::StepType::Given,
                };
                assert_eq!(step.ty, expected_type, "{}", case["source"]);
                assert_eq!(scenario.steps.len(), if matches!(kind, "and" | "but") { 2 } else { 1 });
            }
            _ => panic!("Unexpected keyword kind: {}", kind),
        }
        if matches!(kind, "scenarioOutline" | "examples") {
            assert_eq!(scenario.examples.len(), 1);
            assert_eq!(scenario.examples[0].table.as_ref().unwrap().rows,
                vec![vec!["value"], vec!["ready"]]);
        }
    }
}

#[given("a feature using the legacy Persian feature alias")]
fn legacy_persian(world: &mut ArgumentWorld) {
    let fixture: serde_json::Value = serde_json::from_str(
        include_str!("fixtures/gherkin43-legacy-persian.json")
    ).unwrap();
    world.source = fixture["source"].as_str().unwrap().to_owned();
}

#[when("it is parsed with legacy and Gherkin 43 catalogue policies")]
fn parse_catalogue_policies(world: &mut ArgumentWorld) {
    world.dialect_results = vec![
        gherkin::Feature::parse(&world.source, Default::default()),
        gherkin::Feature::parse(&world.source, gherkin::GherkinEnv::new("fa").unwrap()),
        gherkin::Feature::parse(&world.source, gherkin::GherkinEnv::new_latest("en").unwrap()),
    ];
}

#[then("legacy default and explicit parsing preserve the feature")]
fn legacy_retained(world: &mut ArgumentWorld) {
    assert_eq!(world.dialect_results.len(), 3);
    for result in &world.dialect_results[..2] {
        let feature = result.as_ref().expect("legacy Persian alias remains supported");
        assert_eq!(feature.keyword, "وِیژگی");
        assert_eq!(feature.name, "Legacy account");
        assert_eq!(feature.scenarios.len(), 1);
        assert_eq!(feature.scenarios[0].steps[0].value, "account remains");
    }
}

#[then("the Gherkin 43 policy rejects the obsolete feature alias")]
fn official_alias_rejected(world: &mut ArgumentWorld) {
    assert!(world.dialect_results[2].is_err(), "latest headers must not silently switch to legacy catalogue");
}

#[given("the new Amharic Belarusian and Malayalam dialects")]
fn new_dialects(world: &mut ArgumentWorld) {
    world.languages = vec!["amh".into(), "be".into(), "ml".into()];
}

#[when("dialect availability is requested for both catalogue policies")]
fn dialect_availability(world: &mut ArgumentWorld) {
    world.availability = world.languages.iter().map(|language| (
        gherkin::is_language_supported(language),
        gherkin::is_latest_language_supported(language),
        gherkin::GherkinEnv::new(language).is_ok(),
        gherkin::GherkinEnv::new_latest(language).is_ok(),
    )).collect();
    world.formal_availability = (
        gherkin::is_language_supported("formal"),
        gherkin::is_latest_language_supported("formal"),
    );
}

#[then("only the Gherkin 43 policy reports those new dialects")]
fn new_dialects_opt_in(world: &mut ArgumentWorld) {
    assert_eq!(world.availability, vec![(false, true, false, true); 3]);
    assert_eq!(world.formal_availability, (true, false));
}

#[given(expr = "the official Gherkin 43 doc-string fixture {string}")]
fn official_docstring_fixture(world: &mut ArgumentWorld, fixture: String) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gherkin43-docstrings");
    world.source = std::fs::read_to_string(root.join(&fixture)).unwrap();
    if fixture.contains(".crlf.") {
        // The upstream Git blob is LF even under its .crlf filename. Exercise
        // actual CRLF input while retaining the unmodified official content oracle.
        world.source = world.source.replace("\r\n", "\n").replace('\n', "\r\n");
    }
    world.official_ast = Some(serde_json::from_str(
        &std::fs::read_to_string(root.join(format!("{}.ast.ndjson", fixture))).unwrap()
    ).unwrap());
}

#[when("it is parsed with official doc-string content semantics")]
fn parse_official_docstrings(world: &mut ArgumentWorld) {
    let env = gherkin::GherkinEnv::new("en").unwrap()
        .with_docstring_semantics(gherkin::DocStringSemantics::Gherkin43);
    world.parsed = Some(gherkin::Feature::parse(&world.source, env));
}

#[then("every native doc-string value equals the external official AST content")]
fn official_docstring_content_matches(world: &mut ArgumentWorld) {
    let feature = world.parsed.as_ref().unwrap().as_ref()
        .expect("official doc-string fixture must parse");
    let ast = world.official_ast.as_ref().unwrap();
    let expected = ast["gherkinDocument"]["feature"]["children"].as_array().unwrap();
    assert_eq!(feature.scenarios.len(), expected.len());
    let mut docstrings = 0;
    for (scenario, expected) in feature.scenarios.iter().zip(expected) {
        let expected = &expected["scenario"];
        assert_eq!(scenario.name, expected["name"].as_str().unwrap());
        let steps = expected["steps"].as_array().unwrap();
        assert_eq!(scenario.steps.len(), steps.len());
        for (step, expected) in scenario.steps.iter().zip(steps) {
            assert_eq!(step.value, expected["text"].as_str().unwrap());
            let content = expected["docString"]["content"].as_str();
            assert_eq!(step.docstring.as_deref(), content, "wrong doc-string value for {}", step.value);
            if content.is_some() { docstrings += 1; }
            if let Some(rows) = expected["dataTable"]["rows"].as_array() {
                let rows: Vec<Vec<&str>> = rows.iter().map(|row|
                    row["cells"].as_array().unwrap().iter()
                        .map(|cell| cell["value"].as_str().unwrap()).collect()
                ).collect();
                assert_eq!(step.table.as_ref().expect("combined table retained").rows, rows);
            } else {
                assert!(step.table.is_none());
            }
        }
    }
    assert!(docstrings > 0, "oracle must contain actual doc-string assertions");
}

#[given("a feature containing a legacy doc-string payload")]
fn legacy_docstring_payload(world: &mut ArgumentWorld) {
    feature(world, "Given".into(), "doc".into());
}

#[when(expr = "it is parsed using the {string} legacy-content constructor")]
fn legacy_docstring_constructor(world: &mut ArgumentWorld, policy: String) {
    let env = match policy.as_str() {
        "default" => gherkin::GherkinEnv::default(),
        "explicit legacy" => gherkin::GherkinEnv::default()
            .with_docstring_semantics(gherkin::DocStringSemantics::Legacy),
        "English" => gherkin::GherkinEnv::new("en").unwrap(),
        "latest dialect" => gherkin::GherkinEnv::new_latest("en").unwrap(),
        _ => panic!("Unknown content policy: {}", policy),
    };
    world.parsed = Some(gherkin::Feature::parse(&world.source, env));
}

#[then("the original delimiter-adjacent newlines remain unchanged")]
fn legacy_docstring_unchanged(world: &mut ArgumentWorld) {
    assert_eq!(payload(world).docstring.as_deref(), Some("\nreusable payload\n"));
    next_step(world);
}

#[given(expr = "a feature with a {string} doc-string closure")]
fn malformed_docstring_closure(world: &mut ArgumentWorld, closure: String) {
    let argument = match closure.as_str() {
        "missing" => "      \"\"\"\n      payload\n",
        "mismatched" => "      \"\"\"\n      payload\n      ```\n",
        "same line" => "      \"\"\"payload\"\"\"\n",
        _ => panic!("Unknown closure: {}", closure),
    };
    world.source = format!("Feature: Complete payload blocks\n  Scenario: Reject unclosed content\n    Given a payload\n{}", argument);
}

fn main() {
    futures::executor::block_on(
        ArgumentWorld::cucumber()
            .fail_on_skipped()
            .run_and_exit("tests/features/compatibility/step_arguments.feature"),
    );
}

#[given(expr = "a {string} table with a {string} later row")]
fn rectangular_source(world: &mut ArgumentWorld, kind: String, width: String) {
    let row = match width.as_str() {
        "shorter" => "| first |",
        "longer" => "| first | second | discarded |",
        "matching" => "| first | second |",
        _ => panic!("Unknown table width: {}", width),
    };
    let prefix = match kind.as_str() {
        "step" => "Feature: Rectangular payload\n  Scenario: Preserve values\n    Given a payload\n",
        "Examples" => "Feature: Rectangular examples\n  Scenario Outline: Preserve values\n    Given <name> and <value>\n    Examples:\n",
        _ => panic!("Unknown table kind: {}", kind),
    };
    world.source = format!("{}      | name | value |\n      {}\n", prefix, row);
}

#[given("the official Gherkin 43 inconsistent-cell-count fixture")]
fn published_invalid_table(world: &mut ArgumentWorld) {
    world.source = include_str!("fixtures/gherkin43-tables/inconsistent_cell_count.feature").to_owned();
    let errors = include_str!("fixtures/gherkin43-tables/inconsistent_cell_count.feature.errors.ndjson")
        .lines().map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0]["parseError"]["source"]["location"]["line"], 6);
    assert_eq!(errors[1]["parseError"]["source"]["location"]["line"], 14);
    assert!(errors.iter().all(|error| error["parseError"]["message"]
        .as_str().unwrap().contains("inconsistent cell count within the table")));
}

#[when(expr = "the table feature uses {string} semantics")]
fn parse_table_policy(world: &mut ArgumentWorld, policy: String) {
    let env = match policy.as_str() {
        "Gherkin 43" => gherkin::GherkinEnv::default()
            .with_table_semantics(gherkin::TableSemantics::Gherkin43),
        "default" => gherkin::GherkinEnv::default(),
        "explicit legacy" => gherkin::GherkinEnv::default()
            .with_table_semantics(gherkin::TableSemantics::Legacy),
        "English" => gherkin::GherkinEnv::new("en").unwrap(),
        "latest dialect" => gherkin::GherkinEnv::new_latest("en").unwrap(),
        _ => panic!("Unknown table policy: {}", policy),
    };
    world.table_parsed = Some(gherkin::Feature::parse_file_source(
        "rectangular.feature", &world.source, env));
}

#[then("the table parser reports inconsistent cell count")]
fn inconsistent_table(world: &mut ArgumentWorld) {
    match world.table_parsed.as_ref().expect("table parser invoked") {
        Err(gherkin::ParseFileError::Parsing {
            error: Some(gherkin::EnvError::InconsistentCellCount(rows)), ..
        }) => {
            assert!(rows.len() >= 2);
            assert!(rows.iter().skip(1).any(|row| row.len() != rows[0].len()),
                "The diagnostic must retain the actual inconsistent rows");
        }
        other => panic!("Expected inconsistent-cell-count parse failure, got {:?}", other),
    }
}

#[then(expr = "the {string} table preserves both columns")]
fn rectangular_values(world: &mut ArgumentWorld, kind: String) {
    let feature = world.table_parsed.as_ref().expect("table parser invoked")
        .as_ref().expect("rectangular or legacy-truncated table accepted");
    let scenario = &feature.scenarios[0];
    let table = match kind.as_str() {
        "step" => scenario.steps[0].table.as_ref().unwrap(),
        "Examples" => scenario.examples[0].table.as_ref().unwrap(),
        _ => panic!("Unknown table kind: {}", kind),
    };
    assert_eq!(table.rows, vec![vec!["name", "value"], vec!["first", "second"]]);
}
