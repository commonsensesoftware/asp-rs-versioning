<!-- description: Advertise supported and deprecated API versions from service via response headers. -->

# Version Discovery

Requiring an explicit service version helps ensure existing clients don’t break, but we also need a way to advertise
which service versions are currently supported and which versions are deprecated.

To facilitate this need, services should respond with the `api-supported-versions` and `api-deprecated-versions`, which
are multi-value HTTP headers that indicate the supported and deprecated API versions, respectively. A deprecated version
is still implemented, but is expected to be permanently removed in six months or more.  When a version is no longer
supported, it should stop being advertised. Additional information can be provided via [versioning policies].

Reporting API versions is disabled by default. Service authors can enable this behavior for all services by setting the
[Options::report_versions::enabled] to `true` or scoped to individual services by applying the
`VersionedApi::report_api_versions` builder function.

Service authors might also choose to implement the `OPTIONS` method so that clients and tooling can interrogate which
API versions their service supports.

[versioning policies]: version-policies.md
[Options::report_versions::enabled]: config/options.md#report-api-versions

## Example

```rust
use api_versioning::api_version;
use api_versioning_axum::Router;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;

// OPTIONS /api/myservice?api-version=[1.0|2.0|3.0]
async fn report() ->impl IntoResponse {
    (
        StatusCode::NO_CONTENT,
        [(header::ALLOW, "GET, HEAD, POST, OPTIONS")],
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/api/myservice", |api| {
            api.has([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
                .route("/", |e| e.options(report))
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

```http
HTTP/2 204
allow: GET, POST, OPTIONS
api-supported-versions: 1.0, 2.0, 3.0
```
