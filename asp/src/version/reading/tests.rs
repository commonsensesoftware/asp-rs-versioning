use super::{ApiVersion, Format};
use crate::{prelude::*, test::Result};
use rstest::rstest;
use time::{Date, macros::date};

#[test]
fn neutral_api_version_should_have_no_components() {
    // arrange

    // act
    let version = ApiVersion::NEUTRAL;

    // assert
    assert!(version.format() == Format::Unknown);
    assert!(version.is_neutral());
    assert_eq!(version.major(), None);
    assert_eq!(version.minor(), None);
    assert_eq!(version.date(), None);
    assert_eq!(version.status(), None);
}

#[test]
fn default_api_version_should_be_1_0() {
    // arrange

    // act
    let version = ApiVersion::DEFAULT;

    // assert
    assert!(version.format() == Format::Numeric);
    assert!(!version.is_neutral());
    assert_eq!(version.major(), Some(1));
    assert_eq!(version.minor(), Some(0));
    assert_eq!(version.date(), None);
    assert_eq!(version.status(), None);
}

#[test]
fn custom_api_version_should_only_have_bits_and_status() {
    // arrange

    // act
    let version = ApiVersion::new(0b11 << 30 | 42, Some("beta".into()));

    // assert
    assert!(version.format() == Format::Custom);
    assert_eq!(version.bits(), 42);
    assert_eq!(version.major(), None);
    assert_eq!(version.minor(), None);
    assert_eq!(version.date(), None);
    assert_eq!(version.status(), Some("beta"));
}

#[rstest]
#[case(api_version!(0.0), Some(0), Some(0), None)]
#[case(api_version!(1.20), Some(1), Some(20), None)]
#[case(api_version!(255.255-Preview.1), Some(255), Some(255), Some("Preview.1"))]
fn numeric_api_version_should_have_expected_parts(
    #[case] version: ApiVersion<'static>,
    #[case] major: Option<u8>,
    #[case] minor: Option<u8>,
    #[case] status: Option<&str>,
) {
    // arrange

    // act

    // assert
    assert!(version.format() == Format::Numeric);
    assert_eq!(version.major(), major);
    assert_eq!(version.minor(), minor);
    assert_eq!(version.date(), None);
    assert_eq!(version.status(), status);
}

#[rstest]
#[case(api_version!(2000-01-01), Some(date!(2000-01-01)), None)]
#[case(api_version!(2024-02-29-rc.2), Some(date!(2024-02-29)), Some("rc.2"))]
#[case(api_version!(2255-12-31), Some(date!(2255-12-31)), None)]
fn date_api_version_should_have_expected_parts(
    #[case] version: ApiVersion<'static>,
    #[case] date: Option<Date>,
    #[case] status: Option<&str>,
) {
    // arrange

    // act

    // assert
    assert!(version.format() == Format::Date);
    assert_eq!(version.major(), None);
    assert_eq!(version.minor(), None);
    assert_eq!(version.date(), date);
    assert_eq!(version.status(), status);
}

#[rstest]
#[case("1.0-beta")]
#[case("2024-12-31-rc.2")]
fn from_str_status_remains_readable_after_input_is_dropped(#[case] text: &str) -> Result {
    // arrange

    // act
    let version = {
        let input = text.to_owned();
        input.parse::<ApiVersion<'static>>()?
    };

    // assert
    assert_eq!(version, ApiVersion::parse(text)?);
    assert_eq!(version.status(), text.rsplit_once('-').map(|(_, status)| status));
    Ok(())
}
