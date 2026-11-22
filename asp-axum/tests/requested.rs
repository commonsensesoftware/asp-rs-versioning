use api_versioning::{
    Options, api_version,
    prelude::*,
    readers::{HeaderReader, UrlReader},
};
use api_versioning_axum::{RequestedApiVersion, Router};
use axum::{Extension, body::Body};
use http::{Request, StatusCode};
use rstest::rstest;
use tower::ServiceExt;

mod common;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn format(requested: Option<RequestedApiVersion>) -> String {
    match requested {
        Some(requested) => format!("{:?} {}", requested.raw(), requested.version()),
        None => "none".into(),
    }
}

async fn optional(requested: Option<RequestedApiVersion>) -> String {
    format(requested)
}

async fn required(requested: RequestedApiVersion) -> String {
    format(Some(requested))
}

async fn extension(Extension(requested): Extension<RequestedApiVersion>) -> String {
    format(Some(requested))
}

fn orders(options: Options) -> axum::Router {
    let versions = [
        api_version!(1.0),
        api_version!(2.0-beta),
        api_version!(2026-10-01),
        api_version!(2026-10-01-a.very.long.preview.status),
    ];

    Router::new(options)
        .api("/orders", |api| {
            api.has(versions.clone())
                .route("/optional", |e| e.get(optional))
                .route("/required", |e| e.get(required))
                .route("/extension", |e| e.get(extension))
        })
        .api("/v{api_version}/orders", |api| {
            api.has(versions).route("/", |e| e.get(optional))
        })
        .api("/health", |api| {
            api.neutral()
                .route("/optional", |e| e.get(optional))
                .route("/required", |e| e.get(required))
        })
        .into()
}

async fn send(app: axum::Router, request: Request<Body>) -> Result<(StatusCode, String)> {
    let response = app.oneshot(request).await?;
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, String::from_utf8(body.to_vec())?))
}

async fn get(app: axum::Router, uri: &str) -> Result<(StatusCode, String)> {
    send(app, Request::get(uri).body(Body::empty())?).await
}

#[rstest]
#[case("/orders/optional?api-version=1.0", r#"Some("1.0") 1.0"#)]
#[case("/orders/optional?api-version=1", r#"Some("1") 1.0"#)]
#[case("/orders/optional?api-version=2.0-beta", r#"Some("2.0-beta") 2.0-beta"#)]
#[case("/orders/optional?api-version=2026-10-01", r#"Some("2026-10-01") 2026-10-01"#)]
#[case(
    "/orders/optional?api-version=2026-10-01-a.very.long.preview.status",
    r#"Some("2026-10-01-a.very.long.preview.status") 2026-10-01-a.very.long.preview.status"#
)]
#[case("/orders/required?api-version=1", r#"Some("1") 1.0"#)]
#[case("/orders/extension?api-version=1", r#"Some("1") 1.0"#)]
#[case("/orders/optional?api-version=1&api-version=1.0", r#"Some("1") 1.0"#)]
#[case("/orders/optional?api-version=1.0&api-version=1", r#"Some("1.0") 1.0"#)]
#[case("/v1/orders", r#"Some("1") 1.0"#)]
#[case("/v1%2E0/orders", r#"Some("1.0") 1.0"#)]
#[case("/health/optional?api-version=1", r#"Some("1") 1.0"#)]
#[case("/health/required?api-version=1", r#"Some("1") 1.0"#)]
#[tokio::test]
async fn handler_should_receive_requested_version(#[case] uri: &str, #[case] expected: &str) -> Result {
    // arrange
    let app = orders(Options::default());

    // act
    let (status, body) = get(app, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[tokio::test]
async fn handler_should_receive_requested_version_from_header() -> Result {
    // arrange
    let options = Options {
        reader: HeaderReader::new("x-api-version").combine(UrlReader::default()).into(),
        ..Default::default()
    };
    let app = orders(options);
    let request = Request::get("/orders/required")
        .header("x-api-version", "1")
        .body(Body::empty())?;

    // act
    let (status, body) = send(app, request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"Some("1") 1.0"#);
    Ok(())
}

#[tokio::test]
async fn handler_should_receive_version_without_raw_value_when_default_is_assumed() -> Result {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: true,
        ..Default::default()
    };
    let app = orders(options);

    // act
    let (status, body) = get(app, "/orders/required").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "None 1.0");
    Ok(())
}

#[tokio::test]
async fn handler_should_not_receive_version_when_neutral_endpoint_is_matched_without_version() -> Result {
    // arrange
    let app = orders(Options::default());

    // act
    let (status, body) = get(app, "/health/optional").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "none");
    Ok(())
}

#[tokio::test]
async fn handler_should_not_assume_version_when_neutral_endpoint_is_matched_without_version() -> Result {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: true,
        ..Default::default()
    };
    let app = orders(options);

    // act
    let (status, body) = get(app, "/health/optional").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "none");
    Ok(())
}

#[tokio::test]
async fn required_extractor_should_be_rejected_when_neutral_endpoint_is_matched_without_version() -> Result {
    // arrange
    let app = orders(Options::default());

    // act
    let (status, body) = get(app, "/health/required").await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        common::detail(&body)?,
        "An API version is required, but was not specified."
    );
    Ok(())
}
