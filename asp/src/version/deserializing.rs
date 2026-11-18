use super::ApiVersion;
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use std::fmt::{self, Formatter};

struct ApiVersionVisitor;

impl Visitor<'_> for ApiVersionVisitor {
    type Value = ApiVersion<'static>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("an API version")
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        ApiVersion::parse(value)
            .map(|version| version.to_owned())
            .map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for ApiVersion<'static> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ApiVersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::ApiVersion;
    use rstest::rstest;
    use serde::de::{
        Deserialize,
        value::{Error, StrDeserializer},
    };

    #[rstest]
    #[case("1", api_version!(1.0))]
    #[case("2.1-preview.1", api_version!(2.1-preview.1))]
    #[case("2026-10-01", api_version!(2026-10-01))]
    fn deserialize_should_parse_api_version(#[case] text: &str, #[case] expected: ApiVersion<'static>) {
        // arrange
        let deserializer = StrDeserializer::<Error>::new(text);

        // act
        let version = ApiVersion::deserialize(deserializer).unwrap();

        // assert
        assert_eq!(version, expected);
    }

    #[test]
    fn deserialize_should_fail_when_api_version_is_invalid() {
        // arrange
        let deserializer = StrDeserializer::<Error>::new("abc");

        // act
        let result = ApiVersion::deserialize(deserializer);

        // assert
        assert!(result.is_err());
    }
}
