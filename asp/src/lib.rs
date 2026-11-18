//! # Features
//!
//! | feature           | default | description                                                                    |
//! | ----------------- | ------- | ------------------------------------------------------------------------------ |
//! | `http`            | yes     | HTTP readers, selectors, policies, headers, and reporting                      |
//! | `serde`           | no      | deserializing an [ApiVersion] and the [visibility] of serialized members       |
//! | `macros`          | no      | the `#[visible_in_version]` attribute; implies `serde`                         |
//! | `problem-details` | no      | [problem details](problem) as defined by RFC 9457                              |
//!
//! [ApiVersion], [api_version!], and models never require HTTP. A crate that only defines versioned models can use
//! `default-features = false` with the `macros` feature.

/// Contains functionality related to API versioning HTTP headers.
#[cfg(feature = "http")]
pub mod headers;

mod mapping;
mod metadata;

/// Contains functionality related to API versioning models.
pub mod model;

#[cfg(feature = "http")]
mod options;

/// Contains functionality related to API versioning parameter information.
#[cfg(feature = "http")]
pub mod parameter;

/// Contains functionality related to API versioning policies.
#[cfg(feature = "http")]
pub mod policy;

/// Contains API versioning prelude.
pub mod prelude;

/// Contains functionality related to problem details as defined by [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457).
#[cfg(feature = "problem-details")]
pub mod problem;

mod provider;

/// Contains functionality related to API version readers.
#[cfg(feature = "http")]
pub mod readers;

#[cfg(feature = "http")]
mod reporter;
#[cfg(feature = "http")]
mod request;

/// Contains functionality related to API version selectors.
#[cfg(feature = "http")]
pub mod selectors;

mod variadic;
mod version;

/// Contains functionality related to the visibility of serialized members in specific API versions.
#[cfg(feature = "serde")]
pub mod visibility;

#[cfg(feature = "serde")]
#[doc(hidden)]
pub use serde as __serde;

pub use mapping::Mapping;
pub use metadata::Metadata;
pub use model::Model;
#[cfg(feature = "http")]
pub use options::{Options, ReportHeaders, ReportOptions};
pub use provider::{Provider, ProviderOptions};
#[cfg(feature = "http")]
pub use reporter::Reporter;
#[cfg(feature = "http")]
pub use request::RequestInfo;
pub use variadic::Variadic;
pub use version::{ApiVersion, CustomFormat, Format, ParseError, SetCustomFormatError, builtin, set_custom_format};

#[cfg(feature = "macros")]
pub use api_versioning_macros::visible_in_version;

#[cfg(test)]
pub(crate) mod test {
    use crate::{ApiVersion, CustomFormat, Format, ParseError, builtin, prelude::*};
    use std::borrow::Cow;
    use std::fmt::{self, Formatter};

    pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

    /// Represents a custom format in the form of `r<number>[-<status>]`, which is `rev<number>` in a type path. All
    /// other text uses the built-in formats.
    pub struct Revision;

    impl CustomFormat for Revision {
        fn parse<'a>(&self, text: &'a str) -> Result<ApiVersion<'a>, ParseError> {
            if !text.starts_with('r') {
                return builtin::parse(text);
            }

            let (revision, status) = match text.split_once('-') {
                Some((revision, status)) => (revision, Some(Cow::Borrowed(status))),
                None => (text, None),
            };

            match revision[1..].parse() {
                Ok(bits) => Ok(ApiVersion::custom(bits, status)),
                Err(_) => Err(ParseError::Custom(format!("'{text}' is not a revision"))),
            }
        }

        fn fmt(&self, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
            if version.format() != Format::Custom {
                return builtin::fmt(version, f);
            }

            let prefix = if f.alternate() { "R" } else { "r" };

            write!(f, "{prefix}{}", version.bits())?;

            if let Some(status) = version.status() {
                write!(f, "-{status}")?;
            }

            Ok(())
        }

        fn parse_path<'a>(&self, segment: &'a str) -> Option<ApiVersion<'a>> {
            match segment.strip_prefix("rev") {
                Some(number) => number.parse().ok().map(|bits| ApiVersion::custom(bits, None)),
                None => builtin::parse_path(segment),
            }
        }
    }

    /// Represents a custom format that treats all text and API versions the same and uses the default behaviors.
    pub struct Opaque;

    impl CustomFormat for Opaque {
        fn parse<'a>(&self, _text: &'a str) -> Result<ApiVersion<'a>, ParseError> {
            Ok(ApiVersion::custom(u32::MAX, None))
        }

        fn fmt(&self, _version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str("opaque")
        }
    }
}
