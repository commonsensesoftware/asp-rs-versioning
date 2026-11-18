use api_versioning::{
    Options, ReportOptions, api_version,
    problem::{ProblemDetails, ProblemType},
};
use api_versioning_axum::{
    ApiVersionError, RequestedApiVersion, Router,
    error::{Context, ProblemDetailsResponder, Responder},
};
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use http::{HeaderMap, Request, StatusCode, header::ACCEPT, header::CONTENT_TYPE};
use rstest::rstest;
use tower::ServiceExt;

mod common;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

async fn get_orders() -> &'static str {
    "orders"
}

async fn requested(_: RequestedApiVersion) -> &'static str {
    "requested"
}

fn orders(router: Router) -> axum::Router {
    router
        .api("/orders", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| e.get(get_orders))
        })
        .api("/health", |api| api.neutral().route("/", |e| e.get(requested)))
        .into()
}

async fn send(app: axum::Router, uri: &str, accept: Option<&str>) -> Result<(StatusCode, HeaderMap, String)> {
    let mut request = Request::get(uri);

    if let Some(accept) = accept {
        request = request.header(ACCEPT, accept);
    }

    let response = app.oneshot(request.body(Body::empty())?).await?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, headers, String::from_utf8(body.to_vec())?))
}

#[rstest]
#[case(
    "/orders",
    r#"{"type":"https://docs.api-versioning.org/problems#unspecified","title":"Unspecified API version","status":400,"detail":"An API version is required, but was not specified."}"#
)]
#[case(
    "/orders?api-version=3.0",
    r#"{"type":"https://docs.api-versioning.org/problems#unsupported","title":"Unsupported API version","status":400,"detail":"The HTTP resource that matches the request URI '/orders' does not support the API version '3.0'."}"#
)]
#[case(
    "/orders?api-version=abc",
    r#"{"type":"https://docs.api-versioning.org/problems#invalid","title":"Invalid API version","status":400,"detail":"The HTTP resource that matches the request URI '/orders' does not support the API version 'abc'."}"#
)]
#[case(
    "/orders?api-version=2.0&api-version=1.0",
    r#"{"type":"https://docs.api-versioning.org/problems#ambiguous","title":"Ambiguous API version","status":400,"detail":"The following API versions were requested: 1.0, 2.0. At most, only a single API version may be specified. Please update the intended API version and retry the request."}"#
)]
#[tokio::test]
async fn request_should_respond_with_problem_details(#[case] uri: &str, #[case] expected: &str) -> Result {
    // arrange
    let app = orders(Router::default());

    // act
    let (status, headers, body) = send(app, uri, None).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(headers[CONTENT_TYPE], "application/problem+json");
    assert_eq!(body, expected);
    Ok(())
}

#[tokio::test]
async fn request_should_respond_with_configured_status_for_unsupported_version() -> Result {
    // arrange
    let router = Router::new(Options {
        unsupported_status_code: 501,
        ..Default::default()
    });
    let app = orders(router);

    // act
    let (status, _, body) = send(app, "/orders?api-version=3.0", None).await?;

    // assert
    let problem: ProblemDetails = serde_json::from_str(&body)?;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(
        problem,
        ProblemDetails::new(ProblemType::UNSUPPORTED, 501, common::unsupported("/orders", "3.0"))
    );
    Ok(())
}

#[rstest]
#[case("application/problem+json")]
#[case("application/json")]
#[case("*/*")]
#[case("text/html, application/*;q=0.5")]
#[tokio::test]
async fn request_should_respond_with_problem_details_when_accepted(#[case] accept: &str) -> Result {
    // arrange
    let app = orders(Router::default());

    // act
    let (status, headers, body) = send(app, "/orders", Some(accept)).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(headers[CONTENT_TYPE], "application/problem+json");
    assert_eq!(
        common::detail(&body)?,
        "An API version is required, but was not specified."
    );
    Ok(())
}

#[rstest]
#[case("text/html")]
#[case("application/xml")]
#[case("application/json;q=0, application/problem+json;q=0, */*")]
#[tokio::test]
async fn request_should_respond_without_body_when_problem_details_are_not_accepted(#[case] accept: &str) -> Result {
    // arrange
    let app = orders(Router::default());

    // act
    let (status, headers, body) = send(app, "/orders", Some(accept)).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(!headers.contains_key(CONTENT_TYPE));
    assert_eq!(body, "");
    Ok(())
}

#[tokio::test]
async fn request_should_respond_with_customized_problem_details() -> Result {
    // arrange
    let router = Router::default().with_error_responder(|context: &Context<'_>| {
        let mut problem = context.problem();
        let id = context
            .request()
            .and_then(|request| request.headers.get("x-request-id"))
            .and_then(|id| id.to_str().ok())
            .unwrap_or_default();

        problem.extensions.insert("requestId".into(), id.into());
        context.respond_with(&problem)
    });
    let app = orders(router);
    let request = Request::get("/orders")
        .header("x-request-id", "42")
        .body(Body::empty())?;

    // act
    let response = app.oneshot(request).await?;

    // assert
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let problem: ProblemDetails = serde_json::from_slice(&body)?;
    assert_eq!(problem.r#type.as_deref(), Some(ProblemType::UNSPECIFIED.uri));
    assert_eq!(problem.extensions["requestId"], "42");
    Ok(())
}

/// Represents a responder that responds with plain text.
struct PlainText;

impl Responder for PlainText {
    fn respond(&self, context: &Context<'_>) -> Response {
        (context.status(), context.error().to_string()).into_response()
    }
}

#[tokio::test]
async fn request_should_respond_with_custom_responder() -> Result {
    // arrange
    let router = Router::new(Options {
        report_versions: ReportOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    })
    .with_error_responder(PlainText);
    let app = orders(router);

    // act
    let (status, headers, body) = send(app, "/orders?api-version=3.0", None).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(headers[CONTENT_TYPE], "text/plain; charset=utf-8");
    assert_eq!(headers["api-supported-versions"], "1.0, 2.0");
    assert_eq!(body, common::unsupported("/orders", "3.0"));
    Ok(())
}

#[tokio::test]
async fn extractor_rejection_should_respond_with_problem_details() -> Result {
    // arrange
    let app = orders(Router::default());

    // act
    let (status, headers, body) = send(app, "/health", Some("text/html")).await?;

    // assert
    let problem: ProblemDetails = serde_json::from_str(&body)?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(headers[CONTENT_TYPE], "application/problem+json");
    assert_eq!(
        problem,
        ProblemDetails::new(
            ProblemType::UNSPECIFIED,
            400,
            "An API version is required, but was not specified."
        )
    );
    Ok(())
}

#[test]
fn problem_details_responder_should_respond_without_request() {
    // arrange
    let error = ApiVersionError::Ambiguous(vec!["1.0".into(), "2.0".into()]);
    let context = Context::new(&error, StatusCode::BAD_REQUEST, None);

    // act
    let response = ProblemDetailsResponder.respond(&context);

    // assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[CONTENT_TYPE], "application/problem+json");
}
