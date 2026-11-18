//! API versioning for [Axum](axum).
//!
//! # Remarks
//!
//! Axum does not allow the same path and method to be registered more than once. API versioning allows exactly that, so
//! all of the versioned handlers for a path and method are registered as a single Axum route. When a request is routed,
//! the API version is read from the request and the handler mapped to that version is dispatched. API version
//! information is purely metadata attached to each handler at the time it is registered.
//!
//! Versioned APIs are added to a [Router], which wraps an Axum [Router](axum::Router). The router knows the complete
//! path and API version information of every endpoint, which it [describes](ApiDescriptions) when it is
//! [split](Router::split).
//!
//! # Examples
//!
//! ```
//! use api_versioning::{Options, api_version};
//! use api_versioning_axum::Router;
//!
//! async fn get_orders() -> &'static str {
//!     "orders"
//! }
//!
//! async fn get_orders_v2() -> &'static str {
//!     "orders, version 2"
//! }
//!
//! async fn cancel_order() {}
//!
//! async fn health() {}
//!
//! let app: axum::Router = Router::new(Options::default())
//!     .api("/orders", |api| {
//!         api.name("Orders")
//!             .deprecates(api_version!(0.9))
//!             .has([api_version!(1.0), api_version!(2.0)])
//!             .route("/", |e| e.get(get_orders).map(api_version!(2.0), |e| e.get(get_orders_v2)))
//!             .route("/{id}", |e| e.neutral(|e| e.delete(cancel_order)))
//!     })
//!     .api("/health", |api| api.neutral().route("/", |e| e.get(health)))
//!     .into();
//! ```
//!
//! # Versioning by URL
//!
//! The route template defines where an API version appears in a URL by capturing a parameter, which is named
//! `api_version` by default. Like any other parameter captured by Axum, it is part of the [Path](axum::extract::Path)
//! of a route so it must be accounted for when a handler extracts other parameters. [ApiVersion](api_versioning::ApiVersion)
//! can be extracted directly.
//!
//! When the URL is the only source of an API version, a URL with a missing, invalid, or unsupported API version does not
//! identify a resource and results in HTTP status code 404. Otherwise, the configured status codes are used.
//!
//! ```
//! use api_versioning::{ApiVersion, Options, api_version, readers::UrlReader};
//! use api_versioning_axum::Router;
//! use axum::extract::Path;
//!
//! async fn get_order(Path((version, id)): Path<(ApiVersion<'static>, u32)>) -> String {
//!     format!("order {id} using API version {version}")
//! }
//!
//! let app: axum::Router = Router::new(Options {
//!     reader: Box::new(UrlReader::default()),
//!     ..Default::default()
//! })
//! .api("/v{api_version}/orders", |api| api.has(api_version!(1.0)).route("/{id}", |e| e.get(get_order)))
//! .into();
//! ```

mod api;
mod description;
mod dispatch;
mod endpoints;

/// Contains functionality related to API versioning errors and their responses.
pub mod error;

mod extract;
mod router;
mod versioning;

pub use api::VersionedApi;
pub use description::{ApiDescription, ApiDescriptions};
pub use endpoints::{Described, Endpoints, Handlers, IntoEndpoint};
pub use error::ApiVersionError;
pub use extract::{EndpointMetadata, RequestedApiVersion};
pub use router::Router;
