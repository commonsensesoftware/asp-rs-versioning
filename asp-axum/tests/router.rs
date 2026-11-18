use api_versioning::{Options, api_version};
use api_versioning_axum::{Router, error::Context};
use axum::{
    body::Body,
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::get,
};
use http::{Method, Request, StatusCode};
use rstest::rstest;
use tower::ServiceExt;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

async fn ok() -> &'static str {
    "ok"
}

async fn send(app: axum::Router, uri: &str) -> Result<(StatusCode, Option<String>, String)> {
    let request = Request::get(uri).body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let layered = response
        .headers()
        .get("x-layer")
        .map(|value| value.to_str().unwrap_or_default().to_owned());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, layered, String::from_utf8(body.to_vec())?))
}

async fn tag(mut response: Response) -> Response {
    response.headers_mut().insert("x-layer", "applied".parse().unwrap());
    response
}

fn reports() -> Router {
    Router::default().api("/reports", |api| {
        api.name("Reports")
            .has(api_version!(2.0))
            .route("/", |e| e.get(ok))
            .route("/{id}", |e| e.get(ok).delete(ok))
    })
}

#[test]
fn split_should_describe_endpoints_with_complete_paths() {
    // arrange
    let router: Router = Router::new(Options::default())
        .api("/orders", |api| {
            api.name("Orders")
                .has(api_version!(1.0))
                .route("/", |e| e.get(ok).post(ok))
        })
        .api("/", |api| api.neutral().route("/health", |e| e.get(ok)))
        .nest("/admin", reports())
        .nest(
            "/tenants/{tenant}",
            Router::default().nest(
                "/files",
                Router::default().api("/", |api| api.route("/{*path}", |e| e.get(ok))),
            ),
        )
        .merge(Router::default().api("/people", |api| api.route("/", |e| e.get(ok))));

    // act
    let (_, apis) = router.split();

    // assert
    let actual: Vec<_> = apis
        .iter()
        .map(|api| (api.method().clone(), api.path(), api.name()))
        .collect();
    assert_eq!(
        actual,
        [
            (Method::GET, "/orders", "Orders"),
            (Method::POST, "/orders", "Orders"),
            (Method::GET, "/health", ""),
            (Method::GET, "/admin/reports", "Reports"),
            (Method::GET, "/admin/reports/{id}", "Reports"),
            (Method::DELETE, "/admin/reports/{id}", "Reports"),
            (Method::GET, "/tenants/{tenant}/files/{*path}", ""),
            (Method::GET, "/people", ""),
        ]
    );
}

#[test]
fn split_should_describe_declared_versions_and_metadata() {
    // arrange
    let router: Router = Router::default()
        .api("/orders", |api| {
            api.deprecates(api_version!(0.9))
                .has(api_version!(1.0))
                .advertises(api_version!(3.0))
                .route("/", |e| e.get(ok))
        })
        .nest("/admin", reports())
        .api("/health", |api| api.neutral().route("/", |e| e.get(ok)));

    // act
    let (_, apis) = router.split();

    // assert
    assert_eq!(
        apis.versions(),
        [api_version!(0.9), api_version!(1.0), api_version!(2.0)]
    );
    assert_eq!(apis.len(), 5);
    assert!(apis.iter().last().unwrap().metadata().is_neutral());
}

#[rstest]
#[case("/orders?api-version=1.0", StatusCode::OK)]
#[case("/admin/reports/42?api-version=2.0", StatusCode::OK)]
#[case("/admin/reports?api-version=1.0", StatusCode::BAD_REQUEST)]
#[case("/tenants/7/files/a/b.txt?api-version=1.0", StatusCode::OK)]
#[tokio::test]
async fn request_should_route_to_nested_api(#[case] uri: &str, #[case] expected: StatusCode) -> Result {
    // arrange
    let app: axum::Router = Router::new(Options::default())
        .api("/orders", |api| api.has(api_version!(1.0)).route("/", |e| e.get(ok)))
        .nest("/admin", reports())
        .nest(
            "/tenants/{tenant}",
            Router::default().api("/files", |api| api.route("/{*path}", |e| e.get(ok))),
        )
        .into();

    // act
    let (status, _, _) = send(app, uri).await?;

    // assert
    assert_eq!(status, expected);
    Ok(())
}

#[tokio::test]
async fn request_should_match_version_neutral_endpoint_with_version_declared_by_nested_api() -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .api("/health", |api| api.neutral().route("/", |e| e.get(ok)))
        .nest("/admin", reports())
        .into();

    // act
    let (status, _, _) = send(app, "/health?api-version=2.0").await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0", Some("applied"))]
#[case("/people?api-version=1.0", None)]
#[tokio::test]
async fn layer_should_only_apply_to_routes_added_before_it(
    #[case] uri: &str,
    #[case] expected: Option<&str>,
) -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .api("/orders", |api| api.route("/", |e| e.get(ok)))
        .layer(map_response(tag))
        .api("/people", |api| api.route("/", |e| e.get(ok)))
        .into();

    // act
    let (_, layered, _) = send(app, uri).await?;

    // assert
    assert_eq!(layered.as_deref(), expected);
    Ok(())
}

#[rstest]
#[case("/orders?api-version=1.0", StatusCode::OK, "ok")]
#[case("/version", StatusCode::OK, "unversioned")]
#[case("/legacy/ping", StatusCode::OK, "pong")]
#[case("/missing", StatusCode::NOT_FOUND, "fallback")]
#[tokio::test]
async fn router_should_pass_through_unversioned_routes(
    #[case] uri: &str,
    #[case] expected_status: StatusCode,
    #[case] expected_body: &str,
) -> Result {
    // arrange
    let legacy = axum::Router::new().route("/ping", get(|| async { "pong" }));
    let app: axum::Router = Router::default()
        .api("/orders", |api| api.route("/", |e| e.get(ok)))
        .route("/version", get(|| async { "unversioned" }))
        .nest("/legacy", legacy)
        .fallback(|| async { (StatusCode::NOT_FOUND, "fallback") })
        .into();

    // act
    let (status, _, body) = send(app, uri).await?;

    // assert
    assert_eq!(status, expected_status);
    assert_eq!(body, expected_body);
    Ok(())
}

#[tokio::test]
async fn router_should_use_configured_error_responder() -> Result {
    // arrange
    let app: axum::Router = Router::default()
        .with_error_responder(|context: &Context<'_>| (context.status(), "custom").into_response())
        .nest("/admin", reports())
        .into();

    // act
    let (status, _, body) = send(app, "/admin/reports?api-version=1.0").await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "custom");
    Ok(())
}

#[test]
#[should_panic(
    expected = "Only the root router can be configured. Use Router::default() to create a router that is nested or merged."
)]
fn nest_should_panic_when_router_is_configured() {
    // arrange
    let nested: Router = Router::new(Options::default());

    // act
    let _ = Router::default().nest("/admin", nested);

    // assert
}

#[test]
#[should_panic(
    expected = "Only the root router can be configured. Use Router::default() to create a router that is nested or merged."
)]
fn merge_should_panic_when_router_is_configured() {
    // arrange
    let merged: Router =
        Router::default().with_error_responder(|context: &Context<'_>| context.status().into_response());

    // act
    let _ = Router::default().merge(merged);

    // assert
}
