use api_versioning::{
    ApiVersion, Options, api_version,
    prelude::*,
    readers::{HeaderReader, MediaTypeReader, QueryStringReader, UrlReader},
};
use api_versioning_axum::{ApiDescriptions, Router};
use api_versioning_utoipa::{ApiDocument, DocumentOptions, doc, documents, documents_with};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use rstest::rstest;
use serde::Serialize;
use serde_json::{Value, json};
use std::panic::{self, AssertUnwindSafe};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tower::ServiceExt;
use utoipa::{
    ToSchema,
    openapi::{InfoBuilder, OpenApi, OpenApiBuilder},
};

/// 2026-01-01
const DEPRECATED: u64 = 1_767_225_600;

/// 2026-02-10
const SUNSET: u64 = 1_770_681_600;

/// 2099-01-01
const FUTURE: u64 = 4_070_908_800;

fn date(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[derive(Serialize, ToSchema)]
struct Order {
    id: u32,
}

#[derive(Serialize, ToSchema)]
struct Person {
    name: String,
}

/// Test
///
/// A test API.
#[utoipa::path(get, path = "", responses((status = 200, description = "Pass")))]
async fn get_test() {}

mod v1 {
    use super::Order;

    #[utoipa::path(get, path = "", tag = "Orders (utoipa)", responses((status = 200, body = Order)))]
    pub async fn get_orders() {}

    #[utoipa::path(post, path = "", responses((status = 201)))]
    pub async fn create_order() {}
}

mod v2 {
    use super::Person;

    #[utoipa::path(get, path = "", responses((status = 200, body = Person)))]
    pub async fn get_people() {}

    #[utoipa::path(get, path = "", responses((status = 200)))]
    pub async fn get_orders() {}
}

#[utoipa::path(get, path = "", responses((status = 200, content_type = "application/json")))]
async fn health() {}

#[utoipa::path(post, path = "", request_body(content = String, content_type = "application/json"), responses((status = 200, content_type = "application/json")))]
async fn echo() {}

async fn undocumented() {}

fn base(title: &str) -> OpenApi {
    OpenApiBuilder::new()
        .info(InfoBuilder::new().title(title).version("0.0.0"))
        .build()
}

fn json(document: &ApiDocument) -> Value {
    serde_json::to_value(document.openapi()).unwrap()
}

/// Defines the Orders API with versions 1.0 and 2.0, where 1.0 is deprecated, and a version-neutral health API.
fn apis(options: Options) -> ApiDescriptions {
    Router::<()>::new(options)
        .api("/orders", |api| {
            api.name("Orders")
                .deprecates(api_version!(1.0))
                .has(api_version!(2.0))
                .route("/", |e| {
                    e.get(doc!(v1::get_orders))
                        .map(api_version!(1.0), |e| e.post(doc!(v1::create_order)))
                        .map(api_version!(2.0), |e| e.get(doc!(v2::get_orders)))
                })
                .route("/{id}/notes", |e| e.get(undocumented))
        })
        .api("/people", |api| {
            api.has(api_version!(2.0)).route("/", |e| e.get(doc!(v2::get_people)))
        })
        .api("/health", |api| api.neutral().route("/", |e| e.get(doc!(health))))
        .split()
        .1
}

fn operations(document: &Value) -> Vec<String> {
    let mut operations = Vec::new();

    for (path, item) in document["paths"].as_object().unwrap() {
        for method in item.as_object().unwrap().keys() {
            operations.push(format!(
                "{method} {path} {}",
                item[method]["operationId"].as_str().unwrap()
            ));
        }
    }

    operations
}

#[test]
fn documents_should_describe_api_version_like_asp_net() {
    // arrange
    let mut options = Options::default();

    options
        .policies
        .deprecate(api_version!(1.0))
        .as_of(date(DEPRECATED))
        .link("http://my.api.com/policies/versions/deprecated.html")
        .title("Version Deprecation Policy")
        .media_type("text/html");
    options
        .policies
        .sunset(api_version!(1.0))
        .as_of(date(SUNSET))
        .link("http://my.api.com/policies/versions/sunset.html")
        .title("Version Sunset Policy")
        .media_type("text/html");

    let (_, apis): (axum::Router, _) = Router::new(options)
        .api("/Test", |api| {
            api.name("Test")
                .has(api_version!(1.0))
                .route("/", |e| e.get(doc!(get_test)))
        })
        .split();

    // act
    let documents = documents(&apis, base("Test API"));

    // assert
    assert_eq!(documents.len(), 1);
    assert_eq!(
        json(&documents[0]),
        json!({
            "openapi": "3.1.0",
            "info": {
                "title": "Test API | v1",
                "description": "The API was deprecated on 2026-01-01. The API was sunset on 2026-02-10.\n\n### Links\n\n\
                                - [Version Deprecation Policy](http://my.api.com/policies/versions/deprecated.html)\n\
                                - [Version Sunset Policy](http://my.api.com/policies/versions/sunset.html)",
                "version": "1.0"
            },
            "paths": {
                "/Test": {
                    "get": {
                        "tags": ["Test"],
                        "summary": "Test",
                        "description": "A test API.",
                        "operationId": "get_test",
                        "parameters": [
                            {
                                "name": "api-version",
                                "in": "query",
                                "description": "The requested API version",
                                "required": true,
                                "schema": {
                                    "type": "string",
                                    "enum": ["1.0"]
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Pass"
                            }
                        }
                    }
                }
            },
            "tags": [
                {
                    "name": "Test"
                }
            ],
            "x-api-versioning": {
                "links": [
                    {
                        "title": "Version Deprecation Policy",
                        "type": "text/html",
                        "rel": "deprecation",
                        "url": "http://my.api.com/policies/versions/deprecated.html"
                    },
                    {
                        "title": "Version Sunset Policy",
                        "type": "text/html",
                        "rel": "sunset",
                        "url": "http://my.api.com/policies/versions/sunset.html"
                    }
                ]
            }
        })
    );
}

#[test]
fn documents_should_only_include_operations_for_each_api_version() {
    // arrange
    let apis = apis(Options::default());

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let names: Vec<_> = documents.iter().map(ApiDocument::name).collect();
    assert_eq!(names, ["v1", "v2"]);
    assert_eq!(
        operations(&json(&documents[0])),
        [
            "get /health health",
            "get /orders get_orders",
            "post /orders create_order"
        ]
    );
    assert_eq!(
        operations(&json(&documents[1])),
        ["get /health health", "get /orders get_orders", "get /people get_people"]
    );
}

#[test]
fn documents_should_describe_deprecated_api_version() {
    // arrange
    let apis = apis(Options::default());

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let v1 = json(&documents[0]);
    let v2 = json(&documents[1]);
    assert!(documents[0].deprecated());
    assert!(!documents[1].deprecated());
    assert_eq!(v1["info"]["description"], "The API is deprecated.");
    assert_eq!(v1["paths"]["/orders"]["get"]["deprecated"], true);
    assert_eq!(v1["paths"]["/health"]["get"].get("deprecated"), None);
    assert_eq!(v2["info"].get("description"), None);
    assert_eq!(v2["paths"]["/orders"]["get"].get("deprecated"), None);
}

#[test]
fn documents_should_tag_operations_with_api_name_or_utoipa_tags() {
    // arrange
    let apis = apis(Options::default());

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let v1 = json(&documents[0]);
    let v2 = json(&documents[1]);
    assert_eq!(v1["paths"]["/orders"]["get"]["tags"], json!(["Orders"]));
    assert_eq!(v1["tags"], json!([{ "name": "Orders" }]));
    assert_eq!(v2["paths"]["/people"]["get"].get("tags"), None);
}

#[test]
fn documents_should_only_include_schemas_referenced_by_operations() {
    // arrange
    let apis = apis(Options::default());

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let schemas = |document: &ApiDocument| {
        let mut names: Vec<_> = json(document)["components"]["schemas"]
            .as_object()
            .map(|schemas| schemas.keys().cloned().collect())
            .unwrap_or_default();
        names.sort();
        names
    };
    assert_eq!(schemas(&documents[0]), ["Order"]);
    assert_eq!(schemas(&documents[1]), ["Person"]);
}

#[test]
fn documents_should_add_version_parameters_from_query_string_and_header() {
    // arrange
    let options = Options {
        reader: QueryStringReader::default()
            .combine(HeaderReader::new("x-api-version"))
            .into(),
        ..Default::default()
    };
    let apis = apis(options);

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let v2 = json(&documents[1]);
    assert_eq!(
        v2["paths"]["/people"]["get"]["parameters"],
        json!([
            {
                "name": "api-version",
                "in": "query",
                "description": "The requested API version",
                "required": true,
                "schema": { "type": "string", "enum": ["2.0"] }
            },
            {
                "name": "x-api-version",
                "in": "header",
                "description": "The requested API version",
                "required": false,
                "schema": { "type": "string", "enum": ["2.0"] }
            }
        ])
    );
}

#[test]
fn documents_should_not_add_version_parameters_to_version_neutral_operations() {
    // arrange
    let apis = apis(Options::default());

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    assert_eq!(json(&documents[0])["paths"]["/health"]["get"].get("parameters"), None);
}

#[test]
fn documents_should_add_optional_version_parameters_to_version_neutral_operations_when_configured() {
    // arrange
    let apis = apis(Options::default());
    let options = DocumentOptions {
        add_api_version_parameters_when_version_neutral: true,
        ..Default::default()
    };

    // act
    let documents = documents_with(&apis, base("Orders"), &options);

    // assert
    let parameter = &json(&documents[0])["paths"]["/health"]["get"]["parameters"][0];
    assert_eq!(parameter["name"], "api-version");
    assert_eq!(parameter["required"], false);
}

#[test]
fn documents_should_add_optional_version_parameter_for_assumed_default_version() {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: true,
        default_version: api_version!(2.0),
        ..Default::default()
    };
    let apis = apis(options);

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let parameter = |document: &ApiDocument| json(document)["paths"]["/orders"]["get"]["parameters"][0].clone();
    let (v1, v2) = (parameter(&documents[0]), parameter(&documents[1]));
    assert_eq!(v1["required"], true);
    assert_eq!(v1["schema"], json!({ "type": "string", "enum": ["1.0"] }));
    assert_eq!(v2["required"], false);
    assert_eq!(
        v2["schema"],
        json!({ "type": "string", "enum": ["2.0"], "default": "2.0" })
    );
}

fn url_apis(options: Options) -> ApiDescriptions {
    Router::<()>::new(Options {
        reader: UrlReader::default().combine(QueryStringReader::default()).into(),
        ..options
    })
    .api("/v{api_version}/orders", |api| {
        api.has(api_version!(1.0))
            .route("/{id}", |e| e.get(doc!(v1::get_orders)))
    })
    .api("/orders", |api| {
        api.has(api_version!(1.0))
            .route("/", |e| e.post(doc!(v1::create_order)))
    })
    .split()
    .1
}

#[test]
fn documents_should_add_version_parameter_for_url_segment_when_not_substituted() {
    // arrange
    let apis = url_apis(Options::default());
    let options = DocumentOptions {
        substitute_api_version_in_url: false,
        ..Default::default()
    };

    // act
    let documents = documents_with(&apis, base("Orders"), &options);

    // assert
    let v1 = json(&documents[0]);
    assert_eq!(
        v1["paths"]["/v{api_version}/orders/{id}"]["get"]["parameters"],
        json!([{
            "name": "api_version",
            "in": "path",
            "description": "The requested API version",
            "required": true,
            "schema": { "type": "string", "enum": ["1.0"] }
        }])
    );
    assert_eq!(v1["paths"]["/orders"]["post"]["parameters"][0]["in"], "query");
}

#[rstest]
#[case(api_version!(1.0), "/v1/orders/{id}")]
#[case(api_version!(1.1), "/v1.1/orders/{id}")]
#[case(api_version!(2.0-beta), "/v2-beta/orders/{id}")]
#[case(api_version!(2026-10-01), "/v2026-10-01/orders/{id}")]
fn documents_should_substitute_api_version_in_url_by_default(
    #[case] version: ApiVersion<'static>,
    #[case] expected: &str,
) {
    // arrange
    let (_, apis): (axum::Router, _) = Router::new(Options {
        reader: Box::new(UrlReader::default()),
        ..Default::default()
    })
    .api("/v{api_version}/orders", |api| {
        api.has(version).route("/{id}", |e| e.get(doc!(v1::get_orders)))
    })
    .split();

    // act
    let documents = documents(&apis, base("Orders"));

    // assert
    let paths = json(&documents[0])["paths"].clone();
    assert_eq!(paths.as_object().unwrap().keys().collect::<Vec<_>>(), [expected]);
    assert_eq!(paths[expected]["get"].get("parameters"), None);
}

#[rstest]
#[case(api_version!(1.0))]
#[case(api_version!(2.0-beta))]
#[tokio::test]
async fn substituted_url_should_route_to_api_version(#[case] version: ApiVersion<'static>) -> anyhow::Result<()> {
    // arrange
    let (app, apis): (axum::Router, _) = Router::new(Options {
        reader: Box::new(UrlReader::default()),
        ..Default::default()
    })
    .api("/v{api_version}/orders", |api| {
        api.has(version).route("/{id}", |e| e.get(doc!(v1::get_orders)))
    })
    .split();
    let documents = documents(&apis, base("Orders"));
    let path = json(&documents[0])["paths"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .replace("{id}", "42");
    let request = Request::get(path).body(Body::empty())?;

    // act
    let response = app.oneshot(request).await?;

    // assert
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[test]
fn documents_should_add_version_parameter_to_media_types() {
    // arrange
    let (_, apis): (axum::Router, _) = Router::new(Options {
        reader: Box::new(MediaTypeReader::default()),
        ..Default::default()
    })
    .api("/echo", |api| {
        api.has(api_version!(1.0)).route("/", |e| e.post(doc!(echo)))
    })
    .split();

    // act
    let documents = documents(&apis, base("Echo"));

    // assert
    let operation = &json(&documents[0])["paths"]["/echo"]["post"];
    assert_eq!(operation.get("parameters"), None);
    assert!(
        operation["requestBody"]["content"]
            .get("application/json; v=1.0")
            .is_some()
    );
    assert!(
        operation["responses"]["200"]["content"]
            .get("application/json; v=1.0")
            .is_some()
    );
}

#[test]
fn documents_should_describe_complete_path_of_nested_api() {
    // arrange
    let (_, apis): (axum::Router, _) = Router::default()
        .nest(
            "/tenants/{tenant}",
            Router::default().api("/files", |api| api.route("/{*path}", |e| e.get(doc!(health)))),
        )
        .split();

    // act
    let documents = documents(&apis, base("Files"));

    // assert
    assert_eq!(
        operations(&json(&documents[0])),
        ["get /tenants/{tenant}/files/{path} health"]
    );
}

#[test]
fn documents_should_describe_future_policies_and_only_render_html_links() {
    // arrange
    let mut options = Options::default();

    options
        .policies
        .deprecate(api_version!(1.0))
        .as_of(date(FUTURE))
        .link("http://my.api.com/deprecated.json")
        .media_type("application/json");
    options
        .policies
        .sunset(api_version!(1.0))
        .as_of(date(FUTURE))
        .link("http://my.api.com/sunset.html")
        .media_type("text/html");

    let (_, apis): (axum::Router, _) = Router::new(options)
        .api("/Test", |api| {
            api.has(api_version!(1.0)).route("/", |e| e.get(doc!(get_test)))
        })
        .split();

    // act
    let documents = documents(&apis, base("Test API"));

    // assert
    let v1 = json(&documents[0]);
    assert_eq!(
        v1["info"]["description"],
        "The API will be deprecated on 2099-01-01. The API will be sunset on 2099-01-01.\n\n### Links\n\n\
         - http://my.api.com/sunset.html"
    );
    assert_eq!(v1["x-api-versioning"]["links"].as_array().unwrap().len(), 2);
}

#[test]
fn documents_should_use_configured_notices_and_hide_policy_links() {
    // arrange
    let mut options = Options::default();

    options
        .policies
        .sunset(api_version!(1.0))
        .as_of(date(SUNSET))
        .link("http://my.api.com/sunset.html")
        .media_type("text/html");

    let (_, apis): (axum::Router, _) = Router::new(options)
        .api("/Test", |api| {
            api.has(api_version!(1.0)).route("/", |e| e.get(doc!(get_test)))
        })
        .split();
    let base = OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .title("Test API")
                .description(Some("Tests things"))
                .version("0.0.0"),
        )
        .build();
    let options = DocumentOptions {
        hide_policy_links: true,
        sunset_notice: Box::new(|_| Some("Gone.".into())),
        ..Default::default()
    };

    // act
    let documents = documents_with(&apis, base, &options);

    // assert
    let v1 = json(&documents[0]);
    assert_eq!(v1["info"]["description"], "Tests things. Gone.");
    assert_eq!(
        v1["x-api-versioning"]["links"][0]["url"],
        "http://my.api.com/sunset.html"
    );
}

#[test]
fn documents_should_name_documents_by_api_version() {
    // arrange
    let (_, apis): (axum::Router, _) = Router::default()
        .api("/Test", |api| {
            api.has([api_version!(1.1), api_version!(2026-10-01)])
                .route("/", |e| e.get(doc!(get_test)))
        })
        .split();

    // act
    let documents = documents(&apis, base(""));

    // assert
    let names: Vec<_> = documents
        .iter()
        .map(|document| (document.name(), document.url()))
        .collect();
    assert_eq!(
        names,
        [
            ("v1.1", "/openapi/v1.1.json"),
            ("v2026-10-01", "/openapi/v2026-10-01.json")
        ]
    );
    assert_eq!(documents[0].openapi().info.title, "");
    assert_eq!(documents[1].version(), &api_version!(2026-10-01));
}

#[test]
fn documents_should_panic_when_operation_is_declared_for_another_method() {
    // arrange
    let (_, apis): (axum::Router, _) = Router::default()
        .api("/orders", |api| api.route("/", |e| e.put(doc!(v1::create_order))))
        .split();

    // act
    let error = panic::catch_unwind(AssertUnwindSafe(|| documents(&apis, base("Orders"))))
        .err()
        .unwrap();

    // assert
    assert_eq!(
        error.downcast_ref::<String>().map(String::as_str),
        Some("PUT /orders is described by an OpenAPI operation for POST, but the methods must match.")
    );
}
