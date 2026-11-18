use super::{ApiVersion, Format, builtin::mask};
use crate::prelude::*;
use rstest::rstest;
use std::borrow::Cow::Borrowed;
use std::cmp::Ordering::{self, *};
use time::macros::date;

fn custom(bits: u32, status: Option<&'static str>) -> ApiVersion<'static> {
    ApiVersion::custom(bits, status.map(Borrowed))
}

#[rstest]
#[case(api_version!(1.0), api_version!(2.0), Less)]
#[case(api_version!(2.0), api_version!(1.0), Greater)]
#[case(api_version!(1.0), api_version!(1.0), Equal)]
#[case(api_version!(1.10), api_version!(1.9), Greater)]
#[case(api_version!(2026-01-01), api_version!(2026-10-01), Less)]
#[case(api_version!(1.0-beta), api_version!(1.0), Less)]
#[case(api_version!(1.0), api_version!(1.0-beta), Greater)]
#[case(api_version!(1.0-alpha), api_version!(1.0-beta), Less)]
#[case(ApiVersion::NEUTRAL, ApiVersion::NEUTRAL, Equal)]
fn partial_cmp_should_order_versions_with_same_format(
    #[case] left: ApiVersion<'static>,
    #[case] right: ApiVersion<'static>,
    #[case] expected: Ordering,
) {
    // arrange

    // act
    let ordering = left.partial_cmp(&right);

    // assert
    assert_eq!(ordering, Some(expected));
}

#[rstest]
#[case(custom(1, None), custom(2, None), Less)]
#[case(custom(1, None), custom(1, Some("beta")), Greater)]
#[case(custom(1, Some("beta")), custom(1, Some("beta")), Equal)]
fn partial_cmp_should_order_custom_versions_by_bits_then_status(
    #[case] left: ApiVersion<'static>,
    #[case] right: ApiVersion<'static>,
    #[case] expected: Ordering,
) {
    // arrange

    // act
    let ordering = left.partial_cmp(&right);

    // assert
    assert_eq!(ordering, Some(expected));
}

#[rstest]
#[case(api_version!(1.0), api_version!(2026-10-01))]
#[case(api_version!(1.0), ApiVersion::NEUTRAL)]
#[case(api_version!(2026-10-01), ApiVersion::NEUTRAL)]
#[case(custom(0, None), ApiVersion::NEUTRAL)]
#[case(custom(1, None), api_version!(1.0))]
#[case(custom(1, None), api_version!(2026-10-01))]
fn partial_cmp_should_not_order_versions_with_different_formats(
    #[case] left: ApiVersion<'static>,
    #[case] right: ApiVersion<'static>,
) {
    // arrange

    // act
    let forward = left.partial_cmp(&right);
    let backward = right.partial_cmp(&left);

    // assert
    assert_eq!(forward, None);
    assert_eq!(backward, None);
}

#[rstest]
#[case(api_version!(1.0), api_version!(2.0), Less)]
#[case(api_version!(1.0-beta), api_version!(1.0), Less)]
#[case(api_version!(1.0), api_version!(2026-10-01), Less)]
#[case(api_version!(2026-10-01), api_version!(1.0), Greater)]
#[case(ApiVersion::NEUTRAL, api_version!(1.0), Less)]
#[case(api_version!(1.0), ApiVersion::NEUTRAL, Greater)]
#[case(ApiVersion::NEUTRAL, api_version!(2026-10-01), Less)]
#[case(api_version!(2026-10-01), ApiVersion::NEUTRAL, Greater)]
#[case(ApiVersion::NEUTRAL, custom(0, None), Less)]
#[case(custom(0, None), ApiVersion::NEUTRAL, Greater)]
#[case(custom(u32::MAX, None), api_version!(0.0), Less)]
#[case(api_version!(0.0), custom(u32::MAX, None), Greater)]
#[case(custom(u32::MAX, None), api_version!(2000-01-01), Less)]
#[case(api_version!(2000-01-01), custom(u32::MAX, None), Greater)]
fn total_cmp_should_order_versions(
    #[case] left: ApiVersion<'static>,
    #[case] right: ApiVersion<'static>,
    #[case] expected: Ordering,
) {
    // arrange

    // act
    let ordering = left.total_cmp(&right);

    // assert
    assert_eq!(ordering, expected);
}

#[test]
fn total_cmp_should_sort_versions_by_format_then_value() {
    // arrange
    let mut versions = vec![
        api_version!(2026 - 10 - 01),
        custom(2, None),
        api_version!(2.0),
        ApiVersion::NEUTRAL,
        api_version!(1.0),
        custom(1, None),
        api_version!(2026 - 01 - 01),
        api_version!(1.0 - beta),
    ];

    // act
    versions.sort_by(ApiVersion::total_cmp);

    // assert
    assert_eq!(
        versions,
        [
            ApiVersion::NEUTRAL,
            custom(1, None),
            custom(2, None),
            api_version!(1.0 - beta),
            api_version!(1.0),
            api_version!(2.0),
            api_version!(2026 - 01 - 01),
            api_version!(2026 - 10 - 01),
        ]
    );
}

#[test]
fn to_owned_should_copy_borrowed_status() {
    // arrange
    let status = String::from("beta");
    let borrowed = ApiVersion::new(ApiVersion::DEFAULT.0, Some(Borrowed(status.as_str())));

    // act
    let owned = borrowed.to_owned();

    // assert
    assert_eq!(owned, borrowed);
    drop(status);
    assert_eq!(owned, api_version!(1.0 - beta));
}

#[test]
fn from_reference_should_return_owned_version() {
    // arrange
    let version = api_version!(1.0 - beta);

    // act
    let owned = ApiVersion::from(&version);

    // assert
    assert_eq!(owned, version);
}

#[rstest]
#[case(0, 0)]
#[case(42, 42)]
#[case(0b00111111_11111111_11111111_11111111, 0b00111111_11111111_11111111_11111111)]
#[case(u32::MAX, 0b00111111_11111111_11111111_11111111)]
#[case(0b01000000_00000000_00000000_00000001, 1)]
fn custom_should_create_custom_format_with_lower_bits(#[case] bits: u32, #[case] expected: u32) {
    // arrange

    // act
    let version = ApiVersion::custom(bits, None);

    // assert
    assert!(version.format() == Format::Custom);
    assert_eq!(version.bits(), expected);
}

#[test]
fn custom_should_be_usable_in_constant_expressions() {
    // arrange
    const VERSION: ApiVersion<'static> = ApiVersion::custom(42, Some(Borrowed("beta")));

    // act
    let version = VERSION;

    // assert
    assert!(version.format() == Format::Custom);
    assert_eq!(version.bits(), 42);
    assert_eq!(version.status(), Some("beta"));
}

#[rstest]
#[case(ApiVersion::numeric(1, 0, 0, None), api_version!(1.0))]
#[case(ApiVersion::numeric(255, 255, 0, Some(Borrowed("beta"))), api_version!(255.255-beta))]
#[case(ApiVersion::date(date!(2026-10-01), 0, None), api_version!(2026-10-01))]
#[case(ApiVersion::date(date!(2255-12-31), 0, Some(Borrowed("rc.1"))), api_version!(2255-12-31-rc.1))]
fn constructors_without_extension_should_equal_builtin_versions(
    #[case] constructed: ApiVersion<'static>,
    #[case] expected: ApiVersion<'static>,
) {
    // arrange

    // act

    // assert
    assert_eq!(constructed, expected);
}

#[test]
fn numeric_should_retain_components_with_extension() {
    // arrange

    // act
    let version = ApiVersion::numeric(1, 2, 3, Some(Borrowed("beta")));

    // assert
    assert!(version.format() == Format::Numeric);
    assert_eq!(version.major(), Some(1));
    assert_eq!(version.minor(), Some(2));
    assert_eq!(version.bits() & mask::NUMERIC, 3);
    assert_eq!(version.status(), Some("beta"));
}

#[test]
fn date_should_retain_components_with_extension() {
    // arrange

    // act
    let version = ApiVersion::date(date!(2024 - 02 - 29), 3, Some(Borrowed("beta")));

    // assert
    assert!(version.format() == Format::Date);
    assert_eq!(version.date(), Some(date!(2024 - 02 - 29)));
    assert_eq!(version.bits() & mask::DATE, 3);
    assert_eq!(version.status(), Some("beta"));
}

#[test]
fn numeric_should_only_retain_extension_bits() {
    // arrange

    // act
    let version = ApiVersion::numeric(255, 255, u32::MAX, None);

    // assert
    assert!(version.format() == Format::Numeric);
    assert_eq!(version.major(), Some(255));
    assert_eq!(version.minor(), Some(255));
    assert_eq!(version.bits() & mask::NUMERIC, mask::NUMERIC);
}

#[test]
fn date_should_only_retain_extension_bits() {
    // arrange

    // act
    let version = ApiVersion::date(date!(2255 - 12 - 31), u32::MAX, None);

    // assert
    assert!(version.format() == Format::Date);
    assert_eq!(version.date(), Some(date!(2255 - 12 - 31)));
    assert_eq!(version.bits() & mask::DATE, mask::DATE);
}

#[rstest]
#[case(date!(1999-12-31))]
#[case(date!(2256-01-01))]
#[should_panic(expected = "the specified API version date must be between the years 2000 and 2255")]
fn date_should_panic_when_year_is_out_of_range(#[case] date: time::Date) {
    // arrange

    // act
    let _ = ApiVersion::date(date, 0, None);

    // assert
}

#[test]
fn constructors_should_be_usable_in_constant_expressions() {
    // arrange
    const NUMERIC: ApiVersion<'static> = ApiVersion::numeric(1, 2, 3, None);
    const DATE: ApiVersion<'static> = ApiVersion::date(date!(2026 - 10 - 01), 3, None);

    // act
    let (numeric, date) = (NUMERIC, DATE);

    // assert
    assert_eq!(numeric.major(), Some(1));
    assert_eq!(date.date(), Some(date!(2026 - 10 - 01)));
}

#[rstest]
#[case(ApiVersion::numeric(1, 2, 3, None), api_version!(1.2), Greater)]
#[case(ApiVersion::numeric(1, 2, 0, None), api_version!(1.2), Equal)]
#[case(ApiVersion::numeric(1, 2, 3, None), ApiVersion::numeric(1, 2, 10, None), Less)]
#[case(ApiVersion::numeric(1, 2, mask::NUMERIC, None), api_version!(1.3), Less)]
#[case(
    ApiVersion::numeric(1, 2, 3, Some(Borrowed("beta"))),
    ApiVersion::numeric(1, 2, 3, None),
    Less
)]
#[case(ApiVersion::date(date!(2026-10-01), 1, None), api_version!(2026-10-01), Greater)]
#[case(ApiVersion::date(date!(2026-10-01), mask::DATE, None), api_version!(2026-10-02), Less)]
fn partial_cmp_should_order_by_components_then_extension_then_status(
    #[case] left: ApiVersion<'static>,
    #[case] right: ApiVersion<'static>,
    #[case] expected: Ordering,
) {
    // arrange

    // act
    let ordering = left.partial_cmp(&right);

    // assert
    assert_eq!(ordering, Some(expected));
}

#[rstest]
#[case(ApiVersion::numeric(0, 0, u32::MAX, None), mask::NUMERIC)]
#[case(ApiVersion::date(date!(2000-01-01), u32::MAX, None), mask::DATE)]
#[case(ApiVersion::custom(u32::MAX, None), mask::CUSTOM)]
fn masks_should_cover_available_bits(#[case] version: ApiVersion<'static>, #[case] mask: u32) {
    // arrange

    // act
    let bits = version.bits();

    // assert
    assert_eq!(bits & mask, mask);
}
