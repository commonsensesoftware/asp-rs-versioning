use api_versioning::{
    ApiVersion, Options, ReportOptions, api_version,
    problem::{ProblemDetails, ProblemType},
    readers::{QueryStringReader, UrlReader, combine},
};
use api_versioning_axum::{RequestedApiVersion, Router};
use axum::{body::Body, extract::Path};
use http::{HeaderMap, Request, StatusCode};
use rstest::rstest;
use serde::Deserialize;
use tower::ServiceExt;

mod common;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

async fn get_orders() -> &'static str {
    "get_orders"
}

async fn get_orders_v2() -> &'static str {
    "get_orders_v2"
}

async fn health() -> &'static str {
    "health"
}

async fn requested(requested: Option<RequestedApiVersion>) -> String {
    requested.map_or_else(|| "none".into(), |requested| requested.version().to_string())
}

async fn version_only(Path(version): Path<ApiVersion<'static>>) -> String {
    version.to_string()
}

async fn version_and_id(Path((version, id)): Path<(ApiVersion<'static>, u32)>) -> String {
    format!("{version} {id}")
}

#[derive(Deserialize)]
struct OrderPath {
    api_version: ApiVersion<'static>,
    id: u32,
}

async fn version_in_struct(Path(path): Path<OrderPath>) -> String {
    format!("{} {}", path.api_version, path.id)
}

fn url_only() -> Options {
    Options {
        reader: Box::new(UrlReader::default()),
        ..Default::default()
    }
}

fn orders(options: Options) -> axum::Router {
    let router = Router::new(options);

    router
        .api("/api/v{api_version}/orders", |api| {
            api.deprecates(api_version!(0.9))
                .has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| {
                    e.get(get_orders).map(api_version!(2.0), |e| e.get(get_orders_v2))
                })
        })
        .api("/orders", |api| {
            api.has(api_version!(1.0)).route("/", |e| e.get(get_orders))
        })
        .api("/health", |api| api.neutral().route("/", |e| e.get(health)))
        .into()
}

async fn send(app: axum::Router, uri: &str) -> Result<(StatusCode, HeaderMap, String)> {
    let request = Request::get(uri).body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, headers, String::from_utf8(body.to_vec())?))
}

#[rstest]
#[case("/api/v0.9/orders", "get_orders")]
#[case("/api/v1.0/orders", "get_orders")]
#[case("/api/v1/orders", "get_orders")]
#[case("/api/v1%2E0/orders", "get_orders")]
#[case("/api/v2.0/orders", "get_orders_v2")]
#[case("/health", "health")]
#[tokio::test]
async fn request_should_dispatch_to_handler_for_version_in_url(#[case] uri: &str, #[case] expected: &str) -> Result {
    // arrange
    let app = orders(url_only());

    // act
    let (status, _, body) = send(app, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[rstest]
#[case("/api/v3.0/orders", ProblemType::UNSUPPORTED, &common::unsupported("/api/v3.0/orders", "3.0"))]
#[case("/api/vendors/orders", ProblemType::INVALID, &common::unsupported("/api/vendors/orders", "endors"))]
#[case(
    "/orders",
    ProblemType::UNSPECIFIED,
    "An API version is required, but was not specified."
)]
#[tokio::test]
async fn request_should_return_404_when_url_is_only_version_source(
    #[case] uri: &str,
    #[case] kind: ProblemType,
    #[case] detail: &str,
) -> Result {
    // arrange
    let app = orders(url_only());

    // act
    let (status, _, body) = send(app, uri).await?;

    // assert
    let problem: ProblemDetails = serde_json::from_str(&body)?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(problem, ProblemDetails::new(kind, 404, detail));
    Ok(())
}

#[tokio::test]
async fn request_should_ignore_unsupported_status_code_when_url_is_only_version_source() -> Result {
    // arrange
    let options = Options {
        unsupported_status_code: 422,
        ..url_only()
    };
    let app = orders(options);

    // act
    let (status, _, _) = send(app, "/api/v3.0/orders").await?;

    // assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn request_should_report_api_versions_when_url_version_is_unsupported() -> Result {
    // arrange
    let options = Options {
        report_versions: ReportOptions {
            enabled: true,
            ..Default::default()
        },
        ..url_only()
    };
    let app = orders(options);

    // act
    let (status, headers, _) = send(app, "/api/v3.0/orders").await?;

    // assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(headers["api-supported-versions"], "1.0, 2.0");
    assert_eq!(headers["api-deprecated-versions"], "0.9");
    Ok(())
}

#[tokio::test]
async fn request_should_assume_default_version_when_url_does_not_contain_version() -> Result {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: true,
        ..url_only()
    };
    let app = orders(options);

    // act
    let (status, _, body) = send(app, "/orders").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_orders");
    Ok(())
}

#[rstest]
#[case("/api/v3.0/orders", &common::unsupported("/api/v3.0/orders", "3.0"))]
#[case("/api/vendors/orders", &common::unsupported("/api/vendors/orders", "endors"))]
#[case("/orders", "An API version is required, but was not specified.")]
#[case("/api/v1.0/orders?api-version=2.0", &common::ambiguous("1.0, 2.0"))]
#[tokio::test]
async fn request_should_return_400_when_url_is_not_only_version_source(
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let app = orders(Options::default());

    // act
    let (status, _, body) = send(app, uri).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(common::detail(&body)?, expected);
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0", "get_orders")]
#[case("/api/v2.0/orders", "get_orders_v2")]
#[case("/api/v1/orders?api-version=1.0", "get_orders")]
#[tokio::test]
async fn request_should_read_version_from_any_location_with_default_options(
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let app = orders(Options::default());

    // act
    let (status, _, body) = send(app, uri).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[tokio::test]
async fn request_should_not_read_other_route_parameters_as_version() -> Result {
    // arrange
    let router = Router::default();
    let app: axum::Router = router
        .api("/packages/{name}/{version}", |api| {
            api.has(api_version!(1.0)).route("/", |e| e.get(get_orders))
        })
        .into();

    // act
    let (status, _, body) = send(app, "/packages/orders/not-a-version?api-version=1.0").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_orders");
    Ok(())
}

#[tokio::test]
async fn request_should_read_version_from_custom_route_parameter() -> Result {
    // arrange
    let router = Router::new(Options {
        reader: combine(UrlReader::new("ver")).with(QueryStringReader::default()).into(),
        ..Default::default()
    });
    let app: axum::Router = router
        .api("/{ver}/orders", |api| {
            api.has(api_version!(2026-10-01)).route("/", |e| e.get(requested))
        })
        .into();

    // act
    let (status, _, body) = send(app, "/2026-10-01/orders").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "2026-10-01");
    Ok(())
}

#[tokio::test]
async fn path_should_extract_version() -> Result {
    // arrange
    let router = Router::new(url_only());
    let app: axum::Router = router
        .api("/v{api_version}/orders", |api| {
            api.has(api_version!(2.1-preview.1)).route("/", |e| e.get(version_only))
        })
        .into();

    // act
    let (status, _, body) = send(app, "/v2.1-preview.1/orders").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "2.1-preview.1");
    Ok(())
}

#[tokio::test]
async fn path_should_extract_version_in_tuple() -> Result {
    // arrange
    let router = Router::new(url_only());
    let app: axum::Router = router
        .api("/v{api_version}/orders", |api| {
            api.has(api_version!(1.0)).route("/{id}", |e| e.get(version_and_id))
        })
        .into();

    // act
    let (status, _, body) = send(app, "/v1/orders/42").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "1.0 42");
    Ok(())
}

#[tokio::test]
async fn path_should_extract_version_in_struct() -> Result {
    // arrange
    let router = Router::new(url_only());
    let app: axum::Router = router
        .api("/v{api_version}/orders", |api| {
            api.has(api_version!(1.0)).route("/{id}", |e| e.get(version_in_struct))
        })
        .into();

    // act
    let (status, _, body) = send(app, "/v1.0/orders/42").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "1.0 42");
    Ok(())
}
