use api_versioning::{ApiVersion, Model, Options, ReportHeaders, ReportOptions, api_version};
use api_versioning_axum::{EndpointMetadata, Router, VersionedApi};
use axum::{body::Body, extract::Path};
use http::{HeaderMap, Method, Request, StatusCode};
use rstest::rstest;
use std::panic::{self, AssertUnwindSafe};
use tower::ServiceExt;

mod common;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

async fn get_orders() -> &'static str {
    "get_orders"
}

async fn new_order() -> &'static str {
    "new_order"
}

async fn get_orders_v2() -> &'static str {
    "get_orders_v2"
}

async fn new_order_v2() -> &'static str {
    "new_order_v2"
}

async fn get_order(Path(_id): Path<u32>) -> &'static str {
    "get_order"
}

async fn get_order_v2(Path(_id): Path<u32>) -> &'static str {
    "get_order_v2"
}

async fn cancel_order(Path(_id): Path<u32>) -> &'static str {
    "cancel_order"
}

async fn get_people() -> &'static str {
    "get_people"
}

async fn health() -> &'static str {
    "health"
}

async fn metadata(metadata: EndpointMetadata) -> String {
    format!("{:?}", metadata.as_parts().1)
}

fn app(options: Options) -> axum::Router {
    Router::new(options)
        .api("/orders", |api| {
            api.name("Orders")
                .deprecates(api_version!(0.9))
                .has(api_version!(1.0))
                .has(api_version!(2.0))
                .route("/", |e| {
                    e.get(get_orders)
                        .post(new_order)
                        .map(api_version!(2.0), |e| e.get(get_orders_v2).post(new_order_v2))
                })
                .route("/{id}", |e| {
                    e.map(api_version!(1.0), |e| e.get(get_order))
                        .map(api_version!(2.0), |e| e.get(get_order_v2))
                        .neutral(|e| e.delete(cancel_order))
                })
        })
        .api("/people", |api| {
            api.name("People")
                .has(api_version!(3.0))
                .route("/", |e| e.get(get_people))
        })
        .api("/health", |api| api.neutral().route("/", |e| e.get(health)))
        .into()
}

fn build(api: VersionedApi) -> axum::Router {
    Router::default().api("/", |_| api).into()
}

async fn send(app: axum::Router, method: Method, uri: &str) -> Result<(StatusCode, HeaderMap, String)> {
    let request = Request::builder().method(method).uri(uri).body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, headers, String::from_utf8(body.to_vec())?))
}

#[rstest]
#[case(Method::GET, "/orders?api-version=0.9", "get_orders")]
#[case(Method::GET, "/orders?api-version=1.0", "get_orders")]
#[case(Method::GET, "/orders?api-version=2.0", "get_orders_v2")]
#[case(Method::POST, "/orders?api-version=0.9", "new_order")]
#[case(Method::POST, "/orders?api-version=1.0", "new_order")]
#[case(Method::POST, "/orders?api-version=2.0", "new_order_v2")]
#[case(Method::GET, "/orders/42?api-version=1.0", "get_order")]
#[case(Method::GET, "/orders/42?api-version=2.0", "get_order_v2")]
#[case(Method::DELETE, "/orders/42", "cancel_order")]
#[case(Method::DELETE, "/orders/42?api-version=1.0", "cancel_order")]
#[case(Method::DELETE, "/orders/42?api-version=3.0", "cancel_order")]
#[case(Method::GET, "/people?api-version=3.0", "get_people")]
#[case(Method::GET, "/health", "health")]
#[case(Method::GET, "/health?api-version=0.9", "health")]
#[case(Method::GET, "/health?api-version=3.0", "health")]
#[tokio::test]
async fn request_should_dispatch_to_handler_for_requested_version(
    #[case] method: Method,
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let app = app(Options::default());

    // act
    let (status, _, body) = send(app, method, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[rstest]
#[case(Method::GET, "/orders", "An API version is required, but was not specified.")]
#[case(Method::GET, "/orders?api-version=3.0", &common::unsupported("/orders", "3.0"))]
#[case(Method::GET, "/orders/42?api-version=0.9", &common::unsupported("/orders/42", "0.9"))]
#[case(Method::DELETE, "/orders/42?api-version=4.0", &common::unsupported("/orders/42", "4.0"))]
#[case(Method::GET, "/health?api-version=4.0", &common::unsupported("/health", "4.0"))]
#[case(Method::GET, "/orders?api-version=abc", &common::unsupported("/orders", "abc"))]
#[case(Method::GET, "/orders?api-version=1.0&api-version=2.0", &common::ambiguous("1.0, 2.0"))]
#[case(
    Method::GET,
    "/orders?api-version=10.0&api-version=2.0&api-version=1.0",
    &common::ambiguous("1.0, 2.0, 10.0")
)]
#[case(Method::GET, "/orders?api-version=2026-10-01&api-version=2.0", &common::ambiguous("2.0, 2026-10-01"))]
#[case(Method::GET, "/orders?api-version=1.0&api-version=1.0-beta", &common::ambiguous("1.0-beta, 1.0"))]
#[case(Method::GET, "/orders?api-version=2&api-version=1.0&api-version=2.0", &common::ambiguous("1.0, 2"))]
#[tokio::test]
async fn request_should_return_400_when_version_cannot_be_matched(
    #[case] method: Method,
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let app = app(Options::default());

    // act
    let (status, _, body) = send(app, method, uri).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(common::detail(&body)?, expected);
    Ok(())
}

#[tokio::test]
async fn request_should_assume_default_version_when_unspecified() -> Result {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: true,
        ..Default::default()
    };
    let app = app(options);

    // act
    let (status, _, body) = send(app, Method::GET, "/orders").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_orders");
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0", StatusCode::BAD_REQUEST)]
#[case("/orders?api-version=2.0", StatusCode::OK)]
#[tokio::test]
async fn request_should_use_configured_default_version_for_unversioned_api(
    #[case] uri: &str,
    #[case] expected: StatusCode,
) -> Result {
    // arrange
    let options = Options {
        default_version: api_version!(2.0),
        ..Default::default()
    };
    let app: axum::Router = Router::new(options)
        .api("/orders", |api| api.route("/", |e| e.get(get_orders)))
        .into();

    // act
    let (status, _, _) = send(app, Method::GET, uri).await?;

    // assert
    assert_eq!(status, expected);
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0", "get_orders")]
#[case("/orders?api-version=2.0", "get_orders")]
#[case("/orders?api-version=3.0", "get_orders_v2")]
#[tokio::test]
async fn request_should_dispatch_to_handler_mapped_to_many_versions(
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .api("/orders", |api| {
            api.has([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
                .route("/", |e| {
                    e.map([api_version!(1.0), api_version!(2.0)], |e| e.get(get_orders))
                        .map(api_version!(3.0), |e| e.get(get_orders_v2))
                })
        })
        .into();

    // act
    let (status, _, body) = send(app, Method::GET, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0")]
#[case("/orders?api-version=2.0")]
#[case("/orders/42?api-version=2.0")]
#[tokio::test]
async fn request_should_report_api_versions(#[case] uri: &str) -> Result {
    // arrange
    let options = Options {
        report_versions: ReportOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let app = app(options);

    // act
    let (_, headers, _) = send(app, Method::GET, uri).await?;

    // assert
    assert_eq!(headers["api-supported-versions"], "1.0, 2.0");
    assert_eq!(headers["api-deprecated-versions"], "0.9");
    Ok(())
}

#[tokio::test]
async fn request_should_report_api_versions_with_configured_headers() -> Result {
    // arrange
    let options = Options {
        report_versions: ReportOptions {
            enabled: true,
            header: ReportHeaders {
                supported: "x-supported-versions",
                deprecated: "x-deprecated-versions",
            },
        },
        ..Default::default()
    };
    let app = app(options);

    // act
    let (_, headers, _) = send(app, Method::GET, "/orders?api-version=1.0").await?;

    // assert
    assert_eq!(headers["x-supported-versions"], "1.0, 2.0");
    assert_eq!(headers["x-deprecated-versions"], "0.9");
    assert!(!headers.contains_key("api-supported-versions"));
    assert!(!headers.contains_key("api-deprecated-versions"));
    Ok(())
}

#[tokio::test]
async fn request_should_report_api_versions_when_requested_by_api() -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .api("/orders", |api| {
            api.has(api_version!(1.0))
                .report_api_versions()
                .route("/", |e| e.get(get_orders))
        })
        .into();

    // act
    let (_, headers, _) = send(app, Method::GET, "/orders?api-version=1.0").await?;

    // assert
    assert_eq!(headers["api-supported-versions"], "1.0");
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0", StatusCode::OK)]
#[case("/orders?api-version=2.0", StatusCode::BAD_REQUEST)]
#[case("/orders?api-version=0.9", StatusCode::BAD_REQUEST)]
#[tokio::test]
async fn request_should_not_route_to_advertised_versions(#[case] uri: &str, #[case] expected: StatusCode) -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .api("/orders", |api| {
            api.has(api_version!(1.0))
                .advertises(api_version!(2.0))
                .advertises_deprecated(api_version!(0.9))
                .report_api_versions()
                .route("/", |e| e.get(get_orders))
        })
        .into();

    // act
    let (status, headers, _) = send(app, Method::GET, uri).await?;

    // assert
    assert_eq!(status, expected);
    assert_eq!(headers["api-supported-versions"], "1.0, 2.0");
    assert_eq!(headers["api-deprecated-versions"], "0.9");
    Ok(())
}

#[rstest]
#[case(Method::GET, "/orders?api-version=1.0", Model::builder()
    .add_supported([api_version!(1.0), api_version!(2.0)])
    .add_deprecated(api_version!(0.9))
    .add_advertised(api_version!(3.0))
    .build())]
#[case(Method::GET, "/orders?api-version=2.0", Model::builder()
    .add_declared(api_version!(2.0))
    .add_supported([api_version!(1.0), api_version!(2.0)])
    .add_deprecated(api_version!(0.9))
    .add_advertised(api_version!(3.0))
    .build())]
#[case(Method::DELETE, "/orders/42", Model::neutral())]
#[tokio::test]
async fn request_should_provide_endpoint_metadata(
    #[case] method: Method,
    #[case] uri: &str,
    #[case] expected: Model,
) -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .api("/orders", |api| {
            api.name("Orders")
                .deprecates(api_version!(0.9))
                .has([api_version!(1.0), api_version!(2.0)])
                .advertises(api_version!(3.0))
                .route("/", |e| e.get(metadata).map(api_version!(2.0), |e| e.get(metadata)))
                .route("/{id}", |e| e.neutral(|e| e.delete(metadata)))
        })
        .into();

    // act
    let (_, _, body) = send(app, method, uri).await?;

    // assert
    assert_eq!(body, format!("{expected:?}"));
    Ok(())
}

#[rstest]
#[case(VersionedApi::default().name("Orders"), "the Orders API")]
#[case(VersionedApi::default(), "the API")]
fn split_should_panic_when_mapped_version_is_not_declared(#[case] api: VersionedApi, #[case] name: &str) {
    // arrange
    let api = api
        .has(api_version!(1.0))
        .route("/", |e| e.map(api_version!(3.0), |e| e.get(get_orders)));
    let expected = format!("GET / maps to API version 3.0, which is not declared by {name}.");

    // act
    let error = panic::catch_unwind(AssertUnwindSafe(|| build(api))).err().unwrap();

    // assert
    assert_eq!(error.downcast_ref::<String>(), Some(&expected));
}

fn orders() -> VersionedApi {
    VersionedApi::default().name("Orders").has(api_version!(1.0))
}

#[rstest]
#[case(orders().route("/", |e| e.get(get_orders).get(get_orders_v2)))]
#[case(orders().route("/", |e| e.get(get_orders)).route("/", |e| e.get(get_orders_v2)))]
#[case(orders().route("/", |e| {
    e.map(api_version!(1.0), |e| e.get(get_orders)).map(api_version!(1.0), |e| e.get(get_orders_v2))
}))]
#[case(orders().route("/", |e| e.get(get_orders).neutral(|e| e.get(get_orders_v2))))]
#[should_panic(expected = "GET / has more than one handler for the same API versions in the Orders API.")]
fn split_should_panic_when_handlers_overlap(#[case] api: VersionedApi) {
    // arrange

    // act
    let _ = build(api);

    // assert
}

#[test]
#[should_panic(
    expected = "Options::default_version cannot be ApiVersion::NEUTRAL; the default must be a specific API version."
)]
fn new_should_panic_when_default_version_is_neutral() {
    // arrange
    let options = Options {
        default_version: ApiVersion::NEUTRAL,
        ..Default::default()
    };

    // act
    Router::<()>::new(options);

    // assert
}

#[rstest]
#[case(|api: VersionedApi| api.has(ApiVersion::NEUTRAL))]
#[case(|api: VersionedApi| api.has([api_version!(1.0), ApiVersion::NEUTRAL]))]
#[case(|api: VersionedApi| api.deprecates(ApiVersion::NEUTRAL))]
#[case(|api: VersionedApi| api.advertises(ApiVersion::NEUTRAL))]
#[case(|api: VersionedApi| api.advertises_deprecated(ApiVersion::NEUTRAL))]
#[case(|api: VersionedApi| api.route("/", |e| e.map(ApiVersion::NEUTRAL, |e| e.get(get_orders))))]
#[should_panic(
    expected = "ApiVersion::NEUTRAL cannot be declared as an API version; use neutral() to make an API or endpoint \
                version-neutral."
)]
fn builder_should_panic_when_version_is_neutral(#[case] declare: fn(VersionedApi) -> VersionedApi) {
    // arrange
    let api = VersionedApi::default();

    // act
    declare(api);

    // assert
}
