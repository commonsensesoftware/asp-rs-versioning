use crate::{
    ApiVersion,
    policy::Policies,
    prelude::*,
    readers::{QueryStringReader, Reader, UrlReader},
    selectors::{ConstantSelector, Selector},
};

/// Represents the names of the HTTP headers used to report API versions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportHeaders {
    /// Gets or sets the name of the HTTP header that reports the supported API versions.
    ///
    /// # Remarks
    ///
    /// The default value is `api-supported-versions`.
    pub supported: &'static str,

    /// Gets or sets the name of the HTTP header that reports the deprecated API versions.
    ///
    /// # Remarks
    ///
    /// The default value is `api-deprecated-versions`.
    pub deprecated: &'static str,
}

impl Default for ReportHeaders {
    fn default() -> Self {
        Self {
            supported: "api-supported-versions",
            deprecated: "api-deprecated-versions",
        }
    }
}

/// Represents the options for reporting API versions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReportOptions {
    /// Gets or sets a value indicating whether requests report the API version compatibility information in responses.
    ///
    /// # Remarks
    ///
    /// When set to `true`, the [HTTP headers](Self::header) for the supported and deprecated API versions will be added
    /// to all valid routes. This information is useful for advertising which versions are supported and scheduled for
    /// deprecation to clients. This information is also useful when supporting the `OPTIONS` method. The default value
    /// is `false`.
    pub enabled: bool,

    /// Gets or sets the names of the HTTP headers used to report API versions.
    pub header: ReportHeaders,
}

/// Represents the possible options for API versioning.
#[derive(Clone, Debug)]
pub struct Options {
    /// Gets or sets the options for reporting API versions.
    pub report_versions: ReportOptions,

    /// Gets or sets a value indicating whether a default version is assumed when a client does not provide an API
    /// version.
    ///
    /// # Remarks
    ///
    /// When a default API version is assumed, the version used is based upon the result from the
    /// [API version selector](Self::selector). The default value is `false`.
    pub assume_default_version_when_unspecified: bool,

    /// Gets or sets the default API version applied to services that do not have explicit versions.
    ///
    /// # Remarks
    ///
    /// The default value is [ApiVersion::DEFAULT]. The default API version must be a specific API version; it cannot
    /// be [ApiVersion::NEUTRAL].
    pub default_version: ApiVersion<'static>,

    /// Gets or sets the API version selector used to select the API version when one is assumed.
    ///
    /// # Remarks
    ///
    /// The default value is `None`, which uses a [ConstantSelector] that returns the
    /// [default API version](Self::default_version).
    pub selector: Option<Box<dyn Selector>>,

    /// Gets or sets the API version reader used to read API versions from incoming requests.
    ///
    /// # Remarks
    ///
    /// The default value is a combined [Reader] that consists of the [QueryStringReader] and [UrlReader], which reads
    /// the `api-version` parameter and a path segment in the request URL respectively.
    ///
    /// Replace the default reader with an alternate implementation, such as the [HeaderReader](readers::HeaderReader),
    /// which can read the API version from additional information like HTTP headers.
    pub reader: Box<dyn Reader>,

    /// Gets or sets the builder used for API versioning policies.
    pub policies: Policies,

    /// Gets or sets the HTTP status code used for unsupported versions of an API.
    ///
    /// # Remarks
    ///
    /// Gets or sets the HTTP status code used for unsupported versions of an API.
    ///
    /// | Code | Status          | Description                          |
    /// |------|-----------------|--------------------------------------|
    /// | 400  | Bad Request     | The API doesn't support this version |
    /// | 404  | Not Found       | The API doesn't exist                |
    /// | 501  | Not Implemented | The API isn't implemented            |
    pub unsupported_status_code: u16,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            report_versions: ReportOptions::default(),
            assume_default_version_when_unspecified: false,
            default_version: ApiVersion::DEFAULT,
            selector: None,
            reader: QueryStringReader::default().combine(UrlReader::default()).into(),
            policies: Policies::default(),
            unsupported_status_code: 400,
        }
    }
}

impl Options {
    /// Validates the options.
    ///
    /// # Remarks
    ///
    /// Options are expected to be validated once, before they are used to configure API versioning. Invalid options
    /// are a programming error rather than a runtime condition.
    ///
    /// # Panics
    ///
    /// Panics if the [default API version](Self::default_version) is [neutral](ApiVersion::NEUTRAL).
    #[track_caller]
    pub fn validate(&self) {
        assert!(
            !self.default_version.is_neutral(),
            "Options::default_version cannot be ApiVersion::NEUTRAL; the default must be a specific API version."
        );
    }

    /// Gets the configured API version [selector](Self::selector) or, if there isn't one, a [ConstantSelector] that
    /// returns the [default API version](Self::default_version).
    pub fn selector_or_default(&self) -> Box<dyn Selector> {
        self.selector
            .clone()
            .unwrap_or_else(|| Box::new(ConstantSelector::new(self.default_version.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{DeprecationPolicy, SunsetPolicy};
    use crate::{Model, api_version, selectors::tests::select};

    #[test]
    fn default_options_should_use_expected_values() {
        // arrange

        // act
        let options = Options::default();

        // assert
        assert!(!options.report_versions.enabled);
        assert_eq!(options.report_versions.header.supported, "api-supported-versions");
        assert_eq!(options.report_versions.header.deprecated, "api-deprecated-versions");
        assert!(!options.assume_default_version_when_unspecified);
        assert_eq!(options.default_version, ApiVersion::DEFAULT);
        assert!(options.selector.is_none());
        assert_eq!(options.policies.of::<DeprecationPolicy>().count(), 0);
        assert_eq!(options.policies.of::<SunsetPolicy>().count(), 0);
        assert_eq!(options.unsupported_status_code, 400);
    }

    #[test]
    fn validate_should_accept_default_options() {
        // arrange
        let options = Options::default();

        // act
        options.validate();

        // assert
    }

    #[test]
    #[should_panic(
        expected = "Options::default_version cannot be ApiVersion::NEUTRAL; the default must be a specific API version."
    )]
    fn validate_should_panic_when_default_version_is_neutral() {
        // arrange
        let options = Options {
            default_version: ApiVersion::NEUTRAL,
            ..Default::default()
        };

        // act
        options.validate();

        // assert
    }

    #[test]
    fn selector_or_default_should_select_default_version_when_selector_is_not_configured() {
        // arrange
        let options = Options {
            default_version: api_version!(42.0),
            ..Default::default()
        };

        // act
        let selector = options.selector_or_default();

        // assert
        assert_eq!(select(&*selector, &Model::empty()), Some(api_version!(42.0)));
    }

    #[test]
    fn selector_or_default_should_return_configured_selector() {
        // arrange
        let options = Options {
            default_version: api_version!(42.0),
            selector: Some(Box::new(ConstantSelector::new(api_version!(1.0)))),
            ..Default::default()
        };

        // act
        let selector = options.selector_or_default();

        // assert
        assert_eq!(select(&*selector, &Model::empty()), Some(api_version!(1.0)));
    }
}
