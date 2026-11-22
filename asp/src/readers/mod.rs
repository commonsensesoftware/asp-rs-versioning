mod combine;
mod header;
mod media;
mod names;
mod query;
mod url;

pub use combine::CombinedReader;
pub use header::HeaderReader;
pub use media::MediaTypeReader;
pub use names::IntoNames;
pub use url::UrlReader;

#[doc(hidden)]
pub use media::template as __template;
pub use query::QueryStringReader;

use crate::RequestInfo;
use crate::parameter;
use smallvec::SmallVec;
use std::fmt::Debug;

/// Defines the behavior of an API version reader.
pub trait Reader: Debug + Send + Sync + parameter::Descriptor {
    /// Reads the API versions from the given request.
    fn read<'a>(&self, request: &'a dyn RequestInfo) -> SmallVec<[&'a str; 1]>;

    /// Clones the current reader.
    fn clone(&self) -> Box<dyn Reader>;
}

impl Clone for Box<dyn Reader> {
    fn clone(&self) -> Self {
        (**self).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RequestInfo, prelude::*};
    use http::{HeaderMap, HeaderName, Uri};
    use rstest::rstest;

    struct Request(HeaderMap, Uri);

    impl Request {
        fn new(uri: &'static str, headers: &[(&'static str, &'static str)]) -> Self {
            let mut map = HeaderMap::new();

            for (name, value) in headers {
                map.append(HeaderName::from_static(name), value.parse().unwrap());
            }

            Self(map, Uri::from_static(uri))
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
    #[case("/test", &[], &[])]
    #[case("/test?api-version=1.0", &[], &["1.0"])]
    #[case("/test", &[("x-api-version", "1.0")], &["1.0"])]
    #[case("/test?api-version=1.0", &[("x-api-version", "1.0")], &["1.0"])]
    #[case("/test?api-version=1.0", &[("x-api-version", "2.0")], &["1.0", "2.0"])]
    #[case("/test?api-version=2.0", &[("x-api-version", "1.0")], &["1.0", "2.0"])]
    #[case("/test?api-version=2.0&api-version=1.0", &[("x-api-version", "2.0")], &["1.0", "2.0"])]
    #[case("/test?api-version=3.0", &[("x-api-version", "1.0"), ("x-ms-version", "2.0")], &["1.0", "2.0", "3.0"])]
    #[case("/test?version=1.0", &[("api-version", "2.0")], &[])]
    fn combine_should_return_distinct_sorted_versions(
        #[case] uri: &'static str,
        #[case] headers: &[(&'static str, &'static str)],
        #[case] expected: &[&str],
    ) {
        // arrange
        let reader = QueryStringReader::default().combine(HeaderReader::new(["x-api-version", "x-ms-version"]));
        let request = Request::new(uri, headers);

        // act
        let mut actual = reader.read(&request);

        actual.sort();

        // assert
        assert_eq!(actual.as_slice(), expected);
    }
}
