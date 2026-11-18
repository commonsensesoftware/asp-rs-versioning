use super::{ApiVersionError, accept};
use api_versioning::problem::{self, ProblemDetails};
use axum::response::{IntoResponse, Response};
use http::{HeaderValue, StatusCode, header::CONTENT_TYPE, request::Parts};

/// Represents the context used to create the response for an [API versioning error](ApiVersionError).
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    error: &'a ApiVersionError,
    status: StatusCode,
    request: Option<&'a Parts>,
}

impl<'a> Context<'a> {
    /// Initializes a new [Context].
    ///
    /// # Arguments
    ///
    /// * `error` - the [API versioning error](ApiVersionError) to respond to
    /// * `status` - the HTTP [status code](StatusCode) for the response
    /// * `request` - the [parts](Parts) of the request that caused the error, if available
    pub fn new(error: &'a ApiVersionError, status: StatusCode, request: Option<&'a Parts>) -> Self {
        Self { error, status, request }
    }

    /// Gets the [API versioning error](ApiVersionError) to respond to.
    #[inline]
    pub fn error(&self) -> &'a ApiVersionError {
        self.error
    }

    /// Gets the HTTP [status code](StatusCode) for the response.
    ///
    /// # Remarks
    ///
    /// The status code is determined by the error and the API versioning [options](api_versioning::Options), such as
    /// the [status code for an unsupported API version](api_versioning::Options::unsupported_status_code).
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Gets the [parts](Parts) of the request that caused the error, if available.
    ///
    /// # Remarks
    ///
    /// The request is unavailable when an error is converted into a response outside of request dispatching, such as
    /// when an extractor rejects a request.
    #[inline]
    pub fn request(&self) -> Option<&'a Parts> {
        self.request
    }

    /// Creates the default [problem details](ProblemDetails) for the error.
    pub fn problem(&self) -> ProblemDetails {
        ProblemDetails::new(self.error.problem_type(), self.status.as_u16(), self.error.to_string())
    }

    /// Determines whether the request accepts problem details serialized as JSON.
    ///
    /// # Remarks
    ///
    /// Problem details are accepted when the request has no `Accept` header or when the most specific media range that
    /// matches `application/problem+json` or `application/json` has a nonzero quality. Problem details are always
    /// accepted when the request is unavailable.
    pub fn accepts_problem_details(&self) -> bool {
        self.request
            .is_none_or(|request| accept::problem_details(&request.headers))
    }

    /// Creates a response from the specified problem details.
    ///
    /// # Arguments
    ///
    /// * `problem` - the [problem details](ProblemDetails) to respond with
    ///
    /// # Remarks
    ///
    /// The response always has the [status code](Self::status) of the context. The problem details are serialized as
    /// `application/problem+json` if they are [accepted](Self::accepts_problem_details); otherwise, the response has no
    /// body.
    pub fn respond_with(&self, problem: &ProblemDetails) -> Response {
        if !self.accepts_problem_details() {
            return self.status.into_response();
        }

        match serde_json::to_vec(problem) {
            Ok(body) => (
                self.status,
                [(CONTENT_TYPE, HeaderValue::from_static(problem::JSON))],
                body,
            )
                .into_response(),
            Err(_) => self.status.into_response(),
        }
    }
}
