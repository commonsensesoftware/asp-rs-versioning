<!-- description: Install API Versioning and stand up a versioned Hello World endpoint in a few lines of code. -->

# Getting Started

The simplest way to get started is to install the crate using the default features with `axum`.

```bash
cargo add api-versioning-axum
```

## Example

The following example sets up the ubiquitous "Hello World" service with two versions of the same endpoint. The `version`
parameter resolves to the requested API version and echoes it back to illustrate different endpoints were reached.

```rust
use api_versioning::{Options, api_version};
use api_versioning_axum::{RequestedApiVersion, Router};
use tokio::net::TcpListener;

// note: specifying RequestedApiVersion is not required, but it provides
// a way to determine which API version was routed to the handler

// GET /helloworld?api-version=1.0
async fn hello(requested: RequestedApiVersion) -> String {
    format!("Hello World! (v{})", requested.version())
}

// GET /helloworld?api-version=2.0
async fn hello_v2(requested: RequestedApiVersion) -> String {
    format!("Hello World! (v{})", requested.version())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/helloworld", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
               .route("/", |e| e.get(hello).map(api_version!(2.0), |h| h.get(hello_v2)))
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

To run the example, use:

```bash
cargo run
```

and then navigate to the endpoint or use:

```bash
curl https://localhost:3000/helloworld?api-version=1.0
```

```bash
curl https://localhost:3000/helloworld?api-version=2.0
```

The repository [examples] demonstrate additional features and capabilities.

[example]: examples.md