use crate::ApiVersion;

/// Represent the possible API version policy targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Indicates all versions of the specified API.
    Api(String),

    /// Indicates all APIs with the specified version.
    Version(ApiVersion<'static>),

    /// Indicates a specified API and version.
    Specific {
        api: String,
        api_version: ApiVersion<'static>,
    },
}

impl From<String> for Target {
    fn from(name: String) -> Self {
        Self::from(name.as_str())
    }
}

impl From<&str> for Target {
    fn from(name: &str) -> Self {
        Self::Api(name.into())
    }
}

impl From<ApiVersion<'_>> for Target {
    fn from(api_version: ApiVersion<'_>) -> Self {
        Self::from(&api_version)
    }
}

impl From<&ApiVersion<'_>> for Target {
    fn from(api_version: &ApiVersion<'_>) -> Self {
        Self::Version(api_version.to_owned())
    }
}

impl From<(String, ApiVersion<'_>)> for Target {
    fn from(value: (String, ApiVersion<'_>)) -> Self {
        Self::from((value.0.as_str(), &value.1))
    }
}

impl From<(&str, &ApiVersion<'_>)> for Target {
    fn from((api, api_version): (&str, &ApiVersion<'_>)) -> Self {
        Self::Specific {
            api: api.to_owned(),
            api_version: api_version.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version;

    #[test]
    fn from_name_should_target_api() {
        // arrange
        let expected = Target::Api("test".into());

        // act
        let borrowed = Target::from("test");
        let owned = Target::from(String::from("test"));

        // assert
        assert_eq!(borrowed, expected);
        assert_eq!(owned, expected);
    }

    #[test]
    fn from_api_version_should_target_version() {
        // arrange
        let api_version = api_version!(1.0);
        let expected = Target::Version(api_version.clone());

        // act
        let borrowed = Target::from(&api_version);
        let owned = Target::from(api_version);

        // assert
        assert_eq!(borrowed, expected);
        assert_eq!(owned, expected);
    }

    #[test]
    fn from_tuple_should_target_specific_api_and_version() {
        // arrange
        let api_version = api_version!(1.0);
        let expected = Target::Specific {
            api: "test".into(),
            api_version: api_version.clone(),
        };

        // act
        let borrowed = Target::from(("test", &api_version));
        let owned = Target::from((String::from("test"), api_version));

        // assert
        assert_eq!(borrowed, expected);
        assert_eq!(owned, expected);
    }
}
