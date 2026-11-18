use super::Context;
use axum::response::Response;
use std::fmt::{self, Debug, Formatter};

/// Defines the behavior of an object that creates the response for an
/// [API versioning error](crate::ApiVersionError).
///
/// # Remarks
///
/// A responder is implemented for any function or closure that accepts a [Context] and returns a [Response]. Use
/// [Context::problem] and [Context::respond_with] to customize the default [problem details](ProblemDetailsResponder)
/// rather than replace them.
///
/// # Examples
///
/// ```
/// use api_versioning::Options;
/// use api_versioning_axum::{Router, error::Context};
///
/// let router: Router = Router::new(Options::default()).with_error_responder(|context: &Context<'_>| {
///     let mut problem = context.problem();
///
///     if let Some(id) = context.request().and_then(|request| request.headers.get("x-request-id")) {
///         problem.extensions.insert("requestId".into(), id.to_str().unwrap_or_default().into());
///     }
///
///     context.respond_with(&problem)
/// });
/// ```
pub trait Responder: Send + Sync {
    /// Creates the response for an API versioning error.
    ///
    /// # Arguments
    ///
    /// * `context` - the [context](Context) of the error
    fn respond(&self, context: &Context<'_>) -> Response;
}

impl<F> Responder for F
where
    F: Fn(&Context<'_>) -> Response + Send + Sync,
{
    fn respond(&self, context: &Context<'_>) -> Response {
        self(context)
    }
}

impl Debug for dyn Responder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Responder")
    }
}

/// Represents the default [responder](Responder), which responds with [problem details](Context::problem).
#[derive(Clone, Copy, Debug, Default)]
pub struct ProblemDetailsResponder;

impl Responder for ProblemDetailsResponder {
    fn respond(&self, context: &Context<'_>) -> Response {
        context.respond_with(&context.problem())
    }
}
