use super::ApiVersion;
use serde::ser::{Serialize, Serializer};

impl Serialize for ApiVersion<'_> {
    /// Serializes the API version as a string in the same format as [Display](std::fmt::Display).
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::ApiVersion;
    use rstest::rstest;

    #[rstest]
    #[case(api_version!(1.0), r#""1.0""#)]
    #[case(api_version!(2.1-preview.1), r#""2.1-preview.1""#)]
    #[case(api_version!(2026-10-01), r#""2026-10-01""#)]
    fn serialize_should_write_api_version_as_string(#[case] version: ApiVersion<'static>, #[case] expected: &str) {
        // arrange

        // act
        let json = serde_json::to_string(&version).unwrap();

        // assert
        assert_eq!(json, expected);
    }

    #[test]
    fn serialize_should_round_trip_with_deserialize() {
        // arrange
        let version = api_version!(2.0-beta);

        // act
        let json = serde_json::to_string(&version).unwrap();
        let result: ApiVersion<'static> = serde_json::from_str(&json).unwrap();

        // assert
        assert_eq!(result, version);
    }
}
