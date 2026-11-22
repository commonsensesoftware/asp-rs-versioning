<!-- description: Point the Scalar OpenAPI user interface at the documents generated for each API version. -->

# Scalar Integration

[Scalar] has quickly become one of the more common, modern OpenAPI user interfaces and it easily integrates with API
Versioning. The only thing you need to do is tell Scalar about the documents your application will generate.

To explore your API with [Scalar], use [utoipa-scalar].

```rust
use api_versioning_utoipa::routes;
use serde_json::{Value, json};
use utoipa_scalar::{Scalar, Servable};

// The default template of utoipa-scalar always embeds `$spec` as a document, so a configuration such as `sources`
// requires a custom template. See <https://github.com/juhaku/utoipa/issues/1005>.
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
    let (app, documents) = crate::app();
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
        .title("API Reference");
    let app = app.merge(routes(&documents)).merge(scalar);
    let listener = tokio::net::TcpListener::bind(common::ADDRESS).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

A complete example can be found in [scalar.rs]. You can run the example by cloning the repository or starting a
Codespace and running:

```bash
cargo run --package api-versioning-utoipa --example scalar
```

then navigate to `http://localhost:3000/scalar`.

[Scalar]: https://scalar.com/
[utoipa-scalar]: https://crates.io/crates/utoipa-scalar
[scalar.rs]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-utoipa/examples/scalar.rs
