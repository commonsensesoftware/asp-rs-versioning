<!-- description: Retrofit API versioning onto a service already in production without breaking clients. -->

# Existing Services

While it's great to plan for an API versioning story for your services upfront, it's all too common to need API
versioning after your services are in production. The API Versioning libraries provide features to help you retrofit
existing services and integrate formal API versioning without breaking your existing clients.

Unless a service is API version-neutral, existing services have some logical, yet undefined, API version that is not
formally declared by the service or known to a client. In order to prevent existing clients from breaking, they must be
able to make requests to the original URL without specifying any API version information.

When API versioning is applied, all of the existing services now have an explicit API version on the service side. The
initial, default API version is `1.0`, but that can be configured to be a different API version.

```rust
use api_versioning::api_version;
use api_versioning_axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut options = Options::default();

    // allow a client to call you without specifying an api version
    // since we haven't configured it otherwise, the assumed api version will be 1.0
    options.assume_default_version_when_unspecified = true;

    let app: axum::Router = Router::new(options)
        .api("/people", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
               .route("/", |e| e.get(|| async { "v1" }).map(api_version!(2.0), |h| h.get(|| async { "v2" })))
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```
