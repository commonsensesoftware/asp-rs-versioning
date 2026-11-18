//! Run with `cargo run -p api-versioning-utoipa --example swagger_ui`, then browse to:
//!
//! * Swagger UI with a document for each API version: <http://localhost:3000/swagger-ui>
//! * The OpenAPI document for API version 1.0: <http://localhost:3000/openapi/v1.json>
//! * The orders in API version 2.0: <http://localhost:3000/orders?api-version=2.0>

mod common;

use api_versioning_utoipa::{ApiDocument, routes};
use utoipa_swagger_ui::{Config, SwaggerUi, Url};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (app, documents) = common::app();

    // Swagger UI requires the name and URL of each document to live for the rest of the program
    let documents: &'static [ApiDocument] = documents.leak();
    let latest = documents.len() - 1;

    // the documents are served by routes, so Swagger UI is only configured with their URLs. the latest API version is
    // selected by default.
    let urls = documents
        .iter()
        .enumerate()
        .map(|(index, document)| Url::with_primary(document.name(), document.url(), index == latest));
    let app = app
        .merge(routes(documents))
        .merge(SwaggerUi::new("/swagger-ui").config(Config::new(urls)));
    let listener = tokio::net::TcpListener::bind(common::ADDRESS).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
