use std::path::Path;

use gherkin_rust::{Feature, ParseFileError};

fn load_feature<P: AsRef<Path>>(path: P) -> Result<Feature, ParseFileError> {
    Feature::parse_path(path.as_ref(), Default::default())
}

#[test]
fn inconsistent_cell_count() {
    let error = load_feature("./tests/fixtures/data/bad/inconsistent_cell_count.feature").unwrap_err();
    match error {
        ParseFileError::Reading { .. } => panic!("Invalid error"),
        ParseFileError::Parsing { error, .. } => {
            let error = error.unwrap();
            match error {
                gherkin_rust::EnvError::InconsistentCellCount(_) => {}
                _ => panic!("Invalid error")
            }
        }
    };
}

#[test]
fn invalid_language() {
    let error = load_feature("./tests/fixtures/data/bad/invalid_language.feature").unwrap_err();
    
}

#[test]
fn multiple_parser_errors() {
    let error = load_feature("./tests/fixtures/data/bad/multiple_parser_errors.feature").unwrap_err();

}

#[test]
fn not_gherkin() {
    let error = load_feature("./tests/fixtures/data/bad/not_gherkin.feature").unwrap_err();

}

#[test]
fn single_parser_error() {
    let error = load_feature("./tests/fixtures/data/bad/single_parser_error.feature").unwrap_err();

}

#[test]
fn unexpected_eof() {
    let error = load_feature("./tests/fixtures/data/bad/unexpected_eof.feature").unwrap_err();

}

#[test]
fn whitespace_in_tags() {
    let error = load_feature("./tests/fixtures/data/bad/whitespace_in_tags.feature").unwrap_err();

}