mod header;
mod media;
mod query;
mod url;

pub use header::HeaderReader;
pub use media::MediaTypeReader;
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

/// Represents a combined API version reader that aggregates multiple individual readers.
#[derive(Debug, Default)]
pub struct CombinedReader(Vec<Box<dyn Reader>>);

impl CombinedReader {
    fn new(reader: impl Reader + 'static) -> Self {
        let mut readers = Vec::with_capacity(2);
        readers.push(Box::new(reader) as Box<dyn Reader>);
        Self(readers)
    }

    /// Adds an additional API version reader to the combined reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - the API version [reader](Reader) to add to the combined reader
    pub fn with(mut self, reader: impl Reader + 'static) -> Self {
        self.0.push(Box::new(reader));
        self
    }
}

impl Reader for CombinedReader {
    fn read<'a>(&self, request: &'a dyn RequestInfo) -> SmallVec<[&'a str; 1]> {
        let mut versions = SmallVec::new();

        for reader in &self.0 {
            for version in reader.read(request) {
                if !versions.contains(&version) {
                    versions.push(version);
                }
            }
        }

        versions
    }

    fn clone(&self) -> Box<dyn Reader> {
        Box::new(Clone::clone(self))
    }
}

impl parameter::Descriptor for CombinedReader {
    fn describe<'a>(&'a self, context: &mut dyn parameter::Context<'a>) {
        for reader in &self.0 {
            reader.describe(context);
        }
    }
}

impl Clone for CombinedReader {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|r| Reader::clone(&**r)).collect())
    }
}

impl From<CombinedReader> for Box<dyn Reader> {
    fn from(reader: CombinedReader) -> Self {
        Box::new(reader)
    }
}

impl From<CombinedReader> for Box<dyn parameter::Descriptor> {
    fn from(reader: CombinedReader) -> Self {
        Box::new(reader)
    }
}

/// Combines multiple API version readers into a single reader that aggregates their results.
///
/// # Arguments
///
/// * `reader` - the API version [reader](Reader) to combine
pub fn combine(reader: impl Reader + 'static) -> CombinedReader {
    CombinedReader::new(reader)
}

#[cfg(test)]
mod tests {
    use super::{HeaderReader, QueryStringReader, Reader, combine};
    use crate::RequestInfo;
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
        let reader = combine(QueryStringReader::default()).with(HeaderReader::many(["x-api-version", "x-ms-version"]));
        let request = Request::new(uri, headers);

        // act
        let mut actual = reader.read(&request);

        actual.sort();

        // assert
        assert_eq!(actual.as_slice(), expected);
    }
}
