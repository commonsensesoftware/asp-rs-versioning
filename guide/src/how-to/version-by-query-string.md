<!-- description: Version an API with a query string parameter. -->

# Query String Versioning

Versioning by a query string parameter is the most convenient and pragmatic method without violating any REST
constraints. It is, therefore, the recommended and default versioning method. The default configuration defines the
**api-version** query string parameter.

## Example

The following example updates the configuration to read API versions from a query string parameter.

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::QueryStringReader;
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
        reader: Box::new(QueryStringReader::new("api-version")),
        ..Default::default()
    };
    let app: axum::Router = Router::new(options)
        .api("/helloworld", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
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
GET helloworld?api-version=1.0 HTTP/2
```

```http
HTTP/2 200
content-type: text/plain
content-length: 12

Hello world!
```

A request to version 2.0 will route to the `hello_v2` handler.

```http
GET helloworld?api-version=2.0 HTTP/2
```

```http
HTTP/2 200
content-type: text/plain
content-length: 17

Hello world! (v2)
```