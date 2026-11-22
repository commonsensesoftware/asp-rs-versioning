use crate::ApiVersion;
use smallvec::{SmallVec, smallvec};

/// Represents variadic input - either a single value or multiple values.
pub enum Variadic<T> {
    /// Indicates a single value.
    One(T),

    /// Indicates multiple values.
    Many(Vec<T>),
}

impl<T> Variadic<T> {
    /// Converts the variadic input into a vector.
    pub fn to_vec(self) -> SmallVec<[T; 1]> {
        match self {
            Self::One(value) => smallvec![value],
            Self::Many(values) => SmallVec::from_vec(values),
        }
    }
}

impl From<ApiVersion<'static>> for Variadic<ApiVersion<'static>> {
    fn from(value: ApiVersion<'static>) -> Self {
        Self::One(value)
    }
}

impl<T: IntoIterator> From<T> for Variadic<T::Item> {
    fn from(value: T) -> Self {
        Self::Many(value.into_iter().collect())
    }
}
