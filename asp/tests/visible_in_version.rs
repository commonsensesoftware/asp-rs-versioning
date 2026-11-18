#![cfg(feature = "macros")]

use api_versioning::{ApiVersion, api_version, visibility::scope, visible_in_version};
use rstest::rstest;
use serde::{Deserialize, Serialize};

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[visible_in_version]
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
struct Order {
    id: u32,
    #[range(2.0..=3.0, 4.0..)]
    customer: Option<String>,
    #[range(..2.0)]
    legacy: Option<String>,
    #[range(2.1-preview.1..)]
    preview: Option<String>,
}

fn order(customer: bool, legacy: bool, preview: bool) -> Order {
    Order {
        id: 1,
        customer: customer.then(|| "Bill".into()),
        legacy: legacy.then(|| "old".into()),
        preview: preview.then(|| "new".into()),
    }
}

#[rstest]
#[case(None, r#"{"id":1,"customer":"Bill","legacy":"old","preview":"new"}"#)]
#[case(Some(api_version!(1.0)), r#"{"id":1,"legacy":"old"}"#)]
#[case(Some(api_version!(2.0)), r#"{"id":1,"customer":"Bill"}"#)]
#[case(Some(api_version!(2.1-alpha)), r#"{"id":1,"customer":"Bill"}"#)]
#[case(Some(api_version!(2.1-preview.1)), r#"{"id":1,"customer":"Bill","preview":"new"}"#)]
#[case(Some(api_version!(3.1)), r#"{"id":1,"preview":"new"}"#)]
#[case(Some(api_version!(4.0)), r#"{"id":1,"customer":"Bill","preview":"new"}"#)]
fn serialize_should_only_write_members_in_range(
    #[case] version: Option<ApiVersion<'static>>,
    #[case] expected: &str,
) -> Result {
    // arrange
    let order = order(true, true, true);

    // act
    let json = scope(version, || serde_json::to_string(&order))?;

    // assert
    assert_eq!(json, expected);
    Ok(())
}

#[rstest]
#[case(api_version!(1.0), r#"{"id":1,"legacy":"old"}"#, order(false, true, false))]
#[case(api_version!(2.0), r#"{"id":1,"customer":"Bill"}"#, order(true, false, false))]
#[case(api_version!(2.0), r#"{"id":1}"#, order(false, false, false))]
fn deserialize_should_read_members_in_range(
    #[case] version: ApiVersion<'static>,
    #[case] json: &str,
    #[case] expected: Order,
) -> Result {
    // arrange

    // act
    let actual: Order = scope(Some(version), || serde_json::from_str(json))?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[rstest]
#[case(api_version!(1.0), r#"{"id":1,"customer":"Bill"}"#)]
#[case(api_version!(2.0), r#"{"id":1,"legacy":null}"#)]
#[case(api_version!(3.1), r#"{"id":1,"customer":"Bill"}"#)]
fn deserialize_should_reject_members_out_of_range(#[case] version: ApiVersion<'static>, #[case] json: &str) {
    // arrange

    // act
    let result = scope(Some(version.clone()), || serde_json::from_str::<Order>(json));

    // assert
    let error = result.unwrap_err().to_string();
    assert!(
        error.starts_with(&format!("not supported in API version {version}")),
        "{error}"
    );
}

#[test]
fn attribute_should_support_date_ranges_raw_identifiers_and_generics() -> Result {
    // arrange
    #[visible_in_version]
    #[derive(Serialize)]
    struct Page<T> {
        items: Vec<T>,
        #[range(2026-10-01..)]
        r#type: &'static str,
    }

    let page = Page {
        items: vec![1, 2],
        r#type: "page",
    };

    // act
    let before = scope(Some(api_version!(2026-01-01)), || serde_json::to_string(&page))?;
    let after = scope(Some(api_version!(2026-10-01)), || serde_json::to_string(&page))?;

    // assert
    assert_eq!(before, r#"{"items":[1,2]}"#);
    assert_eq!(after, r#"{"items":[1,2],"type":"page"}"#);
    Ok(())
}

#[test]
fn attribute_should_preserve_existing_serde_attributes() -> Result {
    // arrange
    #[visible_in_version(crate = "::api_versioning")]
    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Customer {
        #[range(2.0..)]
        #[serde(default = "unknown", rename = "fullName")]
        display_name: String,
    }

    fn unknown() -> String {
        "unknown".into()
    }

    let customer = Customer {
        display_name: "Bill".into(),
    };

    // act
    let json = scope(Some(api_version!(2.0)), || serde_json::to_string(&customer))?;
    let absent: Customer = scope(Some(api_version!(1.0)), || serde_json::from_str("{}"))?;

    // assert
    assert_eq!(json, r#"{"fullName":"Bill"}"#);
    assert_eq!(absent.display_name, "unknown");
    Ok(())
}
