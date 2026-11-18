use super::{ApiVersion, ParseError, formatting, parsing};
use std::fmt::{self, Formatter};

/// Contains the masks for the bits of an [API version](ApiVersion) that are available to a
/// [custom format](super::CustomFormat).
///
/// # Remarks
///
/// The masks are applied to the [bits](ApiVersion::bits) of an API version. For example, the extension of a numeric
/// API version is `version.bits() & mask::NUMERIC`.
pub mod mask {
    /// Represents the mask for the extension bits of a [numeric](crate::Format::Numeric) API version.
    pub const NUMERIC: u32 = 0b00000000_00000000_00111111_11111111;

    /// Represents the mask for the extension bits of a [date](crate::Format::Date) API version.
    pub const DATE: u32 = 0b00000000_00000000_00011111_11111111;

    /// Represents the mask for the bits of a [custom](crate::Format::Custom) API version.
    pub const CUSTOM: u32 = 0b00111111_11111111_11111111_11111111;
}

/// Parses a numeric (`major[.minor]`) or date (`yyyy-mm-dd`) API version, optionally followed by a status.
///
/// # Arguments
///
/// * `text` - the text to parse
///
/// # Remarks
///
/// This function never uses the [custom format](super::CustomFormat). Otherwise, it has the same rules as
/// [ApiVersion::parse].
pub fn parse(text: &str) -> Result<ApiVersion<'_>, ParseError> {
    parsing::parse(text)
}

/// Parses a numeric or date API version from a type path segment, such as a module name.
///
/// # Arguments
///
/// * `segment` - the type path segment to parse
///
/// # Remarks
///
/// This function never uses the [custom format](super::CustomFormat). Otherwise, it has the same rules as
/// [ApiVersion::from_type].
pub fn parse_path(segment: &str) -> Option<ApiVersion<'_>> {
    parsing::parse_path(segment)
}

/// Formats an API version using the built-in formats.
///
/// # Arguments
///
/// * `version` - the [API version](ApiVersion) to format
/// * `f` - the [formatter](Formatter) to write to
///
/// # Remarks
///
/// This function never uses the [custom format](super::CustomFormat). Extension bits are not written and nothing is
/// written for a [custom](crate::Format::Custom) API version.
pub fn fmt(version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
    formatting::fmt(version, f)
}
