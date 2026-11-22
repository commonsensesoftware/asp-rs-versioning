<!-- description: Set up API versioning in a new service. -->

# New Services

When a service author creates new services that consider API versioning upfront, then the configuration and setup is
very straightforward. The following examples provide a quick start setup with default configurations.

```rust
use api_versioning::api_version;
use api_versioning_axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::default();
    let app: axum::Router = Router::default()
        .api("/people", |api| {
            api.has([api_version!(1.0)])
               .route("/", |e| e.get(|| async {}))
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```
