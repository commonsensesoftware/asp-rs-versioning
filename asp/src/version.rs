use std::borrow::Cow::{self, Owned};
use std::cmp::Ordering::{self, *};

/// Parses an unquoted numeric or date API version, optionally followed by a status.
///
/// Each token is stringified and joined without whitespace, then parsed using [`ApiVersion::parse_literal`]. Numeric
/// minor components retain their digits: `1.10` represents major version 1 and minor version 10.
///
/// # Panics
///
/// Panics if the version is invalid. In a constant expression, this causes a compilation error.
///
/// This macro does not support custom API version formats. Custom formats are expected to provide their own macros
/// using [ApiVersion::custom]. See [CustomFormat] for an example.
///
/// # Examples
///
/// ```
/// use api_versioning::{api_version, ApiVersion};
///
/// const NUMERIC: ApiVersion<'static> = api_version!(1.10);
/// const DATE: ApiVersion<'static> = api_version!(2026-10-01);
/// const PREVIEW: ApiVersion<'static> = api_version!(2.1-preview.1);
/// const RELEASE_CANDIDATE: ApiVersion<'static> = api_version!(2026-10-01-rc.2);
/// ```
///
/// ```compile_fail
/// use api_versioning::{api_version, ApiVersion};
///
/// const INVALID: ApiVersion<'static> = api_version!(256);
/// ```
///
/// # Inference
///
/// `api_version!(use path)` infers exactly one API version from the path of a module, function, or type using
/// [ApiVersion::infer_from_path]. The path is checked by the compiler, so it stays in sync when the item is renamed.
/// This form is not a constant expression and it supports [custom API version formats](CustomFormat).
///
/// Only the written path is parsed. An alias is not resolved to the path where the item is defined and
/// `api_version!(use super::orders)` does not infer an API version from the parent module's name.
///
/// ## Panics
///
/// Panics if no API version or more than one distinct API version is inferred.
///
/// ## Examples
///
/// ```
/// use api_versioning::api_version;
///
/// mod v1 {
///     pub mod orders {
///         pub fn get() {}
///     }
/// }
///
/// fn hello_v2() {}
///
/// assert_eq!(api_version!(use v1), api_version!(1));
/// assert_eq!(api_version!(use v1::orders), api_version!(1));
/// assert_eq!(api_version!(use v1::orders::get), api_version!(1));
/// assert_eq!(api_version!(use hello_v2), api_version!(2));
/// ```
///
/// ```compile_fail,E0432
/// use api_versioning::api_version;
///
/// mod v1 {}
///
/// let version = api_version!(use v2);
/// ```
#[macro_export]
macro_rules! api_version {
    (use $first:ident $(:: $rest:ident)*) => {{
        #[allow(unused_imports)]
        use $first $(:: $rest)* as _;
        $crate::ApiVersion::infer_from_path(concat!(stringify!($first) $(, "::", stringify!($rest))*))
    }};
    ($($token:tt)+) => {
        $crate::ApiVersion::parse_literal(concat!($(stringify!($token)),+))
    };
}

/// Contains the built-in API version formats.
///
/// # Remarks
///
/// The functions in this module never use the [custom format](CustomFormat). A custom format can use them to provide
/// some or all of its behavior.
pub mod builtin;

mod custom;
#[cfg(feature = "serde")]
mod deserializing;
mod formatting;
mod parsing;
mod reading;
#[cfg(feature = "serde")]
mod serializing;

use builtin::mask;
pub use custom::{CustomFormat, SetCustomFormatError, set_custom_format};
pub use parsing::ParseError;
use time::Date;

// | format  | tag (31-30) | built-in components (29 and down)        | extension (low bits) |
// | ------- | ----------- | ---------------------------------------- | -------------------- |
// | neutral | 00          | none                                     | none                 |
// | numeric | 01          | major (29-22), minor (21-14)             | 14 bits (13-0)       |
// | date    | 10          | year (29-22), month (21-18), day (17-13) | 13 bits (12-0)       |
// | custom  | 11          | none                                     | 30 bits (29-0)       |
const EPOCH: u32 = 2000;
const NUMERIC: u32 = 0b01 << 30;
const DATE: u32 = 0b10 << 30;
const CUSTOM: u32 = 0b11 << 30;
const MAJOR_SHIFT: u32 = 22;
const MINOR_SHIFT: u32 = 14;
const YEAR_SHIFT: u32 = 22;
const MONTH_SHIFT: u32 = 18;
const DAY_SHIFT: u32 = 13;

const fn numeric_bits(major: u8, minor: u8) -> u32 {
    NUMERIC | (major as u32) << MAJOR_SHIFT | (minor as u32) << MINOR_SHIFT
}

const fn date_bits(date: Date) -> u32 {
    let year = (date.year() as u32 - EPOCH) << YEAR_SHIFT;
    let month = (date.month() as u32) << MONTH_SHIFT;
    let day = (date.day() as u32) << DAY_SHIFT;

    DATE | year | month | day
}

/// Represents the possible API version formats.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::manual_non_exhaustive,
    reason = "false positive; custom formats are never understood"
)]
pub enum Format {
    /// Indicates the format is unknown.
    ///
    /// # Remarks
    ///
    /// A neutral API version does not have a known or defined format.
    Unknown,

    /// Indicates the format is numeric in the form of `<major>[.<minor>]`.
    Numeric,

    /// Indicates the format is a date in the form of `yyyy-mm-dd`.
    Date,

    /// Indicates the format is custom.
    ///
    /// # Remarks
    ///
    /// A custom format is only understood by the implementor.
    Custom,
}

/// Represents an API version.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ApiVersion<'a>(u32, Option<Cow<'a, str>>);

impl<'a> ApiVersion<'a> {
    /// Initializes a new [ApiVersion].
    ///
    /// # Arguments
    ///
    /// * `bits` - the encode API version bits
    /// * `status` - the associated status, if any
    ///
    /// # Remarks
    ///
    /// This function is intentionally undocumented and is only intended for use by API version parsers. A custom
    /// API version format uses this function to construct an API version and uses [bits](Self::bits) with its own
    /// traits to extract the relevant components.
    ///
    /// The two high-order bits are reserved. A custom format is expected to use the bits `11` (e.g. 3). The remaining
    /// 30 bits may have any meaning defined by the custom format. Failure to adhere to the expected usage may lead to
    /// undefined behavior. Prefer [numeric](Self::numeric), [date](Self::date), or [custom](Self::custom), which set
    /// the reserved bits.
    ///
    /// Custom API versions are compared by their bits and then by status. A custom format must encode its bits such
    /// that their numeric order matches the intended order of the API versions.
    #[doc(hidden)]
    pub const fn new(bits: u32, status: Option<Cow<'a, str>>) -> Self {
        Self(bits, status)
    }

    /// Initializes a new [numeric](Format::Numeric) [ApiVersion].
    ///
    /// # Arguments
    ///
    /// * `major` - the major version
    /// * `minor` - the minor version
    /// * `extension` - the extension defined by a [custom format](CustomFormat), if any; only the bits in
    ///   [mask::NUMERIC] are retained
    /// * `status` - the associated status, if any
    ///
    /// # Remarks
    ///
    /// The extension is stored in the low-order bits so that API versions are ordered by major version, minor version,
    /// extension, and then status. A custom format can use the extension for an additional component, such as the patch
    /// version in `1.2.3`, while the [built-in numeric components](crate::prelude::NumericApiVersionExt) continue to
    /// work. The built-in formats always use an extension of `0`.
    ///
    /// This function is `const` so that a custom format can provide its own macro in the same manner as
    /// [api_version!].
    pub const fn numeric(major: u8, minor: u8, extension: u32, status: Option<Cow<'a, str>>) -> Self {
        Self(numeric_bits(major, minor) | (extension & mask::NUMERIC), status)
    }

    /// Initializes a new [date](Format::Date) [ApiVersion].
    ///
    /// # Arguments
    ///
    /// * `date` - the version date
    /// * `extension` - the extension defined by a [custom format](CustomFormat), if any; only the bits in
    ///   [mask::DATE] are retained
    /// * `status` - the associated status, if any
    ///
    /// # Remarks
    ///
    /// The extension is stored in the low-order bits so that API versions are ordered by date, extension, and then
    /// status. The [built-in date component](crate::prelude::DateApiVersionExt) continues to work. The built-in formats
    /// always use an extension of `0`.
    ///
    /// This function is `const` so that a custom format can provide its own macro in the same manner as
    /// [api_version!].
    ///
    /// # Panics
    ///
    /// Panics if the year of the date is not between 2000 and 2255. In a constant expression, this causes a compilation
    /// error.
    pub const fn date(date: Date, extension: u32, status: Option<Cow<'a, str>>) -> Self {
        let year = date.year();

        if year < EPOCH as i32 || year > (EPOCH + 255) as i32 {
            panic!("the specified API version date must be between the years 2000 and 2255");
        }

        Self(date_bits(date) | (extension & mask::DATE), status)
    }

    /// Initializes a new [custom](Format::Custom) [ApiVersion].
    ///
    /// # Arguments
    ///
    /// * `bits` - the bits defined by the custom format; only the bits in [mask::CUSTOM] are retained
    /// * `status` - the associated status, if any
    ///
    /// # Remarks
    ///
    /// This function is intended for use by [custom formats](CustomFormat) that are neither numeric nor date-based.
    /// It is `const` so that a custom format can provide its own macro in the same manner as [api_version!].
    ///
    /// Custom API versions are compared by their bits and then by status. A custom format must encode its bits such
    /// that their numeric order matches the intended order of the API versions.
    pub const fn custom(bits: u32, status: Option<Cow<'a, str>>) -> Self {
        Self(CUSTOM | (bits & mask::CUSTOM), status)
    }
}

impl ApiVersion<'_> {
    /// Represents a neutral API version.
    pub const NEUTRAL: ApiVersion<'static> = ApiVersion(0, None);

    /// Represents the default API version of `1.0`.
    pub const DEFAULT: ApiVersion<'static> = ApiVersion(numeric_bits(1, 0), None);

    /// Compares two [API versions](ApiVersion) and returns their [ordering](Ordering).
    ///
    /// # Remarks
    ///
    /// Unlike the standard partial comparison between [API versions](ApiVersion), this comparison always produces an
    /// [ordering](Ordering). When values are not comparable then they are ordered in the following sequence:
    ///
    /// - [Format::Unknown] is always less than other [formats](Format)
    /// - [Format::Custom] is greater than [Format::Unknown], but less than other [formats](Format)
    /// - [Format::Numeric] is greater than [Format::Unknown] and [Format::Custom], but less than [Format::Date]
    /// - [Format::Date] is always greater than other [formats](Format)
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        use self::Format::*;

        const fn rank(format: Format) -> u8 {
            match format {
                Unknown => 0,
                Custom => 1,
                Numeric => 2,
                Date => 3,
            }
        }

        self.partial_cmp(other)
            .unwrap_or_else(|| rank(self.format()).cmp(&rank(other.format())))
    }

    /// Returns an owned version of the API version.
    pub fn to_owned(&self) -> ApiVersion<'static> {
        ApiVersion(self.0, self.1.as_ref().map(|s| Owned(s.to_string())))
    }
}

impl PartialOrd for ApiVersion<'_> {
    /// Partially compares two [API versions](ApiVersion).
    ///
    /// # Remarks
    ///
    /// [API versions](ApiVersion) are only comparable if they have the same [format](Self::format).
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.format() == other.format() {
            let mut ordering = self.0.cmp(&other.0);

            if ordering == Equal {
                ordering = match (&self.1, &other.1) {
                    (None, None) => Equal,
                    (None, Some(_)) => Greater,
                    (Some(_), None) => Less,
                    (Some(s1), Some(s2)) => s1.cmp(s2),
                };
            }

            Some(ordering)
        } else {
            None
        }
    }
}

impl From<&ApiVersion<'_>> for ApiVersion<'static> {
    fn from(value: &ApiVersion<'_>) -> Self {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests;
