use crate::ApiDocument;
use axum::{
    Router,
    body::Bytes,
    http::{HeaderValue, header::CONTENT_TYPE},
    routing::get,
};

/// Creates a router that serves each OpenAPI document as JSON at its [URL](ApiDocument::url).
///
/// # Arguments
///
/// * `documents` - the [documents](ApiDocument) to serve
///
/// # Remarks
///
/// Each document is serialized once, when the router is created. The routes are not versioned, so the router is merged
/// into the application rather than added to an API.
///
/// # Examples
///
/// ```
/// use api_versioning::{Options, api_version};
/// use api_versioning_axum::Router;
/// use api_versioning_utoipa::{doc, documents, routes};
/// use utoipa::openapi::OpenApi;
///
/// #[utoipa::path(get, path = "", responses((status = 200, body = String)))]
/// async fn get_orders() -> &'static str {
///     "orders"
/// }
///
/// let (app, apis): (axum::Router, _) = Router::new(Options::default())
///     .api("/orders", |api| api.has(api_version!(1.0)).route("/", |e| e.get(doc!(get_orders))))
///     .split();
/// let documents = documents(&apis, OpenApi::default());
/// let app = app.merge(routes(&documents)); // serves /openapi/v1.json
/// ```
///
/// # Panics
///
/// Panics if a document cannot be serialized, which only happens if an extension or example cannot be represented as
/// JSON.
pub fn routes<S>(documents: &[ApiDocument]) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    documents.iter().fold(Router::new(), |router, document| {
        let json = Bytes::from(serde_json::to_vec(document.openapi()).expect("an OpenAPI document is valid JSON"));
        let handler = move || {
            let json = json.clone();
            async move { ([(CONTENT_TYPE, HeaderValue::from_static("application/json"))], json) }
        };

        router.route(document.url(), get(handler))
    })
}
