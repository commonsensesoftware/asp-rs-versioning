use super::{IntoNames, Reader};
use crate::RequestInfo;
use crate::parameter::{Context, Descriptor, Location::Path};
use smallvec::{SmallVec, smallvec};
use std::borrow::Cow;

/// Represents an API version reader that reads the value from a route parameter in the request URL.
///
/// # Remarks
///
/// A URL is matched by the routing system of a web framework rather than by the reader. The route template defines where
/// the API version appears, such as `/v{api_version}/orders`, and the reader only reads the value of the captured
/// parameter. Parameter names are case-sensitive.
///
/// # Examples
///
/// ```
/// use api_versioning::readers::UrlReader;
///
/// // matches the parameter in the route template '/v{api_version}/orders'
/// let reader = UrlReader::default();
///
/// // matches the parameter in the route template '/v{ver}/orders'
/// let reader = UrlReader::new("ver");
/// ```
#[derive(Clone, Debug)]
pub struct UrlReader(SmallVec<[Cow<'static, str>; 1]>);

impl Default for UrlReader {
    fn default() -> Self {
        Self(smallvec!["api_version".into()])
    }
}

impl UrlReader {
    /// Initializes a new [UrlReader].
    ///
    /// # Arguments
    ///
    /// * `names` - one or more route parameters to read the API version from
    pub fn new(names: impl IntoNames) -> Self {
        Self(names.into_names())
    }
}

impl Reader for UrlReader {
    fn read<'a>(&self, request: &'a dyn RequestInfo) -> SmallVec<[&'a str; 1]> {
        self.0.iter().filter_map(|name| request.route_parameter(name)).collect()
    }

    fn clone(&self) -> Box<dyn Reader> {
        Box::new(Clone::clone(self))
    }
}

impl Descriptor for UrlReader {
    fn describe<'a>(&'a self, context: &mut dyn Context<'a>) {
        for name in &self.0 {
            context.register(name, Path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Reader, UrlReader};
    use crate::RequestInfo;
    use http::{HeaderMap, Uri};
    use rstest::rstest;

    struct Request(HeaderMap, Uri, &'static [(&'static str, &'static str)]);

    impl Request {
        fn new(parameters: &'static [(&'static str, &'static str)]) -> Self {
            Self(HeaderMap::new(), Uri::default(), parameters)
        }
    }

    impl RequestInfo for Request {
        fn uri(&self) -> &Uri {
            &self.1
        }

        fn headers(&self) -> &HeaderMap {
            &self.0
        }

        fn route_parameter(&self, name: &str) -> Option<&str> {
            self.2.iter().find(|(key, _)| *key == name).map(|(_, value)| *value)
        }
    }

    #[rstest]
    #[case(UrlReader::default(), &[("api_version", "1.0")], &["1.0"])]
    #[case(UrlReader::default(), &[("id", "42"), ("api_version", "1.0")], &["1.0"])]
    #[case(UrlReader::default(), &[("version", "1.0")], &[])]
    #[case(UrlReader::default(), &[("API_VERSION", "1.0")], &[])]
    #[case(UrlReader::default(), &[], &[])]
    #[case(UrlReader::new("ver"), &[("ver", "1.0")], &["1.0"])]
    #[case(UrlReader::new("ver"), &[("api_version", "1.0")], &[])]
    #[case(UrlReader::new(["ver", "v"]), &[("v", "1.0")], &["1.0"])]
    #[case(UrlReader::new(["ver", "v"]), &[("v", "1.0"), ("ver", "2.0")], &["2.0", "1.0"])]
    fn read_should_return_route_parameter_values(
        #[case] reader: UrlReader,
        #[case] parameters: &'static [(&'static str, &'static str)],
        #[case] expected: &[&str],
    ) {
        // arrange
        let request = Request::new(parameters);

        // act
        let versions = reader.read(&request);

        // assert
        assert_eq!(versions.as_slice(), expected);
    }

    #[test]
    fn read_should_return_nothing_when_request_does_not_provide_route_parameters() {
        // arrange
        let reader = UrlReader::default();
        let request = http::Request::new(());

        // act
        let versions = reader.read(&request);

        // assert
        assert!(versions.is_empty());
    }
}
