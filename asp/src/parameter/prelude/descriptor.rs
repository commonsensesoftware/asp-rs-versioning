use super::super::{
    Descriptor,
    Location::{self, *},
};
use smallvec::SmallVec;

struct Context<'a> {
    matched: bool,
    elsewhere: bool,
    location: Location,
    names: SmallVec<[&'a str; 2]>,
}

impl Context<'_> {
    fn new(location: Location) -> Self {
        Self {
            matched: false,
            elsewhere: false,
            location,
            names: SmallVec::new(),
        }
    }
}

impl<'a> super::super::Context<'a> for Context<'a> {
    fn register(&mut self, name: &'a str, location: Location) {
        let matched = self.location == location;

        self.matched |= matched;
        self.elsewhere |= !matched;

        if matched && !self.names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            self.names.push(name);
        }
    }
}

fn versions_by_location(descriptor: &dyn Descriptor, location: Location, allow_multiple_locations: bool) -> bool {
    let mut context = Context::new(location);
    descriptor.describe(&mut context);
    context.matched && (allow_multiple_locations || !context.elsewhere)
}

/// Provides extensions for a parameter [Descriptor].
pub trait DescriptorExt {
    /// Determines whether the specified parameter source versions by query string.
    ///
    /// # Arguments
    ///
    /// * `allow_multiple_locations` - `true` if multiple API version locations are allowed. `false` if the API version
    ///   can only appear in a query string parameter
    fn versions_by_query_string(&self, allow_multiple_locations: bool) -> bool;

    /// Determines whether the specified parameter source versions by HTTP header.
    ///
    /// # Arguments
    ///
    /// * `allow_multiple_locations` - `true` if multiple API version locations are allowed. `false` if the API version
    ///   can only appear in a HTTP header
    fn versions_by_header(&self, allow_multiple_locations: bool) -> bool;

    /// Determines whether the specified parameter source versions by media type.
    ///
    /// # Arguments
    ///
    /// * `allow_multiple_locations` - `true` if multiple API version locations are allowed. `false` if the API version
    ///   can only appear in a media type
    fn versions_by_media_type(&self, allow_multiple_locations: bool) -> bool;

    /// Determines whether the specified parameter source versions by URL path.
    ///
    /// # Arguments
    ///
    /// * `allow_multiple_locations` - `true` if multiple API version locations are allowed. `false` if the API version
    ///   can only appear in a URL path
    fn versions_by_url(&self, allow_multiple_locations: bool) -> bool;

    /// Gets the name of the first registered parameter in the specified location, if any.
    ///
    /// # Arguments
    ///
    /// * `location` - the [location](Location) to get the parameter name for
    ///
    /// # Remarks
    ///
    /// Parameter names are case-insensitive. When names that differ only by case are registered, the first one wins.
    fn parameter(&self, location: Location) -> Option<&str>;

    /// Gets the names of the associated parameters.
    ///
    /// # Arguments
    ///
    /// * `location` - the [location](Location) to get the parameter names for
    fn parameters(&self, location: Location) -> SmallVec<[&str; 2]>;
}

impl DescriptorExt for dyn Descriptor + '_ {
    fn versions_by_query_string(&self, allow_multiple_locations: bool) -> bool {
        versions_by_location(self, Query, allow_multiple_locations)
    }

    fn versions_by_header(&self, allow_multiple_locations: bool) -> bool {
        versions_by_location(self, Header, allow_multiple_locations)
    }

    fn versions_by_media_type(&self, allow_multiple_locations: bool) -> bool {
        versions_by_location(self, MediaType, allow_multiple_locations)
    }

    fn versions_by_url(&self, allow_multiple_locations: bool) -> bool {
        versions_by_location(self, Path, allow_multiple_locations)
    }

    fn parameter(&self, location: Location) -> Option<&str> {
        let mut context = Context::new(location);
        self.describe(&mut context);
        context.names.first().copied()
    }

    fn parameters(&self, location: Location) -> SmallVec<[&str; 2]> {
        let mut context = Context::new(location);
        self.describe(&mut context);
        context.names
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parameter::{
            Descriptor,
            Location::{self, *},
            prelude::*,
        },
        pattern,
        prelude::*,
        readers::*,
    };
    use rstest::rstest;

    fn query_and_header() -> CombinedReader {
        QueryStringReader::default().combine(HeaderReader::new("x-api-version"))
    }

    #[rstest]
    #[case(&QueryStringReader::default(), false, true)]
    #[case(&QueryStringReader::default(), true, true)]
    #[case(&QueryStringReader::new(["api-version", "v"]), false, true)]
    #[case(&HeaderReader::new("x-api-version"), true, false)]
    #[case(&MediaTypeReader::default(), true, false)]
    #[case(&query_and_header(), false, false)]
    #[case(&query_and_header(), true, true)]
    fn versions_by_query_string_should_return_expected_result(
        #[case] descriptor: &dyn Descriptor,
        #[case] allow_multiple_locations: bool,
        #[case] expected: bool,
    ) {
        // arrange

        // act
        let result = descriptor.versions_by_query_string(allow_multiple_locations);

        // assert
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(&HeaderReader::new("x-api-version"), false, true)]
    #[case(&HeaderReader::new("x-api-version"), true, true)]
    #[case(&HeaderReader::new(["x-api-version", "x-ms-version"]), false, true)]
    #[case(&QueryStringReader::default(), true, false)]
    #[case(&MediaTypeReader::default(), true, false)]
    #[case(&query_and_header(), false, false)]
    #[case(&query_and_header(), true, true)]
    fn versions_by_header_should_return_expected_result(
        #[case] descriptor: &dyn Descriptor,
        #[case] allow_multiple_locations: bool,
        #[case] expected: bool,
    ) {
        // arrange

        // act
        let result = descriptor.versions_by_header(allow_multiple_locations);

        // assert
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(&MediaTypeReader::default(), false, true)]
    #[case(&MediaTypeReader::default(), true, true)]
    #[case(&MediaTypeReader::template(pattern!(application/vnd.{v})), false, true)]
    #[case(&QueryStringReader::default(), true, false)]
    #[case(&HeaderReader::new("x-api-version"), true, false)]
    #[case(&QueryStringReader::default().combine(HeaderReader::new("x-api-version")), false, false)]
    #[case(&MediaTypeReader::default().combine(HeaderReader::new("x-api-version")), true, true)]
    fn versions_by_media_type_should_return_expected_result(
        #[case] descriptor: &dyn Descriptor,
        #[case] allow_multiple_locations: bool,
        #[case] expected: bool,
    ) {
        // arrange

        // act
        let result = descriptor.versions_by_media_type(allow_multiple_locations);

        // assert
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(&UrlReader::default(), false, true)]
    #[case(&UrlReader::default(), true, true)]
    #[case(&UrlReader::new(["api_version", "ver"]), false, true)]
    #[case(&QueryStringReader::default(), true, false)]
    #[case(&HeaderReader::new("x-api-version"), true, false)]
    #[case(&UrlReader::default().combine(QueryStringReader::default()), false, false)]
    #[case(&UrlReader::default().combine(QueryStringReader::default()), true, true)]
    fn versions_by_url_should_return_expected_result(
        #[case] descriptor: &dyn Descriptor,
        #[case] allow_multiple_locations: bool,
        #[case] expected: bool,
    ) {
        // arrange

        // act
        let result = descriptor.versions_by_url(allow_multiple_locations);

        // assert
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(&UrlReader::default(), Path, Some("api_version"))]
    #[case(&UrlReader::new("ver"), Path, Some("ver"))]
    #[case(&UrlReader::default(), Query, None)]
    #[case(&QueryStringReader::default(), Query, Some("api-version"))]
    #[case(&QueryStringReader::new("ver"), Query, Some("ver"))]
    #[case(&QueryStringReader::default(), Header, None)]
    #[case(&HeaderReader::new("x-api-version"), Header, Some("x-api-version"))]
    #[case(&HeaderReader::new("x-api-version"), Query, None)]
    #[case(&MediaTypeReader::default(), MediaType, Some("v"))]
    #[case(&MediaTypeReader::new("ver"), MediaType, Some("ver"))]
    #[case(&QueryStringReader::default(), Path, None)]
    #[case(&query_and_header(), Query, Some("api-version"))]
    #[case(&query_and_header(), Header, Some("x-api-version"))]
    #[case(&query_and_header(), MediaType, None)]
    #[case(&QueryStringReader::new(["api-version", "v"]), Query, Some("api-version"))]
    #[case(&HeaderReader::new(["x-api-version", "x-ms-version"]), Header, Some("x-api-version"))]
    #[case(&HeaderReader::new(["X-API-Version", "x-api-version"]), Header, Some("X-API-Version"))]
    fn parameter_should_return_name_for_location(
        #[case] descriptor: &dyn Descriptor,
        #[case] location: Location,
        #[case] expected: Option<&str>,
    ) {
        // arrange

        // act
        let name = descriptor.parameter(location);

        // assert
        assert_eq!(name, expected);
    }

    #[rstest]
    #[case(&QueryStringReader::default(), Query, &["api-version"])]
    #[case(&QueryStringReader::new(["api-version", "v"]), Query, &["api-version", "v"])]
    #[case(&QueryStringReader::new(["api-version", "API-Version"]), Query, &["api-version"])]
    #[case(&QueryStringReader::default(), Header, &[])]
    #[case(&HeaderReader::new(["x-api-version", "x-ms-version"]), Header, &["x-api-version", "x-ms-version"])]
    #[case(&HeaderReader::new(["X-API-Version", "x-api-version"]), Header, &["X-API-Version"])]
    #[case(&MediaTypeReader::default(), MediaType, &["v"])]
    #[case(&MediaTypeReader::new(["v", "ver"]), MediaType, &["v", "ver"])]
    #[case(&UrlReader::new(["api_version", "ver"]), Path, &["api_version", "ver"])]
    #[case(&query_and_header(), Query, &["api-version"])]
    #[case(&query_and_header(), Header, &["x-api-version"])]
    #[case(
        &QueryStringReader::default()
            .combine(QueryStringReader::new("API-VERSION"))
            .combine(QueryStringReader::new("v")),
        Query,
        &["api-version", "v"]
    )]
    fn parameters_should_return_distinct_names_for_location(
        #[case] descriptor: &dyn Descriptor,
        #[case] location: Location,
        #[case] expected: &[&str],
    ) {
        // arrange

        // act
        let names = descriptor.parameters(location);

        // assert
        assert_eq!(names.as_slice(), expected);
    }
}
