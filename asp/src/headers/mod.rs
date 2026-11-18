mod deprecation;
mod error;
mod link;
mod sunset;

pub use deprecation::Deprecation;
pub use error::AttributeNameError;
pub use link::{Link, LinkBuilder, LinkParseError};
pub use sunset::Sunset;

use http::header::HeaderName;

/// Advertises the supported API versions.
///
/// The `api-supported-versions` HTTP header may be single-valued, multi-valued, or appear multiple times.
pub const API_SUPPORTED_VERSIONS: HeaderName = HeaderName::from_static("api-supported-versions");

/// Advertises the deprecated API versions.
///
/// The `api-deprecated-versions` HTTP header may be single-valued, multi-valued, or appear multiple times.
pub const API_DEPRECATED_VERSIONS: HeaderName = HeaderName::from_static("api-deprecated-versions");

pub const DEPRECATION: HeaderName = HeaderName::from_static("deprecation");

/// Advertises a resource-related link defined by [RFC 8288] in a HTTP response.
///
/// [RFC 8288]: https://datatracker.ietf.org/doc/html/rfc8288
pub const LINK: HeaderName = HeaderName::from_static("link");

pub const SUNSET: HeaderName = HeaderName::from_static("sunset");

const fn valid_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_'
}
