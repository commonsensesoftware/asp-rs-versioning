<!-- description: Make an endpoint accept any API version, or none, for cases such as health checks. -->

# Version-Neutral

All services should be explicitly versioned. In rare cases, however, you may have a service that is _version-neutral_.
A common scenario is a health check service that behaves in the exact same way, regardless of API version. This might
also apply to a legacy service that doesn't support API versioning.

A _version-neutral_ service has the following characteristics:

* Accepts **any** valid API version
* Accepts no API version at all (e.g. unspecified)

This is an important distinction and why the term _version-neutral_ is used. A _version-neutral_ service accepts any and
all versions, including none. This behavior can be used to define a service that accepts all API versions or service
that simply does not care about specific API versions.

It is not possible to have some versions of a service that are API version-neutral and other versions of the same
service require an explicit API version.  If the route of an API version-neutral service matches any other service,
it will result in an ambiguous match (e.g. server error).

## Example

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::UrlReader;
use api_versioning_axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/api/health/ping", |api| api.route("/", |e| e.neutral(|h| h.get(|| async {}))))
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

A version-neutral endpoint using the query string method will not require that a client specify an API version. A
version-neutral endpoint using the URL path method will match any well-formed API version in the URL path segment.

While a version-neutral endpoint doesn't require an API version, it can only match versions that are defined. In an
actual application, there would be at least one defined API version. If all endpoints were version-neutral, then there
is no point to API versioning.

```rust
use api_versioning::{ApiVersion, Options, api_version};
use api_versioning::readers::UrlReader;
use api_versioning_axum::Router;
use axum::extract::Pat;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/api/v{api_version}/health/ping", |api| {
            api.has(api_version!(1.0))
               .route("/", |e| e.neutral(|h| h.get(|Path(_): Path<ApiVersion<'static>>| async {})))
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

```http
GET api/health/ping HTTP/2
```
_Figure 1: a request without a query parameter_

```http
GET api/health/ping?api-version=1.0 HTTP/2
```
_Figure 2: a request with a query parameter_

```http
GET api/v1/health/ping HTTP/2
```
_Figure 3: a request with the version in the URL_

All requests will return the same result.

```http
HTTP/2 204
```
