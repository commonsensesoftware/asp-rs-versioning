<!-- description: Read the API version a client requested in a handler. -->

# Requested API Version

All of the service API version information is accessible via extensions. A handler can receive the entire endpoint
`Metadata` or the `RequestedApiVersion`, which will contain the raw, unparsed text and the parsed, valid `ApiVersion`.
These features allow you to determine which API version was requested by a client as well as determine which versions
are supported and deprecated. The API versions provided are automatically aggregated across all service implementations.

The most common usage is the current, client requested API version:

```rust
use api_versioning::api_version;
use api_versioning_axum::{RequestedApiVersion, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/example", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
               .route("/", |e| {
                    e.get(|requested: RequestedApiVersion| async {
                        requested.raw().unwrap_or_default().into()
                    })
                })
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```
