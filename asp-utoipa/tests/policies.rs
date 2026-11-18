use api_versioning::{Options, api_version};
use api_versioning_axum::{ApiDescriptions, Router};
use api_versioning_utoipa::{ApiDocument, doc, documents};
use serde_json::{Value, json};
use std::time::{Duration, UNIX_EPOCH};
use utoipa::openapi::{InfoBuilder, OpenApi, OpenApiBuilder, tag::Tag};

/// 2026-02-10
const SUNSET: u64 = 1_770_681_600;

#[utoipa::path(get, path = "", responses((status = 200)))]
async fn get_orders() {}

#[utoipa::path(get, path = "", responses((status = 200)))]
async fn get_people() {}

fn base() -> OpenApi {
    OpenApiBuilder::new()
        .info(InfoBuilder::new().title("Test").version("0.0.0"))
        .build()
}

fn json(document: &ApiDocument) -> Value {
    serde_json::to_value(document.openapi()).unwrap()
}

fn tag<'a>(document: &'a Value, name: &str) -> &'a Value {
    document["tags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tag| tag["name"] == name)
        .unwrap()
}

/// Defines the Orders API, which deprecates 1.0 and has 2.0, and the People API, which has 1.0 and 2.0.
fn apis(options: Options) -> ApiDescriptions {
    Router::<()>::new(options)
        .api("/orders", |api| {
            api.name("Orders")
                .deprecates(api_version!(1.0))
                .has(api_version!(2.0))
                .route("/", |e| e.get(doc!(get_orders)))
        })
        .api("/people", |api| {
            api.name("People")
                .has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| e.get(doc!(get_people)))
        })
        .split()
        .1
}

#[test]
fn documents_should_describe_policies_of_named_api_in_its_tag() {
    // arrange
    let mut options = Options::default();

    options
        .policies
        .sunset(("Orders", &api_version!(1.0)))
        .as_of(UNIX_EPOCH + Duration::from_secs(SUNSET))
        .link("http://my.api.com/policies/orders/sunset.html")
        .title("Orders Sunset Policy")
        .media_type("text/html");

    let apis = apis(options);

    // act
    let documents = documents(&apis, base());

    // assert
    let v1 = json(&documents[0]);
    assert_eq!(
        tag(&v1, "Orders"),
        &json!({
            "name": "Orders",
            "description": "The API is deprecated. The API was sunset on 2026-02-10.\n\n### Links\n\n\
                            - [Orders Sunset Policy](http://my.api.com/policies/orders/sunset.html)",
            "x-api-versioning": {
                "links": [{
                    "title": "Orders Sunset Policy",
                    "type": "text/html",
                    "rel": "sunset",
                    "url": "http://my.api.com/policies/orders/sunset.html"
                }]
            }
        })
    );
    assert_eq!(tag(&v1, "People"), &json!({ "name": "People" }));
    assert_eq!(v1["info"].get("description"), None);
    assert_eq!(v1.get("x-api-versioning"), None);
    assert_eq!(tag(&json(&documents[1]), "Orders"), &json!({ "name": "Orders" }));
}

#[test]
fn documents_should_describe_policy_for_every_version_of_named_api() {
    // arrange
    let mut options = Options::default();

    options
        .policies
        .deprecate("People")
        .link("http://my.api.com/policies/people.html")
        .media_type("text/html");

    let apis = apis(options);

    // act
    let documents = documents(&apis, base());

    // assert
    for document in &documents {
        let people = json(document);
        let people = tag(&people, "People");
        assert_eq!(
            people["description"],
            "The API is deprecated.\n\n### Links\n\n- http://my.api.com/policies/people.html"
        );
        assert_eq!(people["x-api-versioning"]["links"][0]["rel"], "deprecation");
    }
}

#[test]
fn documents_should_not_describe_deprecated_api_in_tag_when_api_version_is_deprecated() {
    // arrange
    let (_, apis): (axum::Router, _) = Router::new(Options::default())
        .api("/orders", |api| {
            api.name("Orders")
                .deprecates(api_version!(1.0))
                .has(api_version!(2.0))
                .route("/", |e| e.get(doc!(get_orders)))
        })
        .split();

    // act
    let documents = documents(&apis, base());

    // assert
    let v1 = json(&documents[0]);
    assert!(documents[0].deprecated());
    assert_eq!(v1["info"]["description"], "The API is deprecated.");
    assert_eq!(tag(&v1, "Orders"), &json!({ "name": "Orders" }));
}

#[test]
fn documents_should_append_notices_to_tag_defined_by_base_document() {
    // arrange
    let mut base = base();
    let mut existing = Tag::new("Orders");

    existing.description = Some("Manages orders".into());
    base.tags = Some(vec![existing]);

    let apis = apis(Options::default());

    // act
    let documents = documents(&apis, base);

    // assert
    let v1 = json(&documents[0]);
    assert_eq!(
        tag(&v1, "Orders")["description"],
        "Manages orders. The API is deprecated."
    );
    assert_eq!(v1["tags"].as_array().unwrap().len(), 2);
}
