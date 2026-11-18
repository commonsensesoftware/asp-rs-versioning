use api_versioning::{
    ApiVersion, Model, Options, RequestInfo, api_version,
    selectors::{ConstantSelector, CurrentVersionSelector, LowestVersionSelector, SelectedVersion, Selector},
};
use api_versioning_axum::{RequestedApiVersion, Router};
use axum::body::Body;
use http::{Request, StatusCode};
use rstest::rstest;
use tower::ServiceExt;

mod common;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

async fn requested(requested: RequestedApiVersion) -> String {
    format!("{:?} {}", requested.raw(), requested.version())
}

/// Represents a selector that selects a version from a request header after waiting asynchronously.
#[derive(Clone, Debug)]
struct HeaderSelector;

impl Selector for HeaderSelector {
    fn select<'a>(&'a self, request: &'a (dyn RequestInfo + Sync), _model: &'a Model) -> SelectedVersion<'a> {
        Box::pin(async move {
            tokio::task::yield_now().await;

            let value = request.headers().get("x-default-version")?.to_str().ok()?;

            ApiVersion::parse(value).ok()
        })
    }

    fn clone(&self) -> Box<dyn Selector> {
        Box::new(Clone::clone(self))
    }
}

fn assume(selector: impl Selector + 'static) -> Options {
    Options {
        assume_default_version_when_unspecified: true,
        default_version: api_version!(42.0),
        selector: Some(Box::new(selector)),
        ..Default::default()
    }
}

fn orders(options: Options) -> axum::Router {
    let router = Router::new(options);

    router
        .api("/orders", |api| {
            api.has([api_version!(1.0), api_version!(2.0), api_version!(3.0-beta)])
                .advertises(api_version!(4.0))
                .route("/", |e| e.get(requested))
                .route("/{id}", |e| {
                    e.map(api_version!(1.0), |e| e.delete(requested))
                        .map([api_version!(1.0), api_version!(2.0)], |e| e.patch(requested))
                })
        })
        .into()
}

async fn send(app: axum::Router, request: Request<Body>) -> Result<(StatusCode, String)> {
    let response = app.oneshot(request).await?;
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, String::from_utf8(body.to_vec())?))
}

#[tokio::test]
async fn request_should_assume_default_version_with_default_selector() -> Result {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: true,
        default_version: api_version!(2.0),
        ..Default::default()
    };
    let request = Request::get("/orders").body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "None 2.0");
    Ok(())
}

#[tokio::test]
async fn request_should_prefer_selector_over_default_version() -> Result {
    // arrange
    let options = assume(ConstantSelector::new(api_version!(1.0)));
    let request = Request::get("/orders").body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "None 1.0");
    Ok(())
}

#[tokio::test]
async fn request_should_not_use_selector_when_default_version_is_not_assumed() -> Result {
    // arrange
    let options = Options {
        assume_default_version_when_unspecified: false,
        ..assume(ConstantSelector::new(api_version!(1.0)))
    };
    let request = Request::get("/orders").body(Body::empty())?;

    // act
    let (status, _) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn request_should_not_use_selector_when_version_is_specified() -> Result {
    // arrange
    let options = assume(ConstantSelector::new(api_version!(1.0)));
    let request = Request::get("/orders?api-version=2.0").body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"Some("2.0") 2.0"#);
    Ok(())
}

#[rstest]
#[case(Request::get("/orders"), "None 2.0")]
#[case(Request::patch("/orders/1"), "None 2.0")]
#[case(Request::delete("/orders/1"), "None 1.0")]
#[tokio::test]
async fn current_version_selector_should_select_from_versions_of_route_and_method(
    #[case] request: http::request::Builder,
    #[case] expected: &str,
) -> Result {
    // arrange
    let options = assume(CurrentVersionSelector::new(api_version!(42.0)));
    let request = request.body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[tokio::test]
async fn lowest_version_selector_should_select_lowest_version() -> Result {
    // arrange
    let options = assume(LowestVersionSelector::new(api_version!(42.0)));
    let request = Request::get("/orders").body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "None 1.0");
    Ok(())
}

#[tokio::test]
async fn selector_should_read_request_asynchronously() -> Result {
    // arrange
    let options = assume(HeaderSelector);
    let request = Request::get("/orders")
        .header("x-default-version", "2.0")
        .body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "None 2.0");
    Ok(())
}

#[tokio::test]
async fn request_should_be_unspecified_when_selector_does_not_select_version() -> Result {
    // arrange
    let options = assume(HeaderSelector);
    let request = Request::get("/orders").body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        common::detail(&body)?,
        "An API version is required, but was not specified."
    );
    Ok(())
}

#[tokio::test]
async fn request_should_be_unsupported_when_selector_selects_unimplemented_version() -> Result {
    // arrange
    let options = assume(ConstantSelector::new(api_version!(4.0)));
    let request = Request::get("/orders").body(Body::empty())?;

    // act
    let (status, body) = send(orders(options), request).await?;

    // assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(common::detail(&body)?, common::unsupported("/orders", "4.0"));
    Ok(())
}
