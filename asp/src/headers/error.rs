use thiserror::Error;

/// Represents the possible attribute errors.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum AttributeNameError {
    /// Indicates the name was empty.
    #[error("the name cannot be empty")]
    Empty,

    /// Indicates the first later was not a letter.
    #[error("the first character must be a letter")]
    InvalidStart,

    /// Indicates a character was not a letter, number, `.`, or `_`.
    #[error("only letters, numbers, '.', '-', and '_' are allowed")]
    InvalidCharacter,
}
