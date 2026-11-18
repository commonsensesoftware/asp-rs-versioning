//! Run with `cargo run -p api-versioning-utoipa --example scalar`, then browse to:
//!
//! * Scalar with a document for each API version: <http://localhost:3000/scalar>
//! * The OpenAPI document for API version 1.0: <http://localhost:3000/openapi/v1.json>
//! * The orders in API version 2.0: <http://localhost:3000/orders?api-version=2.0>

mod common;

use api_versioning_utoipa::routes;
use serde_json::{Value, json};
use utoipa_scalar::{Scalar, Servable};

/// The HTML of the Scalar page, which is configured with `$spec`.
///
/// Scalar embeds a single document by default. Each document is served by routes instead, and Scalar is configured
/// with the URL of each document so that the API version can be selected on the page.
const HTML: &str = r#"<!doctype html>
<html>
<head>
    <title>$title</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
</head>
<body>
<div id="app"></div>
<script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
<script>
    Scalar.createApiReference('#app', $spec)
</script>
</body>
</html>"#;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (app, documents) = common::app();

    // the latest API version is selected by default
    let latest = documents.len() - 1;
    let sources: Vec<Value> = documents
        .iter()
        .enumerate()
        .map(|(index, document)| {
            json!({
                "title": document.name(),
                "slug": document.name(),
                "url": document.url(),
                "default": index == latest,
            })
        })
        .collect();
    let scalar = Scalar::with_url("/scalar", json!({ "sources": sources }))
        .custom_html(HTML)
        .title("Orders");
    let app = app.merge(routes(&documents)).merge(scalar);
    let listener = tokio::net::TcpListener::bind(common::ADDRESS).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
