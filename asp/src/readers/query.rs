use super::Reader;
use crate::RequestInfo;
use crate::parameter::{Context, Descriptor, Location::Query};
use smallvec::{SmallVec, smallvec};
use std::borrow::Cow;

/// Represents an API version reader that reads the value from the query string in a URL.
#[derive(Clone, Debug)]
pub struct QueryStringReader(SmallVec<[Cow<'static, str>; 1]>);

impl Default for QueryStringReader {
    fn default() -> Self {
        Self(smallvec!["api-version".into()])
    }
}

impl QueryStringReader {
    /// Initializes a new [QueryStringReader].
    ///
    /// # Arguments
    ///
    /// * `name` - the query string parameter to read the API version from
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(smallvec![name.into()])
    }

    /// Initializes a new [QueryStringReader].
    ///
    /// # Arguments
    ///
    /// * `names` - the sequence of query string parameters to read the API version from
    pub fn many(names: impl IntoIterator<Item = impl Into<Cow<'static, str>>>) -> Self {
        Self(
            names
                .into_iter()
                .map(Into::into)
                .collect::<SmallVec<[Cow<'static, str>; 1]>>(),
        )
    }
}

impl Reader for QueryStringReader {
    fn read<'a>(&self, request: &'a dyn RequestInfo) -> SmallVec<[&'a str; 1]> {
        request.uri().query().map_or_else(SmallVec::new, |query| {
            query
                .split('&')
                .filter_map(|pair| match pair.split_once('=') {
                    Some((key, value)) if self.0.iter().any(|k| k.eq_ignore_ascii_case(key)) => Some(value),
                    _ => None,
                })
                .collect()
        })
    }

    fn clone(&self) -> Box<dyn Reader> {
        Box::new(Clone::clone(self))
    }
}

impl Descriptor for QueryStringReader {
    fn describe<'a>(&'a self, context: &mut dyn Context<'a>) {
        for name in &self.0 {
            context.register(name, Query);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{QueryStringReader, Reader};
    use crate::RequestInfo;
    use http::{HeaderMap, Uri};
    use rstest::rstest;

    struct Request(HeaderMap, Uri);

    impl Request {
        fn new(uri: &'static str) -> Self {
            Self(HeaderMap::new(), Uri::from_static(uri))
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
    #[case(QueryStringReader::default(), "/test?api-version=1.0", &["1.0"])]
    #[case(QueryStringReader::default(), "/test?foo=bar&api-version=2.0", &["2.0"])]
    #[case(QueryStringReader::default(), "/test?api-version=1.0&api-version=2.0", &["1.0", "2.0"])]
    #[case(QueryStringReader::default(), "/test?version=1.0", &[])]
    #[case(QueryStringReader::default(), "/test?api-version", &[])]
    #[case(QueryStringReader::default(), "/test", &[])]
    #[case(QueryStringReader::default(), "/test?API-Version=1.0", &["1.0"])]
    #[case(QueryStringReader::new("ver"), "/test?ver=1.0", &["1.0"])]
    #[case(QueryStringReader::new("ver"), "/test?api-version=1.0", &[])]
    #[case(QueryStringReader::new("Ver"), "/test?vER=1.0", &["1.0"])]
    #[case(QueryStringReader::many(["api-version", "v"]), "/test?v=1.0", &["1.0"])]
    #[case(QueryStringReader::many(["api-version", "v"]), "/test?api-version=1.0&v=2.0", &["1.0", "2.0"])]
    #[case(QueryStringReader::many(["Api-Version", "V"]), "/test?API-VERSION=1.0&v=2.0", &["1.0", "2.0"])]
    fn read_should_return_matched_versions(
        #[case] reader: QueryStringReader,
        #[case] uri: &'static str,
        #[case] expected: &[&str],
    ) {
        // arrange
        let request = Request::new(uri);

        // act
        let versions = reader.read(&request);

        // assert
        assert_eq!(versions.as_slice(), expected);
    }
}
