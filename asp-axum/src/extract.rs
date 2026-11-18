use crate::ApiVersionError;
use api_versioning::{ApiVersion, Metadata};
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use http::request::Parts;
use smallvec::SmallVec;
use std::convert::Infallible;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use std::str;
use std::sync::Arc;

/// Represents the API version requested by a client.
///
/// # Remarks
///
/// The requested API version is available from the request extensions of a handler registered as a versioned endpoint.
/// It is not available when a version-neutral endpoint is matched without an API version, which can be handled by
/// extracting `Option<RequestedApiVersion>`. Extracting `RequestedApiVersion` in that case results in HTTP status code
/// 400.
#[derive(Clone, PartialEq, Eq)]
pub struct RequestedApiVersion {
    // the raw value is short enough to be stored inline in almost all cases
    raw: Option<SmallVec<[u8; 24]>>,
    version: ApiVersion<'static>,
}

impl RequestedApiVersion {
    pub(crate) fn new(raw: Option<&str>, version: &ApiVersion) -> Self {
        Self {
            raw: raw.map(|text| SmallVec::from_slice(text.as_bytes())),
            version: version.to_owned(),
        }
    }

    /// Gets the requested [API version](ApiVersion).
    ///
    /// # Remarks
    ///
    /// This is the default API version if the API version is assumed because a client did not specify one.
    #[inline]
    pub fn version(&self) -> &ApiVersion<'static> {
        &self.version
    }

    /// Gets the requested API version in the form it was read from the request, if any.
    ///
    /// # Remarks
    ///
    /// The raw value is meant for purposes such as logging. It might have a different form than the
    /// [API version](Self::version); for example, `1` instead of `1.0`. If equivalent API versions are requested, this is
    /// the first value. A value read from a URL has been percent-decoded by the router. There is no raw value if the API
    /// version is assumed because a client did not specify one.
    #[inline]
    pub fn raw(&self) -> Option<&str> {
        self.raw
            .as_deref()
            .map(|bytes| str::from_utf8(bytes).expect("the raw value originated from a string"))
    }
}

impl Debug for RequestedApiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestedApiVersion")
            .field("raw", &self.raw())
            .field("version", &self.version)
            .finish()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for RequestedApiVersion {
    type Rejection = ApiVersionError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Self>()
            .cloned()
            .ok_or(ApiVersionError::Unspecified)
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for RequestedApiVersion {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Self::Rejection> {
        Ok(parts.extensions.get::<Self>().cloned())
    }
}

/// Represents the API version [metadata](Metadata) of the endpoint that matched a request.
///
/// # Remarks
///
/// The metadata is available in the request extensions of the matched handler.
#[derive(Clone, Debug)]
pub struct EndpointMetadata(pub Arc<Metadata>);

impl Deref for EndpointMetadata {
    type Target = Metadata;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Send + Sync> FromRequestParts<S> for EndpointMetadata {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Self>()
            .cloned()
            .unwrap_or_else(|| Self(Arc::new(Metadata::empty()))))
    }
}
