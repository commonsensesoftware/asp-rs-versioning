use api_versioning::{Options, ReportOptions, api_version};
use api_versioning_axum::{Router, VersionedApi};
use axum::body::Body;
use http::{HeaderMap, Method, Request, StatusCode};
use rstest::rstest;
use std::panic::{self, AssertUnwindSafe};
use tower::ServiceExt;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

mod v1 {
    pub mod orders {
        pub async fn get() -> &'static str {
            "v1::orders::get"
        }

        pub async fn delete() -> &'static str {
            "v1::orders::delete"
        }

        pub async fn get_v2() -> &'static str {
            "v1::orders::get_v2"
        }
    }
}

mod v2 {
    pub mod orders {
        pub async fn get() -> &'static str {
            "v2::orders::get"
        }
    }
}

mod v3 {
    pub mod orders {
        pub async fn get() -> &'static str {
            "v3::orders::get"
        }
    }
}

async fn hello_v1() -> &'static str {
    "hello_v1"
}

async fn hello_v2() -> &'static str {
    "hello_v2"
}

async fn hello() -> &'static str {
    "hello"
}

async fn create() -> &'static str {
    "create"
}

fn new_api() -> VersionedApi {
    VersionedApi::default().name("Orders")
}

fn app(api: VersionedApi) -> axum::Router {
    let options = Options {
        report_versions: ReportOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };

    Router::new(options).api("/", |_| api).into()
}

// form 1: API versions are inferred from the handlers, declared by the API, and mapped to the handlers
fn inferred(api: VersionedApi) -> axum::Router {
    app(api
        .route("/orders", |e| {
            e.infer(|h| h.get(v1::orders::get).get(v2::orders::get).delete(v1::orders::delete))
                .post(create)
        })
        .route("/hello", |e| e.infer(|h| h.get(hello_v1).get(hello_v2))))
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
#[case(Method::GET, "/orders?api-version=1.0", "v1::orders::get")]
#[case(Method::GET, "/orders?api-version=2.0", "v2::orders::get")]
#[case(Method::DELETE, "/orders?api-version=1.0", "v1::orders::delete")]
#[case(Method::GET, "/hello?api-version=1.0", "hello_v1")]
#[case(Method::GET, "/hello?api-version=2.0", "hello_v2")]
#[tokio::test]
async fn request_should_route_to_handler_with_inferred_version(
    #[case] method: Method,
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let app = inferred(new_api());

    // act
    let (status, _, body) = send(app, method, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0")]
#[case("/orders?api-version=2.0")]
#[tokio::test]
async fn request_should_route_to_derived_handler_with_inferred_version(#[case] uri: &str) -> Result {
    // arrange
    let app = inferred(new_api());

    // act
    let (status, _, body) = send(app, Method::POST, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "create");
    Ok(())
}

#[rstest]
#[case(Method::DELETE, "/orders?api-version=2.0")]
#[case(Method::GET, "/orders?api-version=3.0")]
#[tokio::test]
async fn request_should_not_route_to_handler_without_inferred_version(
    #[case] method: Method,
    #[case] uri: &str,
) -> Result {
    // arrange
    let app = inferred(new_api());

    // act
    let (status, _, _) = send(app, method, uri).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn request_should_report_inferred_versions_as_supported() -> Result {
    // arrange
    let app = inferred(new_api());

    // act
    let (_, headers, _) = send(app, Method::GET, "/orders?api-version=1.0").await?;

    // assert
    assert_eq!(headers["api-supported-versions"], "1.0, 2.0");
    assert!(!headers.contains_key("api-deprecated-versions"));
    Ok(())
}

#[tokio::test]
async fn request_should_report_inferred_version_explicitly_deprecated() -> Result {
    // arrange
    let app = inferred(new_api().deprecates(api_version!(use v1)));

    // act
    let (status, headers, body) = send(app, Method::GET, "/orders?api-version=1.0").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "v1::orders::get");
    assert_eq!(headers["api-supported-versions"], "2.0");
    assert_eq!(headers["api-deprecated-versions"], "1.0");
    Ok(())
}

// form 2: API versions are declared from an existing module and derived by the handlers
#[rstest]
#[case("/orders?api-version=3.0", StatusCode::OK)]
#[case("/orders?api-version=1.0", StatusCode::BAD_REQUEST)]
#[tokio::test]
async fn request_should_route_to_derived_handler_with_version_declared_from_module(
    #[case] uri: &str,
    #[case] expected: StatusCode,
) -> Result {
    // arrange
    let app = app(new_api()
        .has(api_version!(use v3::orders))
        .route("/orders", |e| e.get(v3::orders::get)));

    // act
    let (status, headers, _) = send(app, Method::GET, uri).await?;

    // assert
    assert_eq!(status, expected);
    assert_eq!(headers["api-supported-versions"], "3.0");
    Ok(())
}

#[test]
#[should_panic(expected = "An API version could not be inferred from 'infer::hello'.")]
fn infer_should_panic_when_handler_has_no_version() {
    // arrange

    // act
    let _ = new_api().route("/", |e| e.infer(|h| h.get(hello)));

    // assert
}

#[test]
#[should_panic(expected = "An API version could not be inferred from")]
fn infer_should_panic_when_handler_is_closure() {
    // arrange

    // act
    let _ = new_api().route("/", |e| e.infer(|h| h.get(|| async { "closure" })));

    // assert
}

#[test]
#[should_panic(
    expected = "Exactly one API version must be inferred from 'infer::v1::orders::get_v2', but found: 1.0, 2.0."
)]
fn infer_should_panic_when_handler_has_ambiguous_versions() {
    // arrange

    // act
    let _ = new_api().route("/", |e| e.infer(|h| h.get(v1::orders::get_v2)));

    // assert
}

#[test]
fn split_should_panic_when_inferred_version_is_explicitly_advertised() {
    // arrange
    let api = new_api()
        .advertises(api_version!(use v3))
        .route("/", |e| e.infer(|h| h.get(v3::orders::get)));

    // act
    let error = panic::catch_unwind(AssertUnwindSafe(|| app(api))).err().unwrap();

    // assert
    assert_eq!(
        error.downcast_ref::<String>().map(String::as_str),
        Some("GET / maps to API version 3.0, which is not declared by the Orders API.")
    );
}
