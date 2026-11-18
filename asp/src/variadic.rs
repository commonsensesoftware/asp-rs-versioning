use crate::ApiVersion;
use smallvec::{SmallVec, smallvec};

pub enum Variadic<T> {
    One(T),
    Many(Vec<T>),
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

impl<T> Variadic<T> {
    pub fn to_vec(self) -> SmallVec<[T; 1]> {
        match self {
            Self::One(value) => smallvec![value],
            Self::Many(values) => SmallVec::from_vec(values),
        }
    }
}
