use axum::response::{IntoResponse, Response};
use http::header::LOCATION;
use http::{HeaderValue, StatusCode};

/// Represents the response for a created resource.
pub struct Created<T: IntoResponse> {
    pub location: String,
    pub content: T,
}

impl Created<()> {
    /// Initializes a created resources with an empty body.
    ///
    /// # Arguments
    ///
    /// * `location` - the Location of the new resource
    pub fn at<S: Into<String>>(location: S) -> Self {
        Self {
            location: location.into(),
            content: (),
        }
    }

    /// Amends the created resource with the specified content.
    ///
    /// # Arguments
    ///
    /// * `model` - the created resource
    pub fn with<T: IntoResponse>(self, content: T) -> Created<T> {
        Created {
            location: self.location,
            content,
        }
    }
}

impl<T: IntoResponse> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        let location = HeaderValue::from_str(&self.location).unwrap();
        let mut response = self.content.into_response();

        *response.status_mut() = StatusCode::CREATED;

        let headers = response.headers_mut();

        headers.insert(LOCATION, location);
        response
    }
}
