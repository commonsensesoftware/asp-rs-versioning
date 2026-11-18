//! Defines the versioned APIs and OpenAPI documents shared by the examples.

mod created;
mod headers;
mod models;
mod v1;
mod v2;
mod v3;

use api_versioning::{Options, api_version};
use api_versioning_axum::{Router, VersionedApi};
use api_versioning_utoipa::{ApiDocument, doc, documents};
pub(crate) use created::Created;
pub(crate) use headers::location;
use std::time::{Duration, SystemTime};
use utoipa::OpenApi;

/// The address the examples listen on.
pub const ADDRESS: &str = "127.0.0.1:3000";

#[derive(OpenApi)]
#[openapi(info(title = "Example API", description = "An example API."))]
struct ApiDoc;

fn add_policies(options: &mut Options) {
    const DAYS_60: Duration = Duration::from_secs(60 * 24 * 60 * 60);

    options
        .policies
        .deprecate(api_version!(0.9))
        .as_of(SystemTime::now())
        .link("https://example.com/policy.html")
        .title("Version Deprecation Policy")
        .media_type("text/html");

    options
        .policies
        .sunset(api_version!(0.9))
        .as_of(SystemTime::now() + DAYS_60)
        .link("https://example.com/policy.html")
        .title("Version Sunset Policy")
        .media_type("text/html");
}

fn orders(api: VersionedApi) -> VersionedApi {
    api.name("Orders")
        .deprecates(api_version!(0.9))
        .has([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
        .route("/", |e| {
            e.map(api_version!(1.0), |h| {
                h.post(doc!(v1::orders::place)).patch(doc!(v1::orders::update))
            })
            .map(api_version!(2.0), |h| {
                h.get(doc!(v2::orders::all)).post(doc!(v2::orders::place))
            })
            .map(api_version!(3.0), |h| {
                h.get(doc!(v3::orders::all)).post(doc!(v3::orders::place))
            })
        })
        .route("/{id}", |e| {
            e.get(doc!(v1::orders::by_id))
                .map(api_version!(2.0), |h| {
                    h.get(doc!(v2::orders::by_id)).patch(doc!(v2::orders::update))
                })
                .map(api_version!(3.0), |h| {
                    h.get(doc!(v3::orders::by_id)).delete(doc!(v3::orders::cancel))
                })
        })
}

fn people(api: VersionedApi) -> VersionedApi {
    api.name("People")
        .has([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
        .route("/", |e| {
            e.map(api_version!(2.0), |h| h.get(doc!(v2::people::all)))
                .map(api_version!(3.0), |h| {
                    h.get(doc!(v3::people::all)).post(doc!(v3::people::add))
                })
        })
        .route("/{id}", |e| {
            e.get(doc!(v1::people::by_id))
                .map(api_version!(2.0), |h| h.get(doc!(v2::people::by_id)))
                .map(api_version!(3.0), |h| h.get(doc!(v3::people::by_id)))
        })
}

/// Creates the application and an OpenAPI document for each API version.
pub fn app() -> (axum::Router, Vec<ApiDocument>) {
    let mut options = Options::default();

    // reporting api versions will return the headers
    // "api-supported-versions" and "api-deprecated-versions"
    options.report_versions.enabled = true;

    add_policies(&mut options);

    let (app, apis) = Router::new(options)
        .api("/api/orders", orders)
        .api("/api/v{api_version}/people", people)
        .split();

    (app, documents(&apis, ApiDoc::openapi()))
}
