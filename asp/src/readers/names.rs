use smallvec::{SmallVec, smallvec};
use std::borrow::Cow;

/// Defines the behavior of one or more names, such as the parameters or headers an API version is read from.
///
/// # Remarks
///
/// This trait improves the ergonomics of functions that accept one or more names. It is implemented for a single name
/// as a `&'static str`, [String], or [Cow], and for an array, [Vec], or slice of any type that converts into a [Cow].
///
/// # Examples
///
/// ```
/// use api_versioning::readers::QueryStringReader;
///
/// // matches '?ver=1.0'
/// let reader = QueryStringReader::new("ver");
///
/// // matches '?api-version=1.0' or '?v=1.0'
/// let reader = QueryStringReader::new(["api-version", "v"]);
/// ```
pub trait IntoNames: sealed::Sealed {
    /// Converts the value into a sequence of names.
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]>;
}

mod sealed {
    use std::borrow::Cow;

    pub trait Sealed {}

    impl Sealed for &'static str {}
    impl Sealed for String {}
    impl Sealed for Cow<'static, str> {}
    impl<T: Into<Cow<'static, str>>, const N: usize> Sealed for [T; N] {}
    impl<T: Into<Cow<'static, str>>> Sealed for Vec<T> {}
    impl<T: Into<Cow<'static, str>> + Clone> Sealed for &[T] {}
}

impl IntoNames for &'static str {
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]> {
        smallvec![self.into()]
    }
}

impl IntoNames for String {
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]> {
        smallvec![self.into()]
    }
}

impl IntoNames for Cow<'static, str> {
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]> {
        smallvec![self]
    }
}

impl<T: Into<Cow<'static, str>>, const N: usize> IntoNames for [T; N] {
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]> {
        self.into_iter().map(Into::into).collect()
    }
}

impl<T: Into<Cow<'static, str>>> IntoNames for Vec<T> {
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]> {
        self.into_iter().map(Into::into).collect()
    }
}

impl<T: Into<Cow<'static, str>> + Clone> IntoNames for &[T] {
    fn into_names(self) -> SmallVec<[Cow<'static, str>; 1]> {
        self.iter().cloned().map(Into::into).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::IntoNames;
    use std::borrow::Cow;

    #[test]
    fn into_names_should_convert_str() {
        // arrange
        let name = "ver";

        // act
        let names = name.into_names();

        // assert
        assert_eq!(names.as_slice(), ["ver"]);
    }

    #[test]
    fn into_names_should_convert_string() {
        // arrange
        let name = String::from("ver");

        // act
        let names = name.into_names();

        // assert
        assert_eq!(names.as_slice(), ["ver"]);
    }

    #[test]
    fn into_names_should_convert_cow() {
        // arrange
        let name: Cow<'static, str> = Cow::Owned("ver".into());

        // act
        let names = name.into_names();

        // assert
        assert_eq!(names.as_slice(), ["ver"]);
    }

    #[test]
    fn into_names_should_convert_array() {
        // arrange
        let names = ["api-version", "v"];

        // act
        let names = names.into_names();

        // assert
        assert_eq!(names.as_slice(), ["api-version", "v"]);
    }

    #[test]
    fn into_names_should_convert_vec() {
        // arrange
        let names = vec![String::from("api-version"), String::from("v")];

        // act
        let names = names.into_names();

        // assert
        assert_eq!(names.as_slice(), ["api-version", "v"]);
    }

    #[test]
    fn into_names_should_convert_slice() {
        // arrange
        let names: &[&'static str] = &["api-version", "v"];

        // act
        let names = names.into_names();

        // assert
        assert_eq!(names.as_slice(), ["api-version", "v"]);
    }
}
