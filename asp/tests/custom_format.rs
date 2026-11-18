use api_versioning::{
    ApiVersion, CustomFormat, Format, ParseError, SetCustomFormatError, api_version, builtin, set_custom_format,
};
use std::fmt::{self, Formatter};
use std::sync::Once;

// the custom format is process-wide, so every test in this binary shares the same installed format

const fn parse_revision(text: &str) -> Option<u32> {
    let bytes = text.as_bytes();

    if bytes.len() < 2 || bytes[0] != b'r' {
        return None;
    }

    let mut value = 0u32;
    let mut i = 1;

    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return None;
        }

        value = value * 10 + (bytes[i] - b'0') as u32;
        i += 1;
    }

    Some(value)
}

macro_rules! revision {
    ($token:tt) => {
        match parse_revision(stringify!($token)) {
            Some(bits) => ApiVersion::custom(bits, None),
            None => panic!("the specified revision is invalid"),
        }
    };
}

struct Revision;

impl CustomFormat for Revision {
    fn parse<'a>(&self, text: &'a str) -> Result<ApiVersion<'a>, ParseError> {
        if !text.starts_with('r') {
            return builtin::parse(text);
        }

        match parse_revision(text) {
            Some(bits) => Ok(ApiVersion::custom(bits, None)),
            None => Err(ParseError::Custom(format!("'{text}' is not a revision"))),
        }
    }

    fn fmt(&self, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
        if version.format() != Format::Custom {
            return builtin::fmt(version, f);
        }

        write!(f, "r{}", version.bits())
    }

    fn parse_path<'a>(&self, segment: &'a str) -> Option<ApiVersion<'a>> {
        match parse_revision(segment) {
            Some(bits) => Some(ApiVersion::custom(bits, None)),
            None => builtin::parse_path(segment),
        }
    }
}

mod r3 {
    pub mod v1 {
        pub struct Api;
    }
}

fn install() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| set_custom_format(&Revision).unwrap());
}

#[test]
fn parse_should_use_installed_custom_format() {
    // arrange
    install();

    // act
    let version = ApiVersion::parse("r2").unwrap();

    // assert
    assert_eq!(version, revision!(r2));
}

#[test]
fn from_str_should_use_installed_custom_format() {
    // arrange
    install();

    // act
    let version = String::from("r2").parse::<ApiVersion<'static>>().unwrap();

    // assert
    assert_eq!(version, revision!(r2));
}

#[test]
fn parse_should_still_parse_builtin_formats() {
    // arrange
    install();

    // act
    let numeric = ApiVersion::parse("1.0").unwrap();
    let date = ApiVersion::parse("2026-10-01").unwrap();

    // assert
    assert_eq!(numeric, api_version!(1.0));
    assert_eq!(date, api_version!(2026 - 10 - 01));
}

#[test]
fn parse_should_return_error_from_installed_custom_format() {
    // arrange
    install();

    // act
    let error = ApiVersion::parse("rx").unwrap_err();

    // assert
    assert_eq!(error, ParseError::Custom("'rx' is not a revision".into()));
}

#[test]
fn to_string_should_use_installed_custom_format() {
    // arrange
    install();
    const VERSION: ApiVersion<'static> = revision!(r42);

    // act
    let text = VERSION.to_string();

    // assert
    assert_eq!(text, "r42");
}

#[test]
fn custom_versions_should_be_ordered_by_bits() {
    // arrange
    install();
    let mut versions = vec![revision!(r10), revision!(r2), revision!(r1)];

    // act
    versions.sort_by(ApiVersion::total_cmp);

    // assert
    assert_eq!(versions, [revision!(r1), revision!(r2), revision!(r10)]);
}

#[test]
fn from_type_should_use_installed_custom_format() {
    // arrange
    install();

    // act
    let versions = ApiVersion::from_type::<r3::v1::Api>();

    // assert
    assert_eq!(versions, [revision!(r3), api_version!(1)]);
}

#[test]
fn set_custom_format_should_fail_when_already_set() {
    // arrange
    install();

    // act
    let result = set_custom_format(&Revision);

    // assert
    assert_eq!(result, Err(SetCustomFormatError));
}
