mod accept;
mod context;
mod responder;

pub use context::Context;
pub use responder::{ProblemDetailsResponder, Responder};

use crate::versioning::ApiVersioning;
use api_versioning::problem::ProblemType;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;

/// Represents the possible errors that can occur when matching a request to a versioned endpoint.
///
/// # Remarks
///
/// The message of each error is the detail of its [problem details](api_versioning::problem::ProblemDetails).
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ApiVersionError {
    /// Indicates an API version was required, but not specified.
    #[error("An API version is required, but was not specified.")]
    Unspecified,

    /// Indicates the requested API version is not in a valid format.
    #[error("The HTTP resource that matches the request URI '{path}' does not support the API version '{version}'.")]
    Invalid {
        /// Gets the path of the requested resource.
        path: String,

        /// Gets the raw, requested API version.
        version: String,
    },

    /// Indicates more than one distinct API version was requested.
    #[error(
        "The following API versions were requested: {}. At most, only a single API version may be specified. Please \
         update the intended API version and retry the request.",
        .0.join(", ")
    )]
    Ambiguous(
        /// Gets the distinct, raw API versions that were requested.
        Vec<String>,
    ),

    /// Indicates the requested API version is not supported by any endpoint.
    #[error("The HTTP resource that matches the request URI '{path}' does not support the API version '{version}'.")]
    Unsupported {
        /// Gets the path of the requested resource.
        path: String,

        /// Gets the raw, requested API version or the assumed API version if none was requested.
        version: String,
    },
}

impl ApiVersionError {
    /// Gets the [type of problem](ProblemType) that corresponds to the error.
    pub fn problem_type(&self) -> ProblemType {
        match self {
            Self::Unspecified => ProblemType::UNSPECIFIED,
            Self::Invalid { .. } => ProblemType::INVALID,
            Self::Ambiguous(_) => ProblemType::AMBIGUOUS,
            Self::Unsupported { .. } => ProblemType::UNSUPPORTED,
        }
    }

    /// Gets the HTTP status code for the error using the specified API versioning configuration.
    ///
    /// # Arguments
    ///
    /// * `versioning` - the API versioning configuration
    ///
    /// # Remarks
    ///
    /// When the URL is the only source of an API version, the API version is part of the resource identifier. A URL with
    /// a missing, invalid, or unsupported API version therefore identifies a resource that does not exist.
    pub(crate) fn status(&self, versioning: &ApiVersioning) -> StatusCode {
        if versioning.versions_by_url_only() && !matches!(self, Self::Ambiguous(_)) {
            StatusCode::NOT_FOUND
        } else {
            self.status_with(versioning.options().unsupported_status_code)
        }
    }

    fn status_with(&self, unsupported_status_code: u16) -> StatusCode {
        match self {
            Self::Unsupported { .. } => {
                StatusCode::from_u16(unsupported_status_code).unwrap_or(StatusCode::BAD_REQUEST)
            }
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for ApiVersionError {
    /// Converts the error into [problem details](api_versioning::problem::ProblemDetails) with HTTP status code 400.
    ///
    /// # Remarks
    ///
    /// The request is unavailable, so the configured [responder](Responder) and content negotiation are not used.
    fn into_response(self) -> Response {
        let status = self.status_with(StatusCode::BAD_REQUEST.as_u16());
        ProblemDetailsResponder.respond(&Context::new(&self, status, None))
    }
}
