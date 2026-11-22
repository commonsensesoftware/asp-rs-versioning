<!-- description: Point the Swagger UI OpenAPI user interface at the documents generated for each API version. -->

# Swagger UI Integration

The [Swagger UI] is a battle-tested OpenAPI user interface and easily integrates with API Versioning. The only thing you
need to do is tell the Swagger UI about the documents your application will generate.

To explore your API with the [Swagger UI], use [utoipa-swagger-ui].

```rust
use api_versioning_utoipa::{ApiDocument, routes};
use utoipa_swagger_ui::{Config, SwaggerUi, Url};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (app, documents) = crate::app();

    // Swagger UI requires the name and URL of each document to live for the rest of the program
    let documents: &'static [ApiDocument] = documents.leak();
    let latest = documents.len() - 1;
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
```

A complete example can be found in [swagger_ui.rs]. You can run the example by cloning the repository or starting a
Codespace and running:

```bash
cargo run --package api-versioning-utoipa --example swagger_ui
```

then navigate to `http://localhost:3000/swagger`.

[Swagger UI]: https://swagger.io/
[utoipa-swagger-ui]: https://crates.io/crates/utoipa-swagger-ui
[swagger_ui.rs]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-utoipa/examples/swagger_ui.rs
