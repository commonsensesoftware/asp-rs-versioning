//! Run with `cargo run --package api-versioning-axum --example basic`, then test with `basic.http`

use api_versioning::{ApiVersion, api_version};
use api_versioning_axum::{RequestedApiVersion, Router};
use axum::Json;
use axum::extract::{OriginalUri, Path};
use axum::response::IntoResponse;
use http::StatusCode;
use http::header::LOCATION;
use serde_json::{Value, json};

// GET api/values?api-version=1.0
async fn values(requested: RequestedApiVersion) -> String {
    format!("Endpoint = {}\nVersion = {}", stringify!(values), requested.version())
}

// GET api/values?api-version=2.0
async fn values_v2(requested: RequestedApiVersion) -> String {
    format!(
        "Endpoint = {}\nVersion = {}",
        stringify!(values_v2),
        requested.version()
    )
}

mod hello_world {
    use super::*;

    // GET api/v{api_version}/helloworld
    pub async fn get(Path(version): Path<ApiVersion<'static>>) -> Json<Value> {
        Json(json!({
            "endpoint": stringify!(hello_world::get),
            "version": version
        }))
    }

    // GET api/v{api_version}/helloworld/{id}
    pub async fn get_by_id(Path((version, id)): Path<(ApiVersion<'static>, u32)>) -> Json<Value> {
        Json(json!({
            "endpoint": stringify!(hello_world::get_by_id),
            "id": id,
            "version": version
        }))
    }

    // POST api/v{api_version}/helloworld
    pub async fn create(OriginalUri(url): OriginalUri, Path(version): Path<ApiVersion<'static>>) -> impl IntoResponse {
        let location = format!("{}/42", url.path().trim_end_matches('/'));
        let body = Json(json!({
            "endpoint": stringify!(hello_world::create),
            "id": 42,
            "version": version
        }));

        (StatusCode::CREATED, [(LOCATION, location)], body)
    }
}

mod multi_versioned {
    use super::*;

    // GET api/v{api_version}/multi-versioned
    pub async fn get(requested: RequestedApiVersion) -> String {
        format!("Version {}", requested.version())
    }

    // GET api/v{api_version}/multi-versioned
    pub async fn get_v2(requested: RequestedApiVersion) -> String {
        format!("Version {}", requested.version())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app: axum::Router = Router::default()
        .api("/api/values", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| e.get(values).map(api_version!(2.0), |e| e.get(values_v2)))
        })
        .api("/api/v{api_version}/helloworld", |api| {
            api.has(api_version!(1.0))
                .route("/", |e| e.get(hello_world::get).post(hello_world::create))
                .route("/{id}", |e| e.get(hello_world::get_by_id))
        })
        .api("/api/v{api_version}/multi-versioned", |api| {
            api.has([api_version!(1.0), api_version!(2.0)]).route("/", |e| {
                e.get(multi_versioned::get)
                    .map(api_version!(2.0), |e| e.get(multi_versioned::get_v2))
            })
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
