<!-- description: Mark an API version deprecated to advertise that it will become unsupported. -->

# Deprecating Versions

When a service supports multiple API versions, some versions will eventually be deprecated over time. To advertise that
one or more API versions have been deprecated, simply declare the deprecated API versions with `deprecates`. A
deprecated API version does not mean the API version is not supported. A deprecated API version means that the version
will become unsupported after six months or more.

The following examples illustrate how to specify deprecated API versions depending on which service API versioning
approach you selected.

This example demonstrates API versioning using all non-URL segment methods.

```rust
use api_versioning::api_version;
use api_versioning_axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/api/helloworld", |api| {
            api.deprecates(api_version!(1.0))
               .has(api_version!(2.0))
               .route("/", |e| {
                    e.get(|| async { "Hello world!" })
                     .map(api_version!(2.0), |h| h.get(|| async { "Hello world v2.0!" }))
                })
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

This example demonstrates API versioning using the URL segment method.

```rust
use api_versioning::api_version;
use api_versioning_axum::Router;
use axum::extract::Path;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/api/v{api_version}/helloworld", |api| {
            api.deprecates(api_version!(1.0))
               .has(api_version!(2.0))
               .route("/", |e| {
                    e.get(|Path(_): Path<ApiVersion<'static>>| async { "Hello world!" })
                     .map(api_version!(2.0), |h| {
                        h.get(|Path(_): Path<ApiVersion<'static>>| async { "Hello world v2.0!" })
                     })
                })
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

## Removing a Service

To permanently sunset a service, simply remove that route or API version from your implementation. The route will
no longer be matched. When one or more specific API versions cannot be matched, clients will receive HTTP status code
`400` (Bad Request). If no candidate routes match at all, clients will receive HTTP status code `404` (Not Found).
