/// Represents the supported API version parameter description locations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    /// Indicates the API version is expressed as a HTTP query string parameter.
    Query,

    /// Indicates the API version is expressed as a HTTP header.
    Header,

    /// Indicates the API version is expressed in a URL path segment.
    Path,

    /// Indicates the API version is expressed as a media type.
    MediaType,
}
