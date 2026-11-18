//! Run with `cargo run -p api-versioning-axum --example hello`, then try:
//!
//! * `curl -i "http://localhost:3000/hello?api-version=1.0"`
//! * `curl -i "http://localhost:3000/hello?api-version=2.0"`
//! * `curl -i "http://localhost:3000/hello?api-version=3.0"`
//! * `curl -i "http://localhost:3000/hello"`

use api_versioning::{Options, api_version};
use api_versioning_axum::{RequestedApiVersion, Router};

async fn hello_v1() -> &'static str {
    "Hello world"
}

async fn hello_v2(requested: RequestedApiVersion) -> String {
    format!("Hello world from {}", requested.version())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut options = Options::default();

    options.report_versions.enabled = true;

    let app: axum::Router = Router::new(options)
        .api("/hello", |api| {
            api.name("Hello")
                .has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| e.get(hello_v1).map(api_version!(2.0), |e| e.get(hello_v2)))
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
