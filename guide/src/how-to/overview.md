<!-- description: How API versioning disambiguates overlapping routes. -->

# How to Version Your Service

REST services are implemented in [axum] as an endpoint that maps a route to a handler. To version your service, you
simply need to declare your endpoints with the appropriate API version information.

## How It Works

The way that you create and define routes remains unchanged. The key difference is that routes may now overlap. [axum]
assumes a one-to-one mapping between routes and endpoints, therefore, it considers duplicate routes to be ambiguous. The
API versioning `Router` replaces the default implementation and allows endpoints to also be disambiguated by API
version. Although multiple routes may match a request, they are expected to be distinguishable by API version. If the
routes cannot be disambiguated, this is likely a developer mistake and the behavior is the same as the default
implementation.

### Example

The versioned `Router` groups one or more endpoints that are anchored to a top-level path. You may also add a logical
API name to the group. The logical API name is typically useful for OpenAPI documentation.

```rust
use api_versioning::api_version;
use api_versioning_axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/people", |api| {
            api.name("People") // ← provides optional, logical name
                .has(api_version!(1.0))
                .route("/", |e| e.get(|| async { }))
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

## Versioning Methods

Several API versioning methods are supported out-of-the-box:

- [By Query String](version-by-query-string.md) (default)
- [By Media Type](version-by-media-type.md)
- [By Header](version-by-header.md)
- [By URL Segment](version-by-url.md)

Multiple methods of API versioning can be supported simultaneously. Use the `Combine` trait from the crate prelude to
compose two or more [Reader] implementations together. You can also implement your own method of extracting the
requested API version using a custom [Reader].

[axum]: https://crates.io/crates/axum
[Reader]: ../config/reader.md
