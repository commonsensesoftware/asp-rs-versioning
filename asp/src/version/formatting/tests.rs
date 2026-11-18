use super::{ApiVersion, CustomFormat};
use crate::builtin;
use crate::test::{Opaque, Revision};
use rstest::rstest;
use std::fmt::{self, Display, Formatter, Write};

#[test]
fn display_should_use_asterisk_when_neutral() {
    // arrange
    let original = ApiVersion::NEUTRAL.to_string();
    let expected = "*";

    // act
    let actual = format!("{:#}", ApiVersion::NEUTRAL);

    // assert
    assert_eq!(original, expected);
    assert_eq!(actual, expected);
}

#[rstest]
#[case(ApiVersion::NEUTRAL, "*", "*")]
#[case(ApiVersion::DEFAULT, "1.0", "1")]
#[case(api_version!(0), "0.0", "0")]
#[case(api_version!(1.10), "1.10", "1.10")]
#[case(api_version!(255.255), "255.255", "255.255")]
#[case(api_version!(2-preview.1), "2.0-preview.1", "2-preview.1")]
#[case(api_version!(2.1-RC.2), "2.1-RC.2", "2.1-RC.2")]
#[case(api_version!(2000-01-01), "2000-01-01", "2000-01-01")]
#[case(api_version!(2024-02-29-rc.2), "2024-02-29-rc.2", "2024-02-29-rc.2")]
#[case(api_version!(2255-12-31), "2255-12-31", "2255-12-31")]
fn display_uses_the_builtin_format(
    #[case] version: ApiVersion<'static>,
    #[case] expected_default: &str,
    #[case] expected_alt: &str,
) {
    // arrange
    let actual_default = version.to_string();

    // act
    let actual_alt = format!("{version:#}");

    // assert
    assert_eq!(actual_default, expected_default);
    assert_eq!(actual_alt, expected_alt);
}

#[rstest]
#[case(ApiVersion::NEUTRAL, "ApiVersion { neutral: true }")]
#[case(ApiVersion::DEFAULT, "ApiVersion { major: 1, minor: 0, status: \"\" }")]
#[case(api_version!(255.255-beta2), "ApiVersion { major: 255, minor: 255, status: \"beta2\" }")]
#[case(api_version!(2000-01-01), "ApiVersion { date: 2000-01-01, status: \"\" }")]
#[case(api_version!(2024-02-29-rc.2), "ApiVersion { date: 2024-02-29, status: \"rc.2\" }")]
#[case(
    ApiVersion::numeric(1, 2, 3, None),
    "ApiVersion { major: 1, minor: 2, extension: 3, status: \"\" }"
)]
#[case(ApiVersion::date(time::macros::date!(2000-01-01), 3, Some("rc".into())), "ApiVersion { date: 2000-01-01, extension: 3, status: \"rc\" }")]
#[case(ApiVersion::new(0b11 << 30 | 42, None), "ApiVersion { custom: 42, status: \"\" }")]
fn debug_includes_the_builtin_components(#[case] version: ApiVersion<'static>, #[case] expected: &str) {
    // arrange

    // act
    let actual = format!("{version:?}");

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn display_should_be_empty_when_custom() {
    // arrange
    let version = ApiVersion::new(0b11 << 30 | 42, Some("beta".into()));

    // act
    let actual = version.to_string();

    // assert
    assert_eq!(actual, "");
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

#[rstest]
#[case(ApiVersion::NEUTRAL)]
#[case(ApiVersion::DEFAULT)]
#[case(api_version!(1.2-beta))]
#[case(api_version!(2024-02-29))]
#[case(api_version!(2024-02-29-rc.2))]
fn formatting_propagates_writer_errors(#[case] version: ApiVersion<'static>) {
    // arrange
    let expected: fmt::Result = Err(fmt::Error);

    // act
    let results = [
        write!(&mut FailingWriter, "{version}"),
        write!(&mut FailingWriter, "{version:#}"),
        write!(&mut FailingWriter, "{version:?}"),
    ];

    // assert
    for actual in results {
        assert_eq!(actual, expected);
    }
}

struct WithCustom<'a>(&'a ApiVersion<'a>, Option<&'a dyn CustomFormat>);

impl Display for WithCustom<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.display(f, self.1)
    }
}

#[rstest]
#[case(ApiVersion::custom(42, None), "r42", "R42")]
#[case(ApiVersion::custom(7, Some("beta".into())), "r7-beta", "R7-beta")]
fn display_should_use_custom_format(
    #[case] version: ApiVersion<'static>,
    #[case] expected_default: &str,
    #[case] expected_alt: &str,
) {
    // arrange
    let custom = WithCustom(&version, Some(&Revision));

    // act
    let actual_default = custom.to_string();
    let actual_alt = format!("{custom:#}");

    // assert
    assert_eq!(actual_default, expected_default);
    assert_eq!(actual_alt, expected_alt);
}

#[rstest]
#[case(ApiVersion::custom(42, None))]
#[case(api_version!(1.0))]
#[case(api_version!(2026-10-01))]
fn display_should_always_use_custom_format(#[case] version: ApiVersion<'static>) {
    // arrange

    // act
    let actual = WithCustom(&version, Some(&Opaque)).to_string();

    // assert
    assert_eq!(actual, "opaque");
}

#[test]
fn display_should_be_empty_when_custom_without_custom_format() {
    // arrange
    let version = ApiVersion::custom(42, None);

    // act
    let actual = WithCustom(&version, None).to_string();

    // assert
    assert_eq!(actual, "");
}

#[test]
fn display_should_allow_custom_format_to_use_builtin_format() {
    // arrange
    let version = api_version!(1.0);

    // act
    let actual = WithCustom(&version, Some(&Revision)).to_string();

    // assert
    assert_eq!(actual, "1.0");
}

#[test]
fn display_should_propagate_custom_format_writer_errors() {
    // arrange
    let version = ApiVersion::custom(42, Some("beta".into()));
    let custom = WithCustom(&version, Some(&Revision));

    // act
    let result = write!(&mut FailingWriter, "{custom}");

    // assert
    assert_eq!(result, Err(fmt::Error));
}

#[test]
fn display_should_never_use_custom_format_for_neutral() {
    // arrange
    let version = ApiVersion::NEUTRAL;

    // act
    let actual = WithCustom(&version, Some(&Opaque)).to_string();

    // assert
    assert_eq!(actual, "*");
}

// formats all API versions by recursively formatting them again
struct Recursive;

impl CustomFormat for Recursive {
    fn parse<'a>(&self, _text: &'a str) -> Result<ApiVersion<'a>, crate::ParseError> {
        unreachable!()
    }

    fn fmt(&self, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
        version.display(f, Some(self))
    }
}

#[test]
fn display_should_fail_when_custom_format_recursively_formats() {
    // arrange
    let version = api_version!(1.0);
    let mut text = String::new();

    // act
    let result = write!(&mut text, "{}", WithCustom(&version, Some(&Recursive)));

    // assert
    assert_eq!(result, Err(fmt::Error));
}

#[test]
fn display_should_allow_custom_format_after_recursion_failure() {
    // arrange
    let version = ApiVersion::custom(42, None);
    let mut text = String::new();
    let _ = write!(&mut text, "{}", WithCustom(&version, Some(&Recursive)));

    // act
    let actual = WithCustom(&version, Some(&Revision)).to_string();

    // assert
    assert_eq!(actual, "r42");
}

struct Builtin<'a>(&'a ApiVersion<'a>);

impl Display for Builtin<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        builtin::fmt(self.0, f)
    }
}

#[rstest]
#[case(ApiVersion::NEUTRAL, "*")]
#[case(api_version!(1.2-beta), "1.2-beta")]
#[case(ApiVersion::numeric(1, 2, 3, Some("beta".into())), "1.2-beta")]
#[case(ApiVersion::date(time::macros::date!(2026-10-01), 3, None), "2026-10-01")]
#[case(ApiVersion::custom(42, None), "")]
fn builtin_fmt_should_only_write_builtin_components(#[case] version: ApiVersion<'static>, #[case] expected: &str) {
    // arrange

    // act
    let actual = Builtin(&version).to_string();

    // assert
    assert_eq!(actual, expected);
}
