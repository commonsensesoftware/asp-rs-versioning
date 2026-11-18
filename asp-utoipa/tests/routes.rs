use api_versioning::{Options, api_version};
use api_versioning_axum::Router;
use api_versioning_utoipa::{doc, documents, routes};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::Value;
use tower::ServiceExt;
use utoipa::openapi::{InfoBuilder, OpenApiBuilder};

#[utoipa::path(get, path = "", responses((status = 200)))]
async fn get_orders() {}

fn app() -> (axum::Router, Vec<api_versioning_utoipa::ApiDocument>) {
    let (app, apis): (axum::Router, _) = Router::new(Options::default())
        .api("/orders", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| e.get(doc!(get_orders)))
        })
        .split();
    let base = OpenApiBuilder::new()
        .info(InfoBuilder::new().title("Orders").version("0.0.0"))
        .build();
    let documents = documents(&apis, base);

    (app.merge(routes(&documents)), documents)
}

#[tokio::test]
async fn routes_should_serve_each_document_at_its_url() -> anyhow::Result<()> {
    // arrange
    let (app, documents) = app();

    for document in &documents {
        let request = Request::get(document.url()).body(Body::empty())?;

        // act
        let response = app.clone().oneshot(request).await?;

        // assert
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "application/json");

        let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;

        assert_eq!(body, serde_json::to_value(document.openapi())?);
    }

    Ok(())
}

#[tokio::test]
async fn routes_should_not_serve_undefined_document() -> anyhow::Result<()> {
    // arrange
    let (app, _) = app();
    let request = Request::get("/openapi/v3.json").body(Body::empty())?;

    // act
    let response = app.oneshot(request).await?;

    // assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
}
