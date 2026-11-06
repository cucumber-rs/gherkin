use gherkin::{Feature, GherkinEnv};

#[test]
fn parse_comment_only() {
    let feature = Feature::parse("# only a comment", GherkinEnv::default()).unwrap();
    assert_eq!(feature, None);
}

#[test]
fn parse_path_empty() {
    let feature = Feature::parse_path(
        "./tests/fixtures/data/good/empty.feature",
        GherkinEnv::default(),
    )
    .unwrap();
    assert_eq!(feature, None);
}

#[test]
fn parse_path_comment_only() {
    let feature = Feature::parse_path(
        "./tests/fixtures/data/good/incomplete_feature_3.feature",
        GherkinEnv::default(),
    )
    .unwrap();
    assert_eq!(feature, None);
}

#[test]
fn parse_path_sets_path() {
    let feature = Feature::parse_path(
        "./tests/fixtures/data/good/minimal.feature",
        GherkinEnv::default(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        feature.path.as_deref(),
        Some("./tests/fixtures/data/good/minimal.feature".as_ref()),
    );
}

#[test]
fn parse_comments_before_feature() {
    let feature = Feature::parse(
        "# a comment\n\n  # another\nFeature: Foo\n",
        GherkinEnv::default(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(feature.name, "Foo");
}

#[test]
fn parse_language_header_with_feature() {
    let feature = Feature::parse(
        "# language: fr\nFonctionnalité: Foo\n",
        GherkinEnv::default(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(feature.name, "Foo");
}

#[test]
fn parse_crlf_comment_only() {
    let feature = Feature::parse("# a\r\n\r\n# b\r\n", GherkinEnv::default()).unwrap();
    assert_eq!(feature, None);
}

#[test]
fn parse_language_header_without_feature_is_error() {
    assert!(Feature::parse("# language: fr\n", GherkinEnv::default()).is_err());
}

#[test]
fn parse_scenario_without_feature_is_error() {
    assert!(Feature::parse("Scenario: Foo\n", GherkinEnv::default()).is_err());
}
