use super::{ApiVersion, ParseError, builtin};
use std::cell::Cell;
use std::fmt::{self, Formatter};
use std::sync::OnceLock;
use std::thread::LocalKey;
use thiserror::Error;

static FORMAT: OnceLock<&'static dyn CustomFormat> = OnceLock::new();

thread_local! {
    static PARSING: Cell<bool> = const { Cell::new(false) };
    static PARSING_PATH: Cell<bool> = const { Cell::new(false) };
    static FORMATTING: Cell<bool> = const { Cell::new(false) };
}

/// Defines the behavior of a custom API version format.
///
/// # Remarks
///
/// At most one custom format can be installed per process using [set_custom_format]. Once installed, the custom format
/// is always used to parse and display API versions, except for a neutral API version, which can never be
/// reinterpreted. The [builtin] module provides the built-in behaviors so that a custom format can use them for some or
/// all of its work.
///
/// A custom format can extend a built-in format by storing an additional component in the extension bits of
/// [ApiVersion::numeric] or [ApiVersion::date], which keeps the built-in components readable. A format that is neither
/// numeric nor date-based uses [ApiVersion::custom].
///
/// API versions are always compared by their bits and then by status, where an API version without a status is greater
/// than the same API version with a status. A custom format must encode its bits such that their numeric order
/// matches the intended order of the API versions.
///
/// A custom format must not call [ApiVersion::parse], [ApiVersion::from_type], or display an [ApiVersion] from the
/// corresponding function it implements; use the [builtin] functions instead. Recursive calls are detected and fail
/// rather than overflowing the stack.
///
/// The [api_version!](crate::api_version!) macro does not support custom formats. A custom format can provide its own
/// macro by pairing a `const` parsing function with [ApiVersion::numeric], [ApiVersion::date], or [ApiVersion::custom].
///
/// # Examples
///
/// A format that adds a patch version to the numeric format (e.g. `1.2.3`). The patch version is stored in the numeric
/// extension bits and read back through an extension trait in the same manner as the built-in components.
///
/// ```
/// use api_versioning::builtin::{self, mask};
/// use api_versioning::{prelude::*, set_custom_format, ApiVersion, CustomFormat, ParseError};
/// use std::borrow::Cow;
/// use std::fmt::{self, Formatter};
///
/// /// Defines the behavior of a numeric API version with a patch version.
/// pub trait PatchApiVersionExt: NumericApiVersionExt {
///     /// Gets the API patch version, if any.
///     fn patch(&self) -> Option<u16>;
/// }
///
/// impl PatchApiVersionExt for ApiVersion<'_> {
///     fn patch(&self) -> Option<u16> {
///         // only a numeric API version has a patch version; 14 bits always fit in a u16
///         self.major().map(|_| (self.bits() & mask::NUMERIC) as u16)
///     }
/// }
///
/// struct Patch;
///
/// impl CustomFormat for Patch {
///     fn parse<'a>(&self, text: &'a str) -> Result<ApiVersion<'a>, ParseError> {
///         let (version, status) = match text.split_once('-') {
///             Some((version, status)) => (version, Some(Cow::Borrowed(status))),
///             None => (text, None),
///         };
///
///         // anything other than <major>.<minor>.<patch> is a built-in format
///         let Some((major_minor, patch)) = version.rsplit_once('.').filter(|(major_minor, _)| major_minor.contains('.'))
///         else {
///             return builtin::parse(text);
///         };
///
///         let patch = match patch.parse::<u32>() {
///             Ok(patch) if patch <= mask::NUMERIC => patch,
///             _ => return Err(ParseError::Custom(format!("the patch version '{patch}' is invalid"))),
///         };
///         let base = builtin::parse(major_minor)?;
///
///         Ok(ApiVersion::numeric(base.major().unwrap(), base.minor().unwrap(), patch, status))
///     }
///
///     fn fmt(&self, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
///         match (version.major(), version.minor(), version.patch()) {
///             (Some(major), Some(minor), Some(patch)) if patch > 0 => {
///                 write!(f, "{major}.{minor}.{patch}")?;
///
///                 if let Some(status) = version.status() {
///                     write!(f, "-{status}")?;
///                 }
///
///                 Ok(())
///             }
///             _ => builtin::fmt(version, f),
///         }
///     }
/// }
///
/// const V1_2_3: ApiVersion<'static> = ApiVersion::numeric(1, 2, 3, None);
///
/// set_custom_format(&Patch).unwrap();
///
/// let version = ApiVersion::parse("1.2.3-beta").unwrap();
///
/// assert_eq!(version.major(), Some(1));
/// assert_eq!(version.minor(), Some(2));
/// assert_eq!(version.patch(), Some(3));
/// assert_eq!(version.status(), Some("beta"));
/// assert_eq!(version.to_string(), "1.2.3-beta");
/// assert!(version < V1_2_3);
/// assert!(V1_2_3 < api_versioning::api_version!(1.3));
///
/// let version = ApiVersion::parse("1.2").unwrap();
///
/// assert_eq!(version.patch(), Some(0));
/// assert_eq!(version.to_string(), "1.2");
///
/// let version = ApiVersion::parse("2026-10-01").unwrap();
///
/// assert_eq!(version.patch(), None);
/// assert_eq!(version.to_string(), "2026-10-01");
/// ```
pub trait CustomFormat: Send + Sync + 'static {
    /// Parses the specified text into an API version.
    ///
    /// # Arguments
    ///
    /// * `text` - the text to parse
    ///
    /// # Remarks
    ///
    /// This function is called for all text. Use [builtin::parse] to parse the built-in formats.
    fn parse<'a>(&self, text: &'a str) -> Result<ApiVersion<'a>, ParseError>;

    /// Formats the specified API version.
    ///
    /// # Arguments
    ///
    /// * `version` - the [API version](ApiVersion) to format
    /// * `f` - the [formatter](Formatter) to write to
    ///
    /// # Remarks
    ///
    /// This function is called for all API versions except a neutral API version. Use [builtin::fmt] to format the
    /// built-in formats. The alternate form is requested when [Formatter::alternate] is `true`.
    fn fmt(&self, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result;

    /// Parses an API version from a type path segment, such as a module name, if any.
    ///
    /// # Arguments
    ///
    /// * `segment` - the type path segment to parse
    ///
    /// # Remarks
    ///
    /// This function is used by [ApiVersion::from_type] for each module identifier and version suffix. The default
    /// implementation uses [builtin::parse_path].
    fn parse_path<'a>(&self, segment: &'a str) -> Option<ApiVersion<'a>> {
        builtin::parse_path(segment)
    }
}

/// Represents the error that occurs when a custom API version format has already been set.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("a custom API version format has already been set")]
pub struct SetCustomFormatError;

/// Sets the custom API version format for the process.
///
/// # Arguments
///
/// * `format` - the [custom format](CustomFormat) to use
///
/// # Remarks
///
/// The custom format can only be set once and should be set by the application before any API versions are parsed or
/// formatted. Libraries should not set the custom format.
///
/// # Errors
///
/// Returns [SetCustomFormatError] if a custom format has already been set.
pub fn set_custom_format(format: &'static dyn CustomFormat) -> Result<(), SetCustomFormatError> {
    FORMAT.set(format).map_err(|_| SetCustomFormatError)
}

pub(crate) fn custom_format() -> Option<&'static dyn CustomFormat> {
    FORMAT.get().copied()
}

pub(super) fn parse<'a>(format: &dyn CustomFormat, text: &'a str) -> Result<ApiVersion<'a>, ParseError> {
    let _guard = Reentry::enter(&PARSING).ok_or(ParseError::Recursion)?;
    format.parse(text)
}

pub(super) fn parse_path<'a>(format: &dyn CustomFormat, segment: &'a str) -> Option<ApiVersion<'a>> {
    let Some(_guard) = Reentry::enter(&PARSING_PATH) else {
        panic!("the custom API version format recursively parsed a type path");
    };

    format.parse_path(segment)
}

pub(super) fn fmt(format: &dyn CustomFormat, version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
    let _guard = Reentry::enter(&FORMATTING).ok_or(fmt::Error)?;
    format.fmt(version, f)
}

// marks an operation as entered on the current thread until dropped, including when unwinding from a panic
struct Reentry(&'static LocalKey<Cell<bool>>);

impl Reentry {
    fn enter(entered: &'static LocalKey<Cell<bool>>) -> Option<Self> {
        if entered.replace(true) {
            None
        } else {
            Some(Self(entered))
        }
    }
}

impl Drop for Reentry {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
