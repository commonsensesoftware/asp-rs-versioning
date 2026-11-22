<!-- description: Known limitations. -->

# Known Limitations

## URL Path Segment

API versioning does not fundamentally change how routing works. When you elect to support API versioning via a URL path
segment, the API version is part of the path considered in routing. There is currently no built-in method to match a
route where the API version in a URL path segment is unspecified.

The recommended method to enable this scenario is to use _Double Route Registration_ by providing multiple routes for
the corresponding handlers as follows:

```rust
use api_versioning::Options;
use api_versioning_axum::Router;
use axum::extract::Path;
use serde::Deserialize;

// The route parameters shared by both routes. Fields are matched by name, so
// the `api_version` segment captured by `/v{api_version}/...` is ignored, and
// the default route, which has no such segment, also matches.
#[derive(Deserialize)]
struct Params {
    id: u32,
}

// GET api/values/{id}
// GET api/v1/values/{id}
async fn by_id(Path(Params { .. }): Path<Params>) {}

// GET api/v2/values/{id}
async fn by_id_v2(Path(Params { .. }): Path<Params>) {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut options = Options::default();
    let default_version = options.default_version.clone();

    options.assume_default_version_when_unspecified = true;

    let app: axum::Router = Router::new(options)
        .api("/api/values", |api| {
          api.has(default_version)
             .route("/{id}", |e| e.get(by_id))
        })
        .api("/api/v{api_version}/values", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
               .route("/{id}", |e| {
                  e.get(by_id)
                   .map(api_version!(2.0), |h| h.get(by_id_v2))
            })
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```
