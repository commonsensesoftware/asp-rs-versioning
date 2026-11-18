use self::ParseError::*;
use super::custom::{self, CustomFormat, custom_format};
use super::{ApiVersion, EPOCH, builtin, date_bits, numeric_bits};
use const_format::{StrWriter, concatcp, writec};
use std::any::{type_name, type_name_of_val};
use std::borrow::Cow;
use std::str::FromStr;
use thiserror::Error;
use time::{Date, Month, format_description::BorrowedFormatItem, macros::format_description};

const EM_PREFIX: &str = "the specified API version";
const EM_SUFFIX: &str = " is invalid";
const EM_FRAGMENT_SUFFIX: &str = concatcp!("'", EM_SUFFIX);
const EM_FORMAT: &str = concatcp!(EM_PREFIX, EM_SUFFIX);
const EM_DATE_PREFIX: &str = concatcp!(EM_PREFIX, " date '");
const EM_STATUS_PREFIX: &str = concatcp!(EM_PREFIX, " status '");
const EM_DATE: &str = concatcp!(EM_PREFIX, " date", EM_SUFFIX);
const EM_STATUS: &str = concatcp!(EM_PREFIX, " status", EM_SUFFIX);
const PANIC_MESSAGE_CAPACITY: usize = 256;

/// Represents the possible API version parsing errors.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// Indicates the format of the API version is invalid.
    #[error("{EM_FORMAT}")]
    InvalidFormat,

    /// Indicates the specified API version date is invalid.
    #[error("{prefix}{0}{suffix}", prefix = EM_DATE_PREFIX, suffix = EM_FRAGMENT_SUFFIX)]
    InvalidDate(String),

    /// Indicates the specified API version status is invalid.
    #[error("{prefix}{0}{suffix}", prefix = EM_STATUS_PREFIX, suffix = EM_FRAGMENT_SUFFIX)]
    InvalidStatus(String),

    /// Indicates the specified API version is invalid according to the [custom format](super::CustomFormat).
    #[error("{0}")]
    Custom(String),

    /// Indicates the [custom format](super::CustomFormat) recursively parsed an API version.
    ///
    /// # Remarks
    ///
    /// A custom format must use [builtin::parse] rather than [ApiVersion::parse] to parse a built-in format.
    #[error("the custom API version format recursively parsed an API version")]
    Recursion,
}

impl<'a> ApiVersion<'a> {
    /// Parses a numeric (`major[.minor]`) or date (`yyyy-mm-dd`) API version, optionally followed by a status beginning
    /// with an ASCII letter and containing dot-separated, non-empty ASCII alphanumeric components.
    ///
    /// Numeric components must be in the range 0-255 (e.g. one byte). Dates must be between year 2000 and 2255. The
    /// returned version and any error borrow their status or text from the input.
    ///
    /// # Remarks
    ///
    /// If a [custom format](CustomFormat) has been [set](super::set_custom_format), then the text is always parsed by
    /// the custom format; otherwise, the text is parsed by [builtin::parse].
    ///
    /// # Errors
    ///
    /// Returns [ParseError::Recursion] if the custom format calls this function while parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_versioning::{ApiVersion, ParseError};
    ///
    /// let result: Result<ApiVersion<'static>, ParseError> =
    ///     ApiVersion::parse("1.0-beta");
    /// ```
    pub fn parse(text: &'a str) -> Result<Self, ParseError> {
        Self::parse_with(text, custom_format())
    }

    fn parse_with(text: &'a str, custom: Option<&dyn CustomFormat>) -> Result<Self, ParseError> {
        match custom {
            Some(format) => custom::parse(format, text),
            None => builtin::parse(text),
        }
    }

    /// Parses a version for use in constant expressions and literal macros.
    ///
    /// # Remarks
    ///
    /// Although this function is publicly available, consider using the [api_version!!][api_version!] macro for more
    /// idiomatic use.
    ///
    /// This function uses the same rules as [parse][Self::parse], but avoids matching a result containing potentially
    /// owned errors during constant evaluation.
    ///
    /// # Panics
    ///
    /// Panics if the input is invalid. In a constant expression, this causes a compilation error. Errors include the
    /// invalid fragment when the message fits in a 256-byte buffer; otherwise a generic message is provided without the
    /// fragment.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_versioning::ApiVersion;
    ///
    /// const VERSION: ApiVersion<'static> = ApiVersion::parse_literal("1.0-beta");
    /// ```
    ///
    /// ```compile_fail
    /// use api_versioning::ApiVersion;
    ///
    /// const VERSION: ApiVersion<'static> = ApiVersion::parse_literal("256");
    /// ```
    ///
    /// ```compile_fail
    /// use api_versioning::ApiVersion;
    ///
    /// const VERSION: ApiVersion<'static> = ApiVersion::parse_literal("2023-02-29");
    /// ```
    ///
    /// ```compile_fail
    /// use api_versioning::ApiVersion;
    ///
    /// const VERSION: ApiVersion<'static> = ApiVersion::parse_literal("1.0-2beta");
    /// ```
    pub const fn parse_literal(text: &'a str) -> Self {
        match parse_parts(text) {
            Ok((bits, Some(status))) => Self::new(bits, Some(Cow::Borrowed(status))),
            Ok((bits, None)) => Self::new(bits, None),
            Err(error) => error.panic(),
        }
    }

    /// Parses a list of API versions from the path of the specified type.
    ///
    /// # Remarks
    ///
    /// A module identifier can use `v` or `_` as a prefix. `mod _2018_04_01` is equivalent to `mod v2018_04_01`.
    ///
    /// The last segment of the path is the name of an item, such as a type, function, or module. It can be a module
    /// identifier or end with a version suffix that begins with `_v`. `fn hello_v1` and `fn get_v2018_04_01_beta` have
    /// the suffixes `v1` and `v2018_04_01_beta`. Generic arguments are ignored.
    ///
    /// If a [custom format](CustomFormat) has been [set](super::set_custom_format), then each module identifier and
    /// version suffix is always parsed by the custom format; otherwise, it is parsed by [builtin::parse_path].
    ///
    /// The result describes every API version found. Use [infer_from_type](Self::infer_from_type) when exactly one API
    /// version is expected.
    ///
    /// # Panics
    ///
    /// Panics if the custom format calls this function while parsing a module identifier or version suffix.
    pub fn from_type<T: ?Sized>() -> Vec<ApiVersion<'static>> {
        Self::from_type_name(type_name::<T>())
    }

    /// Parses a list of API versions from the path of the type of the specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - the value whose type path is parsed
    ///
    /// # Remarks
    ///
    /// This function has the same rules as [from_type](Self::from_type). It is useful when the type cannot be named,
    /// such as a function item.
    ///
    /// # Panics
    ///
    /// Panics if the custom format calls this function while parsing a module identifier or version suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_versioning::{ApiVersion, api_version};
    ///
    /// mod v1 {
    ///     pub fn get() {}
    /// }
    ///
    /// fn hello_v2() {}
    ///
    /// assert_eq!(ApiVersion::from_val(&v1::get), [api_version!(1)]);
    /// assert_eq!(ApiVersion::from_val(&hello_v2), [api_version!(2)]);
    /// ```
    pub fn from_val<T: ?Sized>(value: &T) -> Vec<ApiVersion<'static>> {
        Self::from_type_name(type_name_of_val(value))
    }

    /// Infers exactly one API version from the path of the specified type.
    ///
    /// # Remarks
    ///
    /// This function has the same rules as [from_type](Self::from_type). The same API version can be found more than
    /// once.
    ///
    /// # Panics
    ///
    /// Panics if no API version or more than one distinct API version is found, or if the custom format calls this
    /// function while parsing a module identifier or version suffix.
    #[track_caller]
    pub fn infer_from_type<T: ?Sized>() -> ApiVersion<'static> {
        let name = type_name::<T>();
        single(name, Self::from_type_name(name))
    }

    /// Infers exactly one API version from the path of the type of the specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - the value whose type path is parsed
    ///
    /// # Remarks
    ///
    /// This function has the same rules as [from_type](Self::from_type). The same API version can be found more than
    /// once. The path of a function item is the path where it is defined rather than the path it is used by. Use
    /// `api_version!(use path)` to infer an API version from a written path.
    ///
    /// # Panics
    ///
    /// Panics if no API version or more than one distinct API version is found, or if the custom format calls this
    /// function while parsing a module identifier or version suffix.
    #[track_caller]
    pub fn infer_from_val<T: ?Sized>(value: &T) -> ApiVersion<'static> {
        let name = type_name_of_val(value);
        single(name, Self::from_type_name(name))
    }

    /// Infers exactly one API version from the specified item path.
    ///
    /// # Arguments
    ///
    /// * `path` - the path of a module, function, or type to parse, such as the expansion of [module_path!]
    ///
    /// # Remarks
    ///
    /// This function has the same rules as [from_type](Self::from_type). The same API version can be found more than
    /// once. Only the specified path is parsed; an alias is not resolved to the path where the item is defined. Consider
    /// using `api_version!(use path)` for more idiomatic use.
    ///
    /// # Panics
    ///
    /// Panics if no API version or more than one distinct API version is found, or if the custom format calls this
    /// function while parsing a module identifier or version suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_versioning::{ApiVersion, api_version};
    ///
    /// mod v1 {
    ///     pub fn version() -> api_versioning::ApiVersion<'static> {
    ///         api_versioning::ApiVersion::infer_from_path(module_path!())
    ///     }
    /// }
    ///
    /// assert_eq!(v1::version(), api_version!(1));
    /// assert_eq!(ApiVersion::infer_from_path("orders::hello_v2"), api_version!(2));
    /// ```
    #[track_caller]
    pub fn infer_from_path(path: &str) -> ApiVersion<'static> {
        single(path, Self::from_type_name(path))
    }

    fn from_type_name(name: &str) -> Vec<ApiVersion<'_>> {
        Self::from_type_name_with(name, custom_format())
    }

    fn from_type_name_with<'n>(name: &'n str, custom: Option<&dyn CustomFormat>) -> Vec<ApiVersion<'n>> {
        let name = name.split_once('<').map_or(name, |(path, _)| path);
        let mut segments = name.split("::").peekable();
        let mut versions = Vec::new();

        while let Some(segment) = segments.next() {
            let mut version = parse_segment(segment, custom);

            if version.is_none() && segments.peek().is_none() {
                version = parse_suffix(segment, custom);
            }

            versions.extend(version);
        }

        versions
    }
}

fn parse_segment<'n>(segment: &'n str, custom: Option<&dyn CustomFormat>) -> Option<ApiVersion<'n>> {
    match custom {
        Some(format) => custom::parse_path(format, segment),
        None => builtin::parse_path(segment),
    }
}

// a version suffix starts at a 'v' that begins the name or follows '_' and continues to the end of the name
fn parse_suffix<'n>(name: &'n str, custom: Option<&dyn CustomFormat>) -> Option<ApiVersion<'n>> {
    name.match_indices('v')
        .filter(|&(i, _)| i == 0 || name.as_bytes()[i - 1] == b'_')
        .find_map(|(i, _)| parse_segment(&name[i..], custom))
}

#[track_caller]
fn single(path: &str, versions: Vec<ApiVersion<'_>>) -> ApiVersion<'static> {
    match versions.split_first() {
        Some((first, rest)) if rest.iter().all(|version| version == first) => first.to_owned(),
        Some(_) => {
            let versions = versions.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            panic!("Exactly one API version must be inferred from '{path}', but found: {versions}.")
        }
        None => panic!("An API version could not be inferred from '{path}'."),
    }
}

pub(super) fn parse(text: &str) -> Result<ApiVersion<'_>, ParseError> {
    match parse_parts(text) {
        Ok((bits, Some(status))) => Ok(ApiVersion::new(bits, Some(Cow::Borrowed(status)))),
        Ok((bits, None)) => Ok(ApiVersion::new(bits, None)),
        Err(ParseFailure::Format) => Err(InvalidFormat),
        Err(ParseFailure::Date(text)) => Err(InvalidDate(text.into())),
        Err(ParseFailure::Status(text)) => Err(InvalidStatus(text.into())),
    }
}

#[derive(Clone, Copy)]
enum ParseFailure<'a> {
    Format,
    Date(&'a str),
    Status(&'a str),
}

impl ParseFailure<'_> {
    const fn panic(self) -> ! {
        let writer: &mut StrWriter = &mut StrWriter::new([0; PANIC_MESSAGE_CAPACITY]);
        let (result, fallback) = match self {
            Self::Format => panic!("{}", EM_FORMAT),
            Self::Date(text) => (writec!(writer, "{EM_DATE_PREFIX}{text}{EM_FRAGMENT_SUFFIX}"), EM_DATE),
            Self::Status(text) => (
                writec!(writer, "{EM_STATUS_PREFIX}{text}{EM_FRAGMENT_SUFFIX}"),
                EM_STATUS,
            ),
        };

        match result {
            Ok(()) => panic!("{}", writer.as_str()),
            Err(_) => panic!("{}", fallback),
        }
    }
}

const fn find_byte(text: &str, byte: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == byte {
            return Some(i);
        }

        i += 1;
    }

    None
}

const fn read_u8(text: &str) -> Option<u8> {
    let bytes = text.as_bytes();
    let mut i = if !bytes.is_empty() && bytes[0] == b'+' { 1 } else { 0 };

    if i == bytes.len() {
        return None;
    }

    let mut value = 0u16;

    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return None;
        }

        value = value * 10 + (bytes[i] - b'0') as u16;

        if value > u8::MAX as u16 {
            return None;
        }

        i += 1;
    }

    Some(value as u8)
}

const fn read_major_minor(text: &str) -> Result<(u8, u8), ParseFailure<'_>> {
    let (lhs, rhs) = match find_byte(text, b'.') {
        Some(i) => {
            let (lhs, tail) = text.split_at(i);
            (lhs, Some(tail.split_at(1).1))
        }
        None => (text, None),
    };
    let major = match read_u8(lhs) {
        Some(value) => value,
        None => return Err(ParseFailure::Format),
    };
    let minor = match rhs {
        Some(text) => match read_u8(text) {
            Some(value) => value,
            None => return Err(ParseFailure::Format),
        },
        None => 0,
    };

    Ok((major, minor))
}

const fn read_status(text: &str) -> Result<&str, ParseFailure<'_>> {
    let bytes = text.as_bytes();

    if bytes.is_empty() {
        return Err(ParseFailure::Format);
    }

    if !bytes[0].is_ascii_alphabetic() {
        return Err(ParseFailure::Status(text));
    }

    let mut i = 1;

    while i < bytes.len() {
        if bytes[i] == b'.' {
            if i + 1 == bytes.len() || !bytes[i + 1].is_ascii_alphanumeric() {
                return Err(ParseFailure::Status(text));
            }
        } else if !bytes[i].is_ascii_alphanumeric() {
            return Err(ParseFailure::Status(text));
        }

        i += 1;
    }

    Ok(text)
}

const fn date_like(text: &str) -> bool {
    let bytes = text.as_bytes();

    if bytes.len() < 10 {
        return false;
    }

    let mut i = 0;

    while i < 10 {
        if i == 4 || i == 7 {
            if bytes[i] != b'-' {
                return false;
            }
        } else if !bytes[i].is_ascii_digit() {
            return false;
        }

        i += 1;
    }

    true
}

const fn read_date(text: &str) -> Result<Date, ParseFailure<'_>> {
    let bytes = text.as_bytes();
    let year = (bytes[0] - b'0') as i32 * 1000
        + (bytes[1] - b'0') as i32 * 100
        + (bytes[2] - b'0') as i32 * 10
        + (bytes[3] - b'0') as i32;

    if year < EPOCH as i32 || year > EPOCH as i32 + u8::MAX as i32 {
        return Err(ParseFailure::Date(text));
    }

    let month = match (bytes[5] - b'0') * 10 + bytes[6] - b'0' {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        _ => return Err(ParseFailure::Date(text)),
    };

    let day = (bytes[8] - b'0') * 10 + bytes[9] - b'0';

    match Date::from_calendar_date(year, month, day) {
        Ok(date) => Ok(date),
        Err(_) => Err(ParseFailure::Date(text)),
    }
}

const fn parse_parts(text: &str) -> Result<(u32, Option<&str>), ParseFailure<'_>> {
    if date_like(text) {
        let (segment, tail) = text.split_at(10);
        let date = match read_date(segment) {
            Ok(date) => date,
            Err(error) => return Err(error),
        };
        let status = if tail.is_empty() {
            None
        } else if tail.as_bytes()[0] == b'-' {
            match read_status(tail.split_at(1).1) {
                Ok(status) => Some(status),
                Err(error) => return Err(error),
            }
        } else {
            return Err(ParseFailure::Format);
        };

        return Ok((date_bits(date), status));
    }

    let (segment, status) = match find_byte(text, b'-') {
        Some(i) => {
            let (segment, tail) = text.split_at(i);

            match read_status(tail.split_at(1).1) {
                Ok(status) => (segment, Some(status)),
                Err(error) => return Err(error),
            }
        }
        None => (text, None),
    };

    match read_major_minor(segment) {
        Ok((major, minor)) => Ok((numeric_bits(major, minor), status)),
        Err(error) => Err(error),
    }
}

// ( 'v' | '_' ) : ( [<year> : ['_'] : <month> : ['_'] : <day>] | [<major> ['_' : <minor>]] ) : ['_'] : [<status>]
//
// - v1
// - v1_1
// - v2_0_Beta
// - v20180401
// - _1
// - _1_1
// - _20180401
// - _2018_04_01
// - _2018_04_01_Beta
pub(super) fn parse_path(mut ident: &str) -> Option<ApiVersion<'_>> {
    if let Some(ch) = ident.chars().next() {
        match ch {
            'v' | '_' => ident = &ident[1..],
            _ => return None,
        };

        let mut bits = 0;

        if ident.len() >= 8 {
            if let (remaining, Some(date)) = consume_date(ident, 8, format_description!("[year][month][day]")) {
                ident = remaining;
                bits = date_bits(date);
            } else if ident.len() >= 10
                && let (remaining, Some(date)) = consume_date(ident, 10, format_description!("[year]_[month]_[day]"))
            {
                ident = remaining;
                bits = date_bits(date);
            }
        }

        let status;

        if ident.is_empty() {
            if bits > 0 {
                Some(ApiVersion::new(bits, None))
            } else {
                None
            }
        } else if bits == 0
            && let (remaining, Some(major)) = consume_number(ident)
        {
            let (remaining, minor) = consume_number(remaining);

            bits = numeric_bits(major, minor.unwrap_or_default());
            status = if remaining.is_empty() {
                None
            } else if let Ok(remaining) = read_status(remaining) {
                Some(Cow::Owned(remaining.to_string()))
            } else {
                return None;
            };

            Some(ApiVersion::new(bits, status))
        } else if bits > 0
            && let Ok(remaining) = read_status(ident)
        {
            status = Some(Cow::Owned(remaining.to_string()));
            Some(ApiVersion::new(bits, status))
        } else {
            None
        }
    } else {
        None
    }
}

fn consume_date<'a>(ident: &'a str, len: usize, format: &[BorrowedFormatItem]) -> (&'a str, Option<Date>) {
    let segment = &ident[..len];

    if let Ok(date) = Date::parse(segment, format) {
        if ident.chars().nth(len) == Some('_') {
            (&ident[(len + 1)..], Some(date))
        } else {
            (&ident[len..], Some(date))
        }
    } else {
        (ident, None)
    }
}

fn consume_number(ident: &str) -> (&str, Option<u8>) {
    let mut len = 0;
    let mut has_sep = false;

    for ch in ident.chars() {
        if ch.is_ascii_digit() {
            len += 1;
        } else {
            has_sep = len > 0 && ch == '_';
            break;
        }
    }

    if len > 0
        && let Ok(number) = ident[..len].parse::<u8>()
    {
        if has_sep {
            len += 1;
        }

        (&ident[len..], Some(number))
    } else {
        (ident, None)
    }
}

impl<'a> TryFrom<&'a str> for ApiVersion<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for ApiVersion<'_> {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        ApiVersion::try_from(text).map(|v| Self::new(v.0, v.1.map(|s| Cow::Owned(s.into_owned()))))
    }
}

#[cfg(test)]
mod tests;
