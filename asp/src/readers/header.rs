use super::{IntoNames, Reader};
use crate::RequestInfo;
use crate::parameter::{Context, Descriptor, Location::Header};
use smallvec::SmallVec;
use std::borrow::Cow;

/// Represents an API version reader that reads the value from a HTTP header.
#[derive(Clone, Debug)]
pub struct HeaderReader(SmallVec<[Cow<'static, str>; 1]>);

impl HeaderReader {
    /// Initializes a new [HeaderReader].
    ///
    /// # Arguments
    ///
    /// * `names` - one or more HTTP header names to read the API version from
    pub fn new(names: impl IntoNames) -> Self {
        Self(names.into_names())
    }
}

impl Reader for HeaderReader {
    fn read<'a>(&self, request: &'a dyn RequestInfo) -> SmallVec<[&'a str; 1]> {
        self.0
            .iter()
            .filter_map(|name| {
                request
                    .headers()
                    .get(name.as_ref())
                    .and_then(|value| value.to_str().ok())
            })
            .collect()
    }

    fn clone(&self) -> Box<dyn Reader> {
        Box::new(Clone::clone(self))
    }
}

impl Descriptor for HeaderReader {
    fn describe<'a>(&'a self, context: &mut dyn Context<'a>) {
        for name in &self.0 {
            context.register(name, Header);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HeaderReader, Reader};
    use crate::RequestInfo;
    use http::{HeaderMap, HeaderName, Uri};
    use rstest::rstest;

    struct Request(HeaderMap, Uri);

    impl Request {
        fn new(headers: &[(&'static str, &'static str)]) -> Self {
            let mut map = HeaderMap::new();

            for (name, value) in headers {
                map.append(HeaderName::from_static(name), value.parse().unwrap());
            }

            Self(map, Uri::default())
        }
    }

    impl RequestInfo for Request {
        fn uri(&self) -> &Uri {
            &self.1
        }

        fn headers(&self) -> &HeaderMap {
            &self.0
        }
    }

    #[rstest]
    #[case(HeaderReader::new("api-version"), &[("api-version", "1.0")], &["1.0"])]
    #[case(HeaderReader::new("api-version"), &[("x-api-version", "1.0")], &[])]
    #[case(HeaderReader::new("api-version"), &[], &[])]
    #[case(HeaderReader::new("API-Version"), &[("api-version", "1.0")], &["1.0"])]
    #[case(HeaderReader::new("X-API-VERSION"), &[("x-api-version", "2.0")], &["2.0"])]
    #[case(HeaderReader::new(["api-version", "x-ms-version"]), &[("x-ms-version", "2.0")], &["2.0"])]
    #[case(
        HeaderReader::new(["api-version", "x-ms-version"]),
        &[("api-version", "1.0"), ("x-ms-version", "2.0")],
        &["1.0", "2.0"]
    )]
    #[case(HeaderReader::new(["Api-Version", "X-MS-Version"]), &[("x-ms-version", "2.0")], &["2.0"])]
    fn read_should_return_matched_versions(
        #[case] reader: HeaderReader,
        #[case] headers: &[(&'static str, &'static str)],
        #[case] expected: &[&str],
    ) {
        // arrange
        let request = Request::new(headers);

        // act
        let versions = reader.read(&request);

        // assert
        assert_eq!(versions.as_slice(), expected);
    }
}
