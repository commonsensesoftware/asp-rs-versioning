use time::Date;

/// Defines the behavior of an API version with a status.
pub trait ApiVersionStatusExt: Sized {
    /// Gets the associated API version status, if any.
    fn status(&self) -> Option<&str>;
}

/// Defines the behavior of a numeric API version.
pub trait NumericApiVersionExt: ApiVersionStatusExt {
    /// Gets the API major version, if any.
    fn major(&self) -> Option<u8>;

    /// Gets the API minor version, if any.
    fn minor(&self) -> Option<u8>;
}

/// Defines the behavior of a date-based API version.
pub trait DateApiVersionExt: ApiVersionStatusExt {
    /// Gets the API date version, if any.
    fn date(&self) -> Option<Date>;
}
