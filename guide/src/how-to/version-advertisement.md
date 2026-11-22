<!-- description: Aggregate and advertise API versions that are split across deployments behind a gateway. -->

# Version Advertisement

Splitting implemented service API versions across hosted applications or endpoints is a fairly common scenario. There
are several reasons why you might choose to split hosted endpoints, such as different run-time versions or traffic load
balancing.

When service API versions are split across deployments, two issues arise:

1. The correct service API version cannot be selected across deployments.
2. The set of implemented service API versions cannot be aggregated across deployments.

## Service Gateway

The first issue can be remedied by a using a service gateway.  The gateway becomes responsible for obfuscating which
endpoints host which API versions. The exact method in which gateways implement this functionality is at the discretion
of service authors.

## Service API Version Advertisement

Since there is no direct way to know or interrogate the available API version information at runtime in a performant
manner when services are deployed separately, an alternate approach is required. This concept is referred to as
*service API version advertisement*. Each service will advertise the supported and deprecated API versions it knows
about.

A service can advertise its supported and deprecated API versions, except they are never considered for routing. The
advertised and implemented API versions are always aggregated together.

The following is an example of a service with API version `2.0` hosted at another endpoint that knows that API version
`1.0` is a supported version somewhere else:

```rust
use api_versioning::api_version;
use api_versioning_axum::{RequestedApiVersion, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/helloworld", |api| {
            api.has(api_version!(2.0))
               .advertises(api_version!(1.0))
               .route("/", |e| e.get(|| async { "Hello world v2.0!" }))
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

This service implementation will now advertise that API version `1.0` and `2.0` are supported through the
`api-supported-versions` HTTP header even though it has no knowledge about where API version `1.0` is. In a similar
fashion, a service can also advertise deprecated API versions using `advertises_deprecated`. Note that the
[Options::report_versions] must be enabled for the HTTP headers to be returned in responses.

The only drawback to this approach is that each implementation needs to be updated with the supported and deprecated API
versions when new API versions are released. One possible solution to this limitation is to populate the advertised API
versions from a configuration source such as a file or database. If this is still undesirable, then there is still the
option of using HTTP header injection by the host server or another mechanism to send the supported and deprecated API
version information.

[Options::report_versions]: ../config/options.md#report-api-versions
