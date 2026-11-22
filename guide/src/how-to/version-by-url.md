<!-- description: Version an API by URL path segment. -->

# URL Path Versioning

An alternate, but common, method of API versioning is to use a URL path segment. This approach does not allow implicitly
matching the initial, default API version of a service; therefore, all API versions must be explicitly declared. In
addition, the API version value specified for the URL segment must still conform to the [version format]. The `v` prefix
is **not** part of the API version, but may be included in route templates if you so desire.

> [!IMPORTANT]
> It is not possible to have a default API version for a URL path segment. This means that setting
> `Options::assumed_default_version_when_unspecified` is unlikely to have any affect when you use this method of
> versioning. For more information and possible solutions to address this scenario, refer to the [known limitations].

[version format]: ../version-format.md
[known limitations]: ../limitations.md#url-path-segment

## Example

The following example updates the configuration to read API versions from the URL path.

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::UrlReader;
use api_versioning_axum::Router;

async fn hello() -> &'static str {
    "Hello world!"
}

async fn hello_v2() -> &'static str {
    "Hello world! (v2)"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // note: this is equivalent to Options::default()
    let options = Options {
        reader: Box::new(UrlReader::new("api_version")),
        ..Default::default()
    };
    let app: axum::Router = Router::new(options)
        .api("/v{api_version}/helloworld", |api| {
            api.has(api_version!(1.0))
               .route("/", |e| e.get(hello).map(api_version!(2.0), |h| h.get(hello_v2)))
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

A request to version 1.0 will route to the `hello` handler.

```http
GET v1/helloworld HTTP/2
```

```http
HTTP/2 200
content-type: text/plain
content-length: 12

Hello world!
```

A request to version 2.0 will route to the `hello_v2` handler.

```http
GET v2/helloworld HTTP/2
```

```http
HTTP/2 200
content-type: text/plain
content-length: 17

Hello world! (v2)
```
