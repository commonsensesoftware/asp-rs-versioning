<!-- description: Version an API with an arbitrary HTTP request header. -->

# Header Versioning

While media type negotiation is the defined method in REST for reasoning about the content expectations between a
client and server, any arbitrary HTTP header can also be used to drive API versioning.

## Example

The following example updates the configuration to read API versions from a custom header.

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::HeaderReader;
use api_versioning_axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options {
        reader: Box::new(HeaderReader::new("x-api-version")),
        ..Default::default()
    };
    let app: axum::Router = Router::new(options)
        .api("/helloworld", |api| {
            api.has(api_version!(1.0))
               .route("/", |e| e.get(|| async { "Hello world!" }))
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

This will allow clients to request a specific API version by the custom HTTP header `x-api-version`. For example:

```http
GET api/helloworld HTTP/2
x-api-version: 1.0
```

```http
HTTP/2 200
content-type: text/plain
content-length: 12

Hello world!
```