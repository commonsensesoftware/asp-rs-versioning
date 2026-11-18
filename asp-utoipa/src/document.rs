use api_versioning::ApiVersion;
use std::fmt::{self, Debug, Formatter};
use utoipa::openapi::OpenApi;

/// Represents the OpenAPI document for a single API version.
#[derive(Clone)]
pub struct ApiDocument {
    version: ApiVersion<'static>,
    name: String,
    url: String,
    deprecated: bool,
    openapi: OpenApi,
}

impl ApiDocument {
    pub(crate) fn new(version: ApiVersion<'static>, deprecated: bool, openapi: OpenApi) -> Self {
        let name = format!("v{version:#}");

        Self {
            url: format!("/openapi/{name}.json"),
            name,
            version,
            deprecated,
            openapi,
        }
    }

    /// Gets the [API version](ApiVersion) the document describes.
    #[inline]
    pub fn version(&self) -> &ApiVersion<'static> {
        &self.version
    }

    /// Gets the name of the document, which is the API version in its shortest form prefixed with `v`, such as `v1`,
    /// `v1.1`, or `v2026-10-01`.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the suggested URL path the document is served at, which is `/openapi/{name}.json`.
    ///
    /// # Remarks
    ///
    /// The URL is consistent across documents, such as for the list of URLs in Swagger UI, and is the URL that
    /// [routes](crate::routes) serves the document at. A document can also be served at any other URL.
    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Gets a value indicating whether the API version is deprecated by every API that declares it.
    #[inline]
    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    /// Gets the [OpenAPI](OpenApi) document.
    #[inline]
    pub fn openapi(&self) -> &OpenApi {
        &self.openapi
    }

    /// Consumes the document and returns the [OpenAPI](OpenApi) document.
    #[inline]
    pub fn into_openapi(self) -> OpenApi {
        self.openapi
    }
}

impl Debug for ApiDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiDocument")
            .field("version", &self.version)
            .field("name", &self.name)
            .field("deprecated", &self.deprecated)
            .finish_non_exhaustive()
    }
}
