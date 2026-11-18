use api_versioning::{ApiVersion, CustomFormat, ParseError, set_custom_format};
use std::fmt::{self, Formatter, Write};
use std::sync::Once;

// a custom format that incorrectly calls back into the public API instead of the built-in functions

struct Recursive;

impl CustomFormat for Recursive {
    fn parse<'a>(&self, text: &'a str) -> Result<ApiVersion<'a>, ParseError> {
        ApiVersion::parse(text)
    }

    fn fmt(&self, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{version}")
    }

    fn parse_path<'a>(&self, _segment: &'a str) -> Option<ApiVersion<'a>> {
        ApiVersion::from_type::<v1::Api>().pop()
    }
}

mod v1 {
    pub struct Api;
}

fn install() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| set_custom_format(&Recursive).unwrap());
}

#[test]
fn parse_should_fail_when_custom_format_recursively_parses() {
    // arrange
    install();

    // act
    let error = ApiVersion::parse("1.0").unwrap_err();

    // assert
    assert_eq!(error, ParseError::Recursion);
}

#[test]
fn display_should_fail_when_custom_format_recursively_formats() {
    // arrange
    install();
    let version = api_versioning::api_version!(1.0);
    let mut text = String::new();

    // act
    let result = write!(&mut text, "{version}");

    // assert
    assert_eq!(result, Err(fmt::Error));
}

#[test]
#[should_panic(expected = "the custom API version format recursively parsed a type path")]
fn from_type_should_panic_when_custom_format_recursively_parses() {
    // arrange
    install();

    // act
    let _ = ApiVersion::from_type::<v1::Api>();

    // assert
}
