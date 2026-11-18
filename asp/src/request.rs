use http::{HeaderMap, Uri};

/// Defines the behavior to access read-only HTTP request information.
pub trait RequestInfo {
    /// Gets the URI of the request.
    fn uri(&self) -> &Uri;

    /// Gets the headers of the request.
    fn headers(&self) -> &HeaderMap;

    /// Gets the value of a named parameter captured by the routing system, if any.
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the route parameter
    ///
    /// # Remarks
    ///
    /// The default implementation does not provide any route parameters. Routing is the responsibility of a web
    /// framework, which is expected to provide the parameters it captured when it matched the request.
    fn route_parameter(&self, _name: &str) -> Option<&str> {
        None
    }
}

impl<T> RequestInfo for http::Request<T> {
    fn uri(&self) -> &Uri {
        self.uri()
    }

    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
