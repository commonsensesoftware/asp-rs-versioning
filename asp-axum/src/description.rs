use crate::versioning::ApiVersioning;
use api_versioning::{
    ApiVersion, Metadata, Options,
    policy::{DeprecationPolicy, Manager, SunsetPolicy},
};
use http::{Extensions, Method};
use std::{slice::Iter, sync::Arc};

/// Represents the description of a single endpoint in a versioned API.
#[derive(Clone, Debug)]
pub struct ApiDescription {
    method: Method,
    path: String,
    metadata: Arc<Metadata>,
    extensions: Extensions,
}

impl ApiDescription {
    pub(crate) fn new(method: Method, path: String, metadata: Arc<Metadata>, extensions: Extensions) -> Self {
        Self {
            method,
            path,
            metadata,
            extensions,
        }
    }

    /// Gets the HTTP [method](Method) of the endpoint.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Gets the route template of the endpoint, relative to the root [Router](crate::Router).
    ///
    /// # Remarks
    ///
    /// The route template uses the Axum syntax, such as `/orders/{id}` or `/files/{*path}`.
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Gets the logical name of the API the endpoint belongs to, which is empty if the API is unnamed.
    #[inline]
    pub fn name(&self) -> &str {
        self.metadata.name()
    }

    /// Gets the API version [metadata](Metadata) of the endpoint.
    #[inline]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Gets the additional metadata attached to the handler of the endpoint, such as with [Described](crate::Described).
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }
}

/// Represents the descriptions of all of the endpoints of the versioned APIs in a [Router](crate::Router).
///
/// # Remarks
///
/// The descriptions are created by [Router::split](crate::Router::split) and are intended to be used by components
/// that describe the APIs, such as OpenAPI document generators.
#[derive(Clone, Debug)]
pub struct ApiDescriptions {
    versioning: ApiVersioning,
    versions: Vec<ApiVersion<'static>>,
    items: Vec<ApiDescription>,
}

impl ApiDescriptions {
    pub(crate) fn new(versioning: ApiVersioning, items: Vec<ApiDescription>) -> Self {
        Self {
            versions: versioning.declared(),
            versioning,
            items,
        }
    }

    /// Gets the API versions declared by all of the APIs in ascending order.
    ///
    /// # Remarks
    ///
    /// API versions that are only advertised are not declared because they are implemented elsewhere.
    #[inline]
    pub fn versions(&self) -> &[ApiVersion<'static>] {
        &self.versions
    }

    /// Gets the API versioning [options](Options).
    #[inline]
    pub fn options(&self) -> &Options {
        self.versioning.options()
    }

    /// Gets the [manager](Manager) for [deprecation policies](DeprecationPolicy).
    #[inline]
    pub fn deprecation_policies(&self) -> &Manager<DeprecationPolicy> {
        self.versioning.deprecation_policies()
    }

    /// Gets the [manager](Manager) for [sunset policies](SunsetPolicy).
    #[inline]
    pub fn sunset_policies(&self) -> &Manager<SunsetPolicy> {
        self.versioning.sunset_policies()
    }

    /// Gets the number of described endpoints.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Gets a value indicating whether there are no described endpoints.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Gets an iterator over the [descriptions](ApiDescription) in the order the endpoints were defined.
    #[inline]
    pub fn iter(&self) -> Iter<'_, ApiDescription> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a ApiDescriptions {
    type Item = &'a ApiDescription;
    type IntoIter = Iter<'a, ApiDescription>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
