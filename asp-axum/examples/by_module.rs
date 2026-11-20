//! Run with `cargo run --package api-versioning-axum --example by_module`, then test with `by_module.http`

use api_versioning::{ApiVersion, Options};
use api_versioning_axum::{RequestedApiVersion, Router};
use axum::Json;
use axum::extract::Path;
use serde::{Deserialize, Serialize};

// The route parameters shared by both routes. Fields are matched by name, so the `api_version` segment captured by
// `/v{api_version}/...` is ignored, and the query string route, which has no such segment, also matches. This is only
// required because the example demonstrates both query string and url versioning simultaneously. Aligning to one
// approach would use:
//
// * query string - Path<u32>
// * url - Path<(ApiVersion<'static>, u32)>
#[derive(Deserialize)]
struct Params {
    id: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Account {
    endpoint: String,
    account_id: String,
    api_version: ApiVersion<'static>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Order {
    endpoint: String,
    account_id: String,
    api_version: ApiVersion<'static>,
}

mod v1 {
    use super::*;

    pub mod accounts {
        use super::*;

        // GET v1/accounts/{id}
        // GET accounts/{id}?api-version=1.0
        pub async fn by_id(Path(Params { id }): Path<Params>, requested: RequestedApiVersion) -> Json<Account> {
            Json(Account {
                endpoint: stringify!(v1::accounts::by_id).to_string(),
                account_id: id.to_string(),
                api_version: requested.version().clone(),
            })
        }
    }

    pub mod orders {
        use super::*;

        // GET v1/orders/{id}
        // GET orders/{id}?api-version=1.0
        pub async fn by_id(Path(Params { id }): Path<Params>, requested: RequestedApiVersion) -> Json<Order> {
            Json(Order {
                endpoint: stringify!(v1::orders::by_id).to_string(),
                account_id: id.to_string(),
                api_version: requested.version().clone(),
            })
        }
    }
}

mod v2 {
    use super::*;

    pub mod accounts {
        use super::*;

        // GET v2/accounts/{id}
        // GET accounts/{id}?api-version=2.0
        pub async fn by_id(Path(Params { id }): Path<Params>, requested: RequestedApiVersion) -> Json<Account> {
            Json(Account {
                endpoint: stringify!(v2::accounts::by_id).to_string(),
                account_id: id.to_string(),
                api_version: requested.version().clone(),
            })
        }
    }

    pub mod orders {
        use super::*;

        // GET v2/orders/{id}
        // GET orders/{id}?api-version=2.0
        pub async fn by_id(Path(Params { id }): Path<Params>, requested: RequestedApiVersion) -> Json<Order> {
            Json(Order {
                endpoint: stringify!(v2::orders::by_id).to_string(),
                account_id: id.to_string(),
                api_version: requested.version().clone(),
            })
        }
    }
}

mod v3 {
    use super::*;

    pub mod accounts {
        use super::*;

        // GET v3/accounts/{id}
        // GET accounts/{id}?api-version=3.0
        pub async fn by_id(Path(Params { id }): Path<Params>, requested: RequestedApiVersion) -> Json<Account> {
            Json(Account {
                endpoint: stringify!(v3::accounts::by_id).to_string(),
                account_id: id.to_string(),
                api_version: requested.version().clone(),
            })
        }
    }

    pub mod orders {
        use super::*;

        // GET v3/orders/{id}
        // GET orders/{id}?api-version=3.0
        pub async fn by_id(Path(Params { id }): Path<Params>, requested: RequestedApiVersion) -> Json<Order> {
            Json(Order {
                endpoint: stringify!(v3::orders::by_id).to_string(),
                account_id: id.to_string(),
                api_version: requested.version().clone(),
            })
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut options = Options::default();

    // enable api versioning and return the headers "api-supported-versions" and "api-deprecated-versions"
    options.report_versions.enabled = true;

    // Endpoints::infer supports deriving API versions directly from the handler function path. the API version can
    // appear in a module path or the function name itself. identifiers cannot start with numbers so the expected prefix
    // is `_` or `v`. `_`, however, typically refers to an unused identifier. numbers, dates, and any other format that
    // can be parsed via ApiVersion::infer_from_type or ApiVersion::infer_from_val may be used.

    // NOTE: you should NOT use both the query string and url segment methods together. this configuration is merely
    // illustrating that they can coexist and allows you to easily experiment with either configuration. one of these
    // would be removed in a real application.
    let app: axum::Router = Router::new(options)
        .api("/accounts", |api| {
            api.route("/{id}", |e| {
                e.infer(|h| {
                    h.get(v1::accounts::by_id)
                        .get(v2::accounts::by_id)
                        .get(v3::accounts::by_id)
                })
            })
        })
        .api("/v{api_version}/accounts", |api| {
            api.route("/{id}", |e| {
                e.infer(|h| {
                    h.get(v1::accounts::by_id)
                        .get(v2::accounts::by_id)
                        .get(v3::accounts::by_id)
                })
            })
        })
        .api("/orders", |api| {
            api.route("/{id}", |e| {
                e.infer(|h| h.get(v1::orders::by_id).get(v2::orders::by_id).get(v3::orders::by_id))
            })
        })
        .api("/v{api_version}/orders", |api| {
            api.route("/{id}", |e| {
                e.infer(|h| h.get(v1::orders::by_id).get(v2::orders::by_id).get(v3::orders::by_id))
            })
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
