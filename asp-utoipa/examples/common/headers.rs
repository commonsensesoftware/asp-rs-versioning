use http::Uri;
use std::fmt::Display;

/// Creates the location of a resource created by a request to its collection, such as `/orders/42`.
///
/// # Arguments
///
/// * `url` - the original URL of the request, which includes the path of any nested router
/// * `id` - the identifier of the created resource
///
/// # Remarks
///
/// The location is relative, so the client resolves it against the URL it requested. The server cannot determine the
/// scheme or host the client used, such as behind a proxy, so an absolute URL would require a configured base URL.
pub fn location(url: &Uri, id: impl Display) -> String {
    format!("{}/{id}", url.path().trim_end_matches('/'))
}
