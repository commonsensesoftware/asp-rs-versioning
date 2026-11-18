use crate::error::Responder;
use api_versioning::{
    ApiVersion, Options, Reporter,
    parameter::{Descriptor, prelude::DescriptorExt},
    policy::{DeprecationPolicy, Manager, SunsetPolicy},
    selectors::Selector,
};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
struct Inner {
    options: Options,
    reporter: Reporter,
    deprecation: Arc<Manager<DeprecationPolicy>>,
    sunset: Arc<Manager<SunsetPolicy>>,
    selector: Box<dyn Selector>,
    responder: Box<dyn Responder>,
    declared: RwLock<Vec<ApiVersion<'static>>>,
    versions_by_url: bool,
    versions_by_url_only: bool,
}

/// Represents the API versioning configuration shared by all of the APIs in a [Router](crate::Router).
///
/// # Remarks
///
/// All of the versioned APIs in the same router are aggregated together. For example, a version-neutral endpoint can
/// match any API version declared by any of the aggregated APIs.
#[derive(Clone, Debug)]
pub(crate) struct ApiVersioning(Arc<Inner>);

impl ApiVersioning {
    /// Initializes a new [ApiVersioning] configuration.
    ///
    /// # Arguments
    ///
    /// * `options` - the API versioning [options](Options)
    /// * `responder` - the [responder](Responder) that creates the response for an API versioning error
    ///
    /// # Remarks
    ///
    /// The options are validated when the root [Router](crate::Router) is created.
    pub fn new(options: Options, responder: Box<dyn Responder>) -> Self {
        let deprecation = Arc::new(Manager::<DeprecationPolicy>::new(&options));
        let sunset = Arc::new(Manager::<SunsetPolicy>::new(&options));
        let reporter = Reporter::new(&options.report_versions.header, deprecation.clone(), sunset.clone());
        let selector = options.selector_or_default();
        let descriptor: &dyn Descriptor = &*options.reader;
        let versions_by_url_only = descriptor.versions_by_url(false);
        let versions_by_url = versions_by_url_only || descriptor.versions_by_url(true);

        Self(Arc::new(Inner {
            options,
            reporter,
            deprecation,
            sunset,
            selector,
            responder,
            declared: RwLock::new(Vec::new()),
            versions_by_url,
            versions_by_url_only,
        }))
    }

    /// Gets the [responder](Responder) that creates the response for an API versioning error.
    #[inline]
    pub fn error_responder(&self) -> &dyn Responder {
        &*self.0.responder
    }

    /// Gets a value indicating whether an API version can be read from the URL.
    #[inline]
    pub fn versions_by_url(&self) -> bool {
        self.0.versions_by_url
    }

    /// Gets a value indicating whether an API version can only be read from the URL.
    #[inline]
    pub fn versions_by_url_only(&self) -> bool {
        self.0.versions_by_url_only
    }

    /// Gets the API versioning [options](Options).
    #[inline]
    pub fn options(&self) -> &Options {
        &self.0.options
    }

    /// Gets the API version [reporter](Reporter).
    #[inline]
    pub fn reporter(&self) -> &Reporter {
        &self.0.reporter
    }

    /// Gets the [manager](Manager) for [deprecation policies](DeprecationPolicy).
    #[inline]
    pub fn deprecation_policies(&self) -> &Manager<DeprecationPolicy> {
        &self.0.deprecation
    }

    /// Gets the [manager](Manager) for [sunset policies](SunsetPolicy).
    #[inline]
    pub fn sunset_policies(&self) -> &Manager<SunsetPolicy> {
        &self.0.sunset
    }

    /// Gets the API version [selector](Selector) used when an API version is assumed.
    #[inline]
    pub fn selector(&self) -> &dyn Selector {
        &*self.0.selector
    }

    /// Gets the API versions declared by all of the aggregated APIs in ascending order.
    pub fn declared(&self) -> Vec<ApiVersion<'static>> {
        self.0.declared.read().unwrap().clone()
    }

    /// Determines whether the specified API version is declared by any aggregated API.
    ///
    /// # Arguments
    ///
    /// * `version` - the [API version](ApiVersion) to evaluate
    pub fn is_declared(&self, version: &ApiVersion) -> bool {
        self.0
            .declared
            .read()
            .unwrap()
            .binary_search_by(|v| v.total_cmp(version))
            .is_ok()
    }

    /// Declares the specified API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - the sequence of [API versions](ApiVersion) to declare
    pub fn declare<'a>(&self, versions: impl IntoIterator<Item = &'a ApiVersion<'a>>) {
        let mut declared = self.0.declared.write().unwrap();

        for version in versions {
            if let Err(index) = declared.binary_search_by(|v| v.total_cmp(version)) {
                declared.insert(index, version.to_owned());
            }
        }
    }
}
