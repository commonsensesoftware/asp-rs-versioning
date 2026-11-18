use crate::{
    ApiVersion, Metadata, ReportHeaders,
    headers::{DEPRECATION, Deprecation, LINK, Link, SUNSET, Sunset},
    policy::{DeprecationPolicy, Manager, SunsetPolicy},
};
use http::{HeaderMap, HeaderName};
use std::{fmt::Write, sync::Arc};

/// Represents the API version reporter, which reports API version information in HTTP response headers.
#[derive(Clone, Debug)]
pub struct Reporter {
    deprecation: Arc<Manager<DeprecationPolicy>>,
    sunset: Arc<Manager<SunsetPolicy>>,
    supported_header: HeaderName,
    deprecated_header: HeaderName,
}

fn header_name(name: &str) -> HeaderName {
    HeaderName::from_bytes(name.as_bytes()).unwrap_or_else(|_| panic!("'{name}' is not a valid HTTP header name."))
}

impl Reporter {
    /// Initializes a new [Reporter].
    ///
    /// # Arguments
    ///
    /// * `headers` - the [names of the headers](ReportHeaders) used to report API versions
    /// * `deprecation_policy_manager` - the [manager](Manager) for [deprecation policies](DeprecationPolicy)
    /// * `sunset_policy_manager` - the [manager](Manager) for [sunset policies](SunsetPolicy)
    ///
    /// # Panics
    ///
    /// Panics if a header name is not a valid HTTP header name.
    pub fn new(
        headers: &ReportHeaders,
        deprecation_policy_manager: Arc<Manager<DeprecationPolicy>>,
        sunset_policy_manager: Arc<Manager<SunsetPolicy>>,
    ) -> Self {
        Self {
            deprecation: deprecation_policy_manager,
            sunset: sunset_policy_manager,
            supported_header: header_name(headers.supported),
            deprecated_header: header_name(headers.deprecated),
        }
    }

    /// Reports the API versions defined in the specified metadata.
    ///
    /// # Arguments
    ///
    /// * `headers` - the current response [headers](HeaderMap)
    /// * `api_version` - the current [API version](ApiVersion) being requested, if any
    /// * `metadata` - the [metadata](Metadata) containing the API versions to report
    pub fn report(&self, headers: &mut HeaderMap, api_version: Option<&ApiVersion>, metadata: &Metadata) {
        if metadata.is_neutral() {
            return;
        }

        let model = metadata.map_all();

        write_versions(headers, &self.supported_header, model.supported());
        write_versions(headers, &self.deprecated_header, model.deprecated());

        let name = if metadata.name().is_empty() {
            None
        } else {
            Some(metadata.name())
        };
        let sunset;

        if let Some(policy) = self.sunset.resolve(name, api_version) {
            sunset = policy.date();
            write_sunset(headers, policy);
        } else {
            sunset = None;
        }

        if let Some(policy) = self.deprecation.resolve(name, api_version) {
            if let Some(date) = sunset {
                if policy.in_effect(date) {
                    write_deprecation(headers, policy);
                }
            } else {
                write_deprecation(headers, policy);
            }
        }
    }
}

fn write_versions(headers: &mut HeaderMap, name: &HeaderName, versions: &[ApiVersion]) {
    if versions.is_empty() || headers.contains_key(name) {
        return;
    }

    if versions.len() == 1 {
        headers.append(name.clone(), versions[0].to_string().parse().unwrap());
        return;
    }

    // base capacity: 3 dates with the format "YYYY-MM-DD" and commas
    let mut value = String::with_capacity(34);

    value.write_fmt(format_args!("{}", versions[0])).unwrap();

    for version in &versions[1..] {
        value.write_fmt(format_args!(", {}", version)).unwrap();
    }

    headers.append(name.clone(), value.parse().unwrap());
}

fn write_deprecation(headers: &mut HeaderMap, policy: &DeprecationPolicy) {
    if headers.contains_key(&DEPRECATION) {
        return;
    }

    if let Some(date) = policy.date() {
        headers.append(DEPRECATION, Deprecation::from(date).to_string().parse().unwrap());
    }

    write_links(headers, policy.links());
}

fn write_sunset(headers: &mut HeaderMap, policy: &SunsetPolicy) {
    if headers.contains_key(&SUNSET) {
        return;
    }

    if let Some(date) = policy.date() {
        headers.append(SUNSET, Sunset::from(date).to_string().parse().unwrap());
    }

    write_links(headers, policy.links());
}

fn write_links(headers: &mut HeaderMap, links: &[Link]) {
    for link in links {
        headers.append(LINK, link.to_string().parse().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Model, api_version};

    fn reporter(headers: &ReportHeaders) -> Reporter {
        Reporter::new(headers, Arc::default(), Arc::default())
    }

    fn metadata() -> Metadata {
        let model = Model::builder()
            .add_declared([api_version!(1.0), api_version!(2.0)])
            .add_supported([api_version!(1.0), api_version!(2.0)])
            .add_deprecated(api_version!(0.9))
            .build();

        Metadata::new("", model.clone(), model)
    }

    #[test]
    fn report_should_write_default_headers() {
        // arrange
        let reporter = reporter(&ReportHeaders::default());
        let mut headers = HeaderMap::new();

        // act
        reporter.report(&mut headers, None, &metadata());

        // assert
        assert_eq!(headers["api-supported-versions"], "1.0, 2.0");
        assert_eq!(headers["api-deprecated-versions"], "0.9");
    }

    #[test]
    fn report_should_write_configured_headers() {
        // arrange
        let reporter = reporter(&ReportHeaders {
            supported: "X-Supported-Versions",
            deprecated: "x-deprecated-versions",
        });
        let mut headers = HeaderMap::new();

        // act
        reporter.report(&mut headers, None, &metadata());

        // assert
        assert_eq!(headers["x-supported-versions"], "1.0, 2.0");
        assert_eq!(headers["x-deprecated-versions"], "0.9");
        assert!(!headers.contains_key("api-supported-versions"));
        assert!(!headers.contains_key("api-deprecated-versions"));
    }

    #[test]
    fn report_should_not_replace_existing_headers() {
        // arrange
        let reporter = reporter(&ReportHeaders::default());
        let mut headers = HeaderMap::new();

        headers.insert("api-supported-versions", "3.0".parse().unwrap());

        // act
        reporter.report(&mut headers, None, &metadata());

        // assert
        assert_eq!(headers.get_all("api-supported-versions").iter().count(), 1);
        assert_eq!(headers["api-supported-versions"], "3.0");
    }

    #[test]
    fn report_should_not_write_headers_for_neutral_metadata() {
        // arrange
        let reporter = reporter(&ReportHeaders::default());
        let mut headers = HeaderMap::new();

        // act
        reporter.report(&mut headers, None, &Metadata::neutral());

        // assert
        assert!(headers.is_empty());
    }

    #[test]
    #[should_panic(expected = "'api supported versions' is not a valid HTTP header name.")]
    fn new_should_panic_when_header_name_is_invalid() {
        // arrange
        let headers = ReportHeaders {
            supported: "api supported versions",
            ..Default::default()
        };

        // act
        reporter(&headers);

        // assert
    }
}
