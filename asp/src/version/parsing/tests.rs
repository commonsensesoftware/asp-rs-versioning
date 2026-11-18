use super::{
    ApiVersion, CustomFormat,
    ParseError::{self, *},
    builtin,
};
use crate::test::{Opaque, Revision};
use crate::{prelude::*, test::Result};
use rstest::rstest;
use std::any::type_name;
use time::{Date, Month};

const NUMERIC: ApiVersion<'static> = api_version!(255.255 - beta2);
const DATE: ApiVersion<'static> = api_version!(2024 - 12 - 31 - rc1);

#[test]
fn unquoted_version_macros_should_use_the_literal_parser() {
    // arrange

    // act
    const NUMERIC: ApiVersion<'static> = api_version!(1.10);
    const DATE: ApiVersion<'static> = api_version!(2026 - 10 - 01);
    const SPACED: ApiVersion<'static> = api_version!(2026 - 10 - 01);
    const MAJOR_ONLY: ApiVersion<'static> = api_version!(2);
    const PREVIEW: ApiVersion<'static> = api_version!(2.1 - preview.1);
    const RELEASE_CANDIDATE: ApiVersion<'static> = api_version!(2026 - 10 - 01 - rc.2);

    // assert
    assert_eq!(NUMERIC.major(), Some(1));
    assert_eq!(NUMERIC.minor(), Some(10));
    assert!(NUMERIC == ApiVersion::parse_literal("1.10"));
    assert!(DATE == ApiVersion::parse_literal("2026-10-01"));
    assert!(SPACED == DATE);
    assert!(MAJOR_ONLY == ApiVersion::parse_literal("2"));
    assert_eq!(PREVIEW.major(), Some(2));
    assert_eq!(PREVIEW.minor(), Some(1));
    assert_eq!(PREVIEW.status(), Some("preview.1"));
    assert!(PREVIEW == ApiVersion::parse_literal("2.1-preview.1"));
    assert_eq!(RELEASE_CANDIDATE.date(), DATE.date());
    assert_eq!(RELEASE_CANDIDATE.status(), Some("rc.2"));
    assert!(RELEASE_CANDIDATE == ApiVersion::parse_literal("2026-10-01-rc.2"));
}

#[test]
fn compile_time_parsing_supports_macro() -> Result {
    // arrange
    let date = Date::from_calendar_date(2024, Month::December, 31)?;

    // act

    // assert
    assert_eq!(NUMERIC.major(), Some(255));
    assert_eq!(NUMERIC.minor(), Some(255));
    assert_eq!(NUMERIC.status(), Some("beta2"));
    assert_eq!(DATE.date(), Some(date));
    assert_eq!(DATE.status(), Some("rc1"));
    Ok(())
}

#[test]
fn compile_time_parsing_supports_errors() -> Result {
    // arrange

    // act
    let invalid_date = ApiVersion::parse("2023-02-29");
    let invalid_status = ApiVersion::parse("1.0-2beta");
    let invalid_format = ApiVersion::parse("256");

    // assert
    assert_eq!(invalid_date, Err(InvalidDate("2023-02-29".to_owned())));
    assert_eq!(invalid_status, Err(InvalidStatus("2beta".to_owned())));
    assert_eq!(invalid_format, Err(InvalidFormat));
    Ok(())
}

#[rstest]
#[case("0", 0, 0, None)]
#[case("1", 1, 0, None)]
#[case("1.10", 1, 10, None)]
#[case("0.255", 0, 255, None)]
#[case("001.002", 1, 2, None)]
#[case("+1.+2", 1, 2, None)]
#[case("255.255", 255, 255, None)]
#[case("1.2-beta2", 1, 2, Some("beta2"))]
#[case("2.1-preview.1", 2, 1, Some("preview.1"))]
#[case("1.0-beta.1.2", 1, 0, Some("beta.1.2"))]
#[case("1-RC.2", 1, 0, Some("RC.2"))]
fn numeric_versions_preserve_components(
    #[case] text: &str,
    #[case] major: u8,
    #[case] minor: u8,
    #[case] status: Option<&str>,
) -> Result {
    // arrange
    let input = text.to_owned();

    // act
    let version = ApiVersion::parse(&input)?;

    // assert
    assert_eq!(version.major(), Some(major), "{text}");
    assert_eq!(version.minor(), Some(minor), "{text}");
    assert_eq!(version.status(), status, "{text}");

    if let Some(status) = version.status() {
        assert_eq!(status.as_ptr(), input.as_ptr().wrapping_add(input.len() - status.len()));
    }

    assert!(version == ApiVersion::try_from(input.as_str())?);
    assert!(version == input.parse()?);
    assert_eq!(version, ApiVersion::parse_literal(text));
    Ok(())
}

#[rstest]
#[case(2000, 1, 1, None)]
#[case(2000, 2, 29, None)]
#[case(2024, 1, 31, None)]
#[case(2024, 2, 29, Some("preview.1"))]
#[case(2024, 3, 31, None)]
#[case(2024, 4, 30, None)]
#[case(2024, 5, 31, None)]
#[case(2024, 6, 30, None)]
#[case(2024, 7, 31, None)]
#[case(2024, 8, 31, None)]
#[case(2024, 9, 30, None)]
#[case(2024, 10, 31, None)]
#[case(2024, 11, 30, None)]
#[case(2024, 12, 31, Some("RC.2"))]
#[case(2255, 12, 31, None)]
fn dates_round_trip_all_months_and_year_boundaries(
    #[case] year: i32,
    #[case] month: u8,
    #[case] day: u8,
    #[case] status: Option<&str>,
) -> Result {
    // arrange
    let mut input = format!("{year:04}-{month:02}-{day:02}");
    let expected = Date::from_calendar_date(year, Month::try_from(month)?, day)?;

    if let Some(status) = status {
        input.push('-');
        input.push_str(status);
    }

    // act
    let version = ApiVersion::parse(&input)?;

    // assert
    assert_eq!(version.date(), Some(expected));
    assert_eq!(version.status(), status);
    assert_eq!(version, ApiVersion::try_from(input.as_str())?);
    assert_eq!(version, input.parse::<ApiVersion<'static>>()?);
    assert_eq!(version, ApiVersion::parse_literal(&input));

    if let Some(status) = version.status() {
        assert_eq!(status.as_ptr(), input.as_ptr().wrapping_add(input.len() - status.len()));
    }

    Ok(())
}

#[rstest]
#[case("", InvalidFormat)]
#[case(".", InvalidFormat)]
#[case("1.", InvalidFormat)]
#[case(".1", InvalidFormat)]
#[case("1.2.3", InvalidFormat)]
#[case("256", InvalidFormat)]
#[case("1.256", InvalidFormat)]
#[case("999999999999999999999999", InvalidFormat)]
#[case("-1", InvalidStatus("1".into()))]
#[case("+", InvalidFormat)]
#[case(" 1", InvalidFormat)]
#[case("1 ", InvalidFormat)]
#[case("1-", InvalidFormat)]
#[case("2024-01-01-", InvalidFormat)]
#[case("2024-01-01junk", InvalidFormat)]
#[case("2024-01-01β", InvalidFormat)]
#[case("éééééé", InvalidFormat)]
#[case("123456789é", InvalidFormat)]
#[case("１.０", InvalidFormat)]
#[case("*", InvalidFormat)]
#[case("2024-1-01", InvalidStatus("1-01".into()))]
#[case("1999-12-31", InvalidDate("1999-12-31".into()))]
#[case("2256-01-01", InvalidDate("2256-01-01".into()))]
#[case("2023-02-29", InvalidDate("2023-02-29".into()))]
#[case("2100-02-29", InvalidDate("2100-02-29".into()))]
#[case("2024-00-01", InvalidDate("2024-00-01".into()))]
#[case("2024-13-01", InvalidDate("2024-13-01".into()))]
#[case("2024-01-00", InvalidDate("2024-01-00".into()))]
#[case("2024-01-32", InvalidDate("2024-01-32".into()))]
#[case("2024-04-31", InvalidDate("2024-04-31".into()))]
#[case("2023-02-29-beta", InvalidDate("2023-02-29".into()))]
#[case("1-1beta", InvalidStatus("1beta".into()))]
#[case("1-beta-x", InvalidStatus("beta-x".into()))]
#[case("1-béta", InvalidStatus("béta".into()))]
#[case("2024-01-01-rc_1", InvalidStatus("rc_1".into()))]
#[case("1-.beta", InvalidStatus(".beta".into()))]
#[case("1-beta.", InvalidStatus("beta.".into()))]
#[case("1-beta..1", InvalidStatus("beta..1".into()))]
fn malformed_input_returns_exact_errors_without_panicking(#[case] text: &str, #[case] expected: ParseError) {
    // arrange

    // act

    // asert
    assert_eq!(ApiVersion::parse(text), Err(expected.clone()));
    assert_eq!(ApiVersion::try_from(text), Err(expected.clone()));
    assert_eq!(text.parse::<ApiVersion>(), Err(expected));
}

#[test]
fn from_str_returns_owned_version() {
    // arrange

    // act
    let version = {
        let input = String::from("2024-12-31-beta");
        input.parse::<ApiVersion<'static>>().unwrap()
    };

    // assert
    assert_eq!(version.status(), Some("beta"));
}

#[test]
fn from_str_returns_owned_error() {
    // arrange

    // act
    let error = {
        let input = String::from("1-béta");
        input.parse::<ApiVersion<'static>>().err().unwrap()
    };

    // assert
    assert_eq!(error, ParseError::InvalidStatus("béta".to_owned()));
}

fn literal_panic_message(text: &str) -> String {
    let panic = std::panic::catch_unwind(|| ApiVersion::parse_literal(text)).expect_err("invalid literal must panic");

    if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        panic!("unexpected panic payload");
    }
}

#[rstest]
#[case("256", "the specified API version is invalid")]
#[case("2023-02-29", "the specified API version date '2023-02-29' is invalid")]
#[case("2024-13-01", "the specified API version date '2024-13-01' is invalid")]
#[case("1.0-2beta", "the specified API version status '2beta' is invalid")]
#[case("1.0-béta", "the specified API version status 'béta' is invalid")]
#[case("2024-01-01-rc_1", "the specified API version status 'rc_1' is invalid")]
fn literal_panics_match_parse_error_messages(#[case] text: &str, #[case] expected: &str) {
    // arrange

    // act

    // assert
    assert_eq!(ApiVersion::parse(text).unwrap_err().to_string(), expected);
    assert_eq!(literal_panic_message(text), expected);
}

#[test]
fn literal_panics_include_fragments_up_to_buffer_capacity() {
    // arrange
    let message_overhead = "the specified API version status '' is invalid".len();
    let max_fragment_len = 256 - message_overhead;
    let fragment = format!("{}!", "a".repeat(max_fragment_len - 1));
    let input = format!("1-{fragment}");

    // act
    let expected = ApiVersion::parse(&input).err().unwrap().to_string();

    // assert
    assert_eq!(expected.len(), 256);
    assert_eq!(literal_panic_message(&input), expected);
}

#[rstest]
#[case(format!("{}!", "a".repeat(256)))]
#[case("é".repeat(256))]
fn literal_panics_fall_back_when_the_fragment_exceeds_capacity(#[case] fragment: String) {
    // arrange
    let input = format!("1-{fragment}");

    // act
    let message = literal_panic_message(&input);

    // assert
    assert_eq!(message, "the specified API version status is invalid");
    assert_eq!(ApiVersion::parse(&input), Err(InvalidStatus(fragment)));
}

mod v1 {
    pub struct Type;
}

mod v1rc {
    pub struct Type;
}

mod v1_rc {
    pub struct Type;
}

mod v20180401 {
    pub struct Type;
}

mod v20180401_beta {
    pub struct Type;
}

mod v20180401beta {
    pub struct Type;
}

mod _1 {
    pub struct Type;
}

mod _1_1 {
    pub struct Type;
}

mod _0_9_beta {
    pub struct Type;
}

mod v2018_04_01 {
    pub mod endpoints {
        pub mod v2_0_rc {
            pub struct Type;
        }
    }
}

mod _20180401 {
    pub struct Type;
}

mod _20180401_beta {
    pub struct Type;
}

mod _2018_04_01 {
    pub struct Type;
}

mod _2018_04_01_beta {
    pub struct Type;
}

mod contoso {
    pub mod api {
        pub mod endpoints {
            pub struct Type;
        }
        pub mod v0_9 {
            pub mod endpoints {
                pub struct Type;
            }
        }
        pub mod v1 {
            pub mod endpoints {
                pub struct Type;

                pub mod v1 {
                    pub struct Type;
                }
            }
        }
        pub mod v1_1 {
            pub mod endpoints {
                pub struct Type;

                pub mod v1 {
                    pub struct Type;
                }
            }
        }
        pub mod v2_0 {
            pub mod endpoints {
                pub mod v2 {
                    pub struct Type;
                }
            }
        }
        pub mod v20180401 {
            pub mod endpoints {
                pub struct Type;

                pub mod v1 {
                    pub struct Type;
                }
            }
        }
        pub mod v2018_04_01 {
            pub mod endpoints {
                pub struct Type;

                pub mod v2_0_beta {
                    pub struct Type;
                }
            }
        }
        pub mod v20180401_beta {
            pub mod endpoints {
                pub struct Type;
            }
        }
        pub mod v2018_04_01_beta {
            pub mod endpoints {
                pub struct Type;
            }
        }
        pub mod _1 {
            pub mod endpoints {
                pub struct Type;
            }
        }
        pub mod _2018_04_01 {
            pub mod endpoints {
                pub struct Type;
            }
        }
        pub mod _2018_04_01_beta {
            pub mod endpoints {
                pub struct Type;
            }
        }
    }
}

mod my_restaurant {
    pub mod vegetarian {
        pub mod food {
            pub mod v1_1 {
                pub mod endpoints {
                    pub struct Type;
                }
            }
        }
    }
}

mod versioning_sample {
    pub mod v5 {
        pub mod endpoints {
            pub struct Type;
        }
    }
}

#[rstest]
#[case(type_name::<v1::Type>(), api_version!(1))]
#[case(type_name::<v1rc::Type>(), api_version!(1-rc))]
#[case(type_name::<v1_rc::Type>(), api_version!(1-rc))]
#[case(type_name::<v20180401::Type>(), api_version!(2018-04-01))]
#[case(type_name::<v20180401_beta::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<v20180401beta::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<_0_9_beta::Type>(), api_version!(0.9-beta))]
#[case(type_name::<_1::Type>(), api_version!(1))]
#[case(type_name::<_1_1::Type>(), api_version!(1.1))]
#[case(type_name::<_20180401::Type>(), api_version!(2018-04-01))]
#[case(type_name::<_20180401_beta::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<_2018_04_01::Type>(), api_version!(2018-04-01))]
#[case(type_name::<_2018_04_01_beta::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<contoso::api::v0_9::endpoints::Type>(), api_version!(0.9))]
#[case(type_name::<contoso::api::v1::endpoints::Type>(), api_version!(1))]
#[case(type_name::<contoso::api::v1_1::endpoints::Type>(), api_version!(1.1))]
#[case(type_name::<contoso::api::v20180401::endpoints::Type>(), api_version!(2018-04-01))]
#[case(type_name::<contoso::api::v20180401_beta::endpoints::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<contoso::api::v2018_04_01::endpoints::Type>(), api_version!(2018-04-01))]
#[case(type_name::<contoso::api::v2018_04_01_beta::endpoints::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<contoso::api::_1::endpoints::Type>(), api_version!(1))]
#[case(type_name::<contoso::api::_2018_04_01::endpoints::Type>(), api_version!(2018-04-01))]
#[case(type_name::<contoso::api::_2018_04_01_beta::endpoints::Type>(), api_version!(2018-04-01-beta))]
#[case(type_name::<my_restaurant::vegetarian::food::v1_1::endpoints::Type>(), api_version!(1.1))]
#[case(type_name::<versioning_sample::v5::endpoints::Type>(), api_version!(5))]
fn parse_should_return_single_version(#[case] type_name: &str, #[case] expected: ApiVersion) {
    // arrange

    // act
    let actual = ApiVersion::from_type_name(type_name);

    // assert
    assert_eq!(actual, [expected]);
}

#[rstest]
#[case(type_name::<contoso::api::v1::endpoints::v1::Type>(), &[api_version!(1), api_version!(1)])]
#[case(type_name::<contoso::api::v1_1::endpoints::v1::Type>(), &[api_version!(1.1), api_version!(1)])]
#[case(type_name::<contoso::api::v2_0::endpoints::v2::Type>(), &[api_version!(2.0), api_version!(2)])]
#[case(type_name::<contoso::api::v20180401::endpoints::v1::Type>(), &[api_version!(2018-04-01), api_version!(1)])]
#[case(type_name::<contoso::api::v2018_04_01::endpoints::v2_0_beta::Type>(), &[api_version!(2018-04-01), api_version!(2.0-beta)])]
#[case(type_name::<v2018_04_01::endpoints::v2_0_rc::Type>(), &[api_version!(2018-04-01), api_version!(2.0-rc)])]
fn parse_should_return_multiple_versions(#[case] type_name: &str, #[case] expected: &[ApiVersion]) {
    // arrange

    // act
    let actual = ApiVersion::from_type_name(type_name);

    // assert
    assert_eq!(actual, expected);
}

#[rstest]
#[case("contoso::v::Type")]
#[case("contoso::_::Type")]
#[case("::contoso::Type")]
fn parse_should_ignore_segments_without_version(#[case] type_name: &str) {
    // arrange

    // act
    let versions = ApiVersion::from_type_name(type_name);

    // assert
    assert!(versions.is_empty());
}

#[test]
fn parse_should_return_no_versions() {
    // arrange

    // act
    let versions = ApiVersion::from_type::<contoso::api::endpoints::Type>();

    // assert
    assert!(versions.is_empty());
}

#[rstest]
#[case("my_api::orders::hello_v1", api_version!(1))]
#[case("my_api::orders::get_v1_1", api_version!(1.1))]
#[case("my_api::orders::get_v1beta", api_version!(1-beta))]
#[case("my_api::orders::get_v1_rc", api_version!(1-rc))]
#[case("my_api::orders::get_v2018_04_01_beta", api_version!(2018-04-01-beta))]
#[case("my_api::orders::remove_vehicle_v2", api_version!(2))]
#[case("my_api::orders::v3", api_version!(3))]
#[case("my_api::v1::orders::get", api_version!(1))]
#[case("my_api::v1::orders::get<my_api::v2::State>", api_version!(1))]
#[case("my_api::v1::orders::{{closure}}", api_version!(1))]
fn parse_should_return_version_from_item_path(#[case] type_name: &str, #[case] expected: ApiVersion) {
    // arrange

    // act
    let actual = ApiVersion::from_type_name(type_name);

    // assert
    assert_eq!(actual, [expected]);
}

#[rstest]
#[case("my_api::orders::get_page_2")]
#[case("my_api::orders::get_v1_by_id")]
#[case("my_api::orders::getv1")]
#[case("my_api::orders::version")]
#[case("my_api::orders::Type_V1")]
#[case("my_api::v1_by_id::orders::get")]
#[case("axum::handler::Layered<my_api::v1::orders::get>")]
fn parse_should_ignore_item_path_without_version(#[case] type_name: &str) {
    // arrange

    // act
    let versions = ApiVersion::from_type_name(type_name);

    // assert
    assert!(versions.is_empty());
}

mod inferred {
    pub mod v1 {
        pub mod orders {
            pub fn get() {}
            pub fn get_v1() {}
            pub fn get_v2() {}
            pub async fn post() {}
            pub struct Unit;
        }
    }

    pub mod orders {
        pub fn get() {}
        pub fn hello_v2() {}
        pub fn generic_v3<T>() {}
    }
}

#[rstest]
#[case(ApiVersion::from_val(&inferred::v1::orders::get), api_version!(1))]
#[case(ApiVersion::from_val(&inferred::v1::orders::post), api_version!(1))]
#[case(ApiVersion::from_val(&inferred::orders::hello_v2), api_version!(2))]
#[case(ApiVersion::from_val(&inferred::orders::generic_v3::<inferred::v1::orders::Unit>), api_version!(3))]
fn from_val_should_return_version_from_function(
    #[case] actual: Vec<ApiVersion<'static>>,
    #[case] expected: ApiVersion,
) {
    // arrange

    // act

    // assert
    assert_eq!(actual, [expected]);
}

#[test]
fn from_val_should_return_all_versions() {
    // arrange

    // act
    let versions = ApiVersion::from_val(&inferred::v1::orders::get_v2);

    // assert
    assert_eq!(versions, [api_version!(1), api_version!(2)]);
}

#[test]
fn infer_from_val_should_return_single_version() {
    // arrange

    // act
    let version = ApiVersion::infer_from_val(&inferred::orders::hello_v2);

    // assert
    assert_eq!(version, api_version!(2));
}

#[test]
fn infer_from_val_should_allow_duplicate_versions() {
    // arrange

    // act
    let version = ApiVersion::infer_from_val(&inferred::v1::orders::get_v1);

    // assert
    assert_eq!(version, api_version!(1));
}

#[test]
#[should_panic(expected = "An API version could not be inferred from")]
fn infer_from_val_should_panic_without_version() {
    // arrange

    // act
    let _ = ApiVersion::infer_from_val(&inferred::orders::get);

    // assert
}

#[test]
#[should_panic(expected = "orders::get_v2', but found: 1.0, 2.0.")]
fn infer_from_val_should_panic_with_ambiguous_versions() {
    // arrange

    // act
    let _ = ApiVersion::infer_from_val(&inferred::v1::orders::get_v2);

    // assert
}

#[test]
fn infer_from_type_should_return_single_version() {
    // arrange

    // act
    let version = ApiVersion::infer_from_type::<contoso::api::v1_1::endpoints::Type>();

    // assert
    assert_eq!(version, api_version!(1.1));
}

#[test]
#[should_panic(expected = "An API version could not be inferred from")]
fn infer_from_type_should_panic_without_version() {
    // arrange

    // act
    let _ = ApiVersion::infer_from_type::<contoso::api::endpoints::Type>();

    // assert
}

#[rstest]
#[case("my_api::v1", api_version!(1))]
#[case("my_api::v1::orders", api_version!(1))]
#[case("my_api::v1::orders::v1", api_version!(1))]
#[case("my_api::orders::v2018_04_01", api_version!(2018-04-01))]
#[case("my_api::orders::_2018_04_01", api_version!(2018-04-01))]
#[case("my_api::orders::hello_v2", api_version!(2))]
fn infer_from_path_should_return_single_version(#[case] path: &str, #[case] expected: ApiVersion) {
    // arrange

    // act
    let version = ApiVersion::infer_from_path(path);

    // assert
    assert_eq!(version, expected);
}

#[test]
#[should_panic(expected = "An API version could not be inferred from 'my_api::orders'.")]
fn infer_from_path_should_panic_without_version() {
    // arrange

    // act
    let _ = ApiVersion::infer_from_path("my_api::orders");

    // assert
}

#[test]
#[should_panic(expected = "Exactly one API version must be inferred from 'my_api::v1::v2', but found: 1.0, 2.0.")]
fn infer_from_path_should_panic_with_ambiguous_versions() {
    // arrange

    // act
    let _ = ApiVersion::infer_from_path("my_api::v1::v2");

    // assert
}

#[test]
fn api_version_should_infer_from_module() {
    // arrange

    // act
    let relative = api_version!(use inferred::v1::orders);
    let qualified = api_version!(use crate::version::parsing::tests::inferred::v1);

    // assert
    assert_eq!(relative, api_version!(1));
    assert_eq!(qualified, api_version!(1));
}

#[test]
fn api_version_should_infer_from_function() {
    // arrange

    // act
    let module = api_version!(use inferred::v1::orders::get);
    let suffix = api_version!(use inferred::orders::hello_v2);
    let generic = api_version!(use inferred::orders::generic_v3);

    // assert
    assert_eq!(module, api_version!(1));
    assert_eq!(suffix, api_version!(2));
    assert_eq!(generic, api_version!(3));
}

// parses all text by recursively parsing it again
struct Recursive;

impl CustomFormat for Recursive {
    fn parse<'a>(&self, text: &'a str) -> std::result::Result<ApiVersion<'a>, ParseError> {
        ApiVersion::parse_with(text, Some(self))
    }

    fn fmt(&self, _version: &ApiVersion<'_>, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }

    fn parse_path<'a>(&self, _segment: &'a str) -> Option<ApiVersion<'a>> {
        ApiVersion::from_type_name_with("contoso::v1::Type", Some(self)).pop()
    }
}

#[rstest]
#[case("r42", ApiVersion::custom(42, None))]
#[case("r7-beta", ApiVersion::custom(7, Some("beta".into())))]
#[case("1.0", api_version!(1.0))]
#[case("2026-10-01-rc.1", api_version!(2026-10-01-rc.1))]
fn parse_should_use_custom_format(#[case] text: &str, #[case] expected: ApiVersion) -> Result {
    // arrange

    // act
    let version = ApiVersion::parse_with(text, Some(&Revision))?;

    // assert
    assert_eq!(version, expected);
    Ok(())
}

#[rstest]
#[case("1.0")]
#[case("2026-10-01")]
#[case("not a version")]
fn parse_should_always_use_custom_format(#[case] text: &str) -> Result {
    // arrange

    // act
    let version = ApiVersion::parse_with(text, Some(&Opaque))?;

    // assert
    assert_eq!(version, ApiVersion::custom(u32::MAX, None));
    Ok(())
}

#[test]
fn parse_should_return_custom_error_from_custom_format() {
    // arrange
    let expected = Custom("'rx' is not a revision".into());

    // act
    let error = ApiVersion::parse_with("rx", Some(&Revision)).unwrap_err();

    // assert
    assert_eq!(error.to_string(), "'rx' is not a revision");
    assert_eq!(error, expected);
}

#[test]
fn parse_should_return_invalid_format_without_custom_format() {
    // arrange

    // act
    let error = ApiVersion::parse_with("r42", None).unwrap_err();

    // assert
    assert_eq!(error, InvalidFormat);
}

#[test]
fn parse_should_fail_when_custom_format_recursively_parses() {
    // arrange

    // act
    let error = ApiVersion::parse_with("1.0", Some(&Recursive)).unwrap_err();

    // assert
    assert_eq!(error, Recursion);
}

#[test]
fn parse_should_allow_custom_format_after_recursion_failure() -> Result {
    // arrange
    let _ = ApiVersion::parse_with("1.0", Some(&Recursive));

    // act
    let version = ApiVersion::parse_with("r1", Some(&Revision))?;

    // assert
    assert_eq!(version, ApiVersion::custom(1, None));
    Ok(())
}

#[rstest]
#[case(Some(&Revision as &dyn CustomFormat), &[ApiVersion::custom(3, None), api_version!(1)])]
#[case(Some(&Opaque as &dyn CustomFormat), &[api_version!(1)])]
#[case(None, &[api_version!(1)])]
fn from_type_name_should_use_custom_format(
    #[case] custom: Option<&dyn CustomFormat>,
    #[case] expected: &[ApiVersion<'static>],
) {
    // arrange
    let type_name = "contoso::api::rev3::endpoints::v1::Type";

    // act
    let versions = ApiVersion::from_type_name_with(type_name, custom);

    // assert
    assert_eq!(versions, expected);
}

#[test]
#[should_panic(expected = "the custom API version format recursively parsed a type path")]
fn from_type_name_should_panic_when_custom_format_recursively_parses() {
    // arrange
    let type_name = "contoso::v1::Type";

    // act
    let _ = ApiVersion::from_type_name_with(type_name, Some(&Recursive));

    // assert
}

#[rstest]
#[case("1.0", api_version!(1.0))]
#[case("2026-10-01-rc.1", api_version!(2026-10-01-rc.1))]
fn builtin_parse_should_parse_builtin_formats(#[case] text: &str, #[case] expected: ApiVersion) -> Result {
    // arrange

    // act
    let version = builtin::parse(text)?;

    // assert
    assert_eq!(version, expected);
    Ok(())
}

#[rstest]
#[case("v1", Some(api_version!(1)))]
#[case("rev1", None)]
fn builtin_parse_path_should_parse_builtin_formats(#[case] segment: &str, #[case] expected: Option<ApiVersion>) {
    // arrange

    // act
    let version = builtin::parse_path(segment);

    // assert
    assert_eq!(version, expected);
}
