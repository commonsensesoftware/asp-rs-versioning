use crate::ApiVersion;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Key {
    pub(super) name: Option<String>,
    pub(super) api_version: Option<ApiVersion<'static>>,
}

impl Key {
    pub fn new(name: impl Into<String>, api_version: ApiVersion<'static>) -> Self {
        Self {
            name: Some(name.into()),
            api_version: Some(api_version),
        }
    }
}

impl From<&str> for Key {
    fn from(name: &str) -> Self {
        Self {
            name: Some(name.into()),
            api_version: None,
        }
    }
}

impl From<String> for Key {
    fn from(name: String) -> Self {
        Self {
            name: Some(name),
            api_version: None,
        }
    }
}

impl From<ApiVersion<'_>> for Key {
    fn from(api_version: ApiVersion<'_>) -> Self {
        Self::from(&api_version)
    }
}

impl From<&ApiVersion<'_>> for Key {
    fn from(api_version: &ApiVersion<'_>) -> Self {
        Self {
            name: None,
            api_version: Some(api_version.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version;

    #[test]
    fn new_should_set_name_and_api_version() {
        // arrange
        let api_version = api_version!(1.0);

        // act
        let key = Key::new("test", api_version.clone());

        // assert
        assert_eq!(key.name.as_deref(), Some("test"));
        assert_eq!(key.api_version, Some(api_version));
    }

    #[test]
    fn from_name_should_only_set_name() {
        // arrange
        let name = "test";

        // act
        let borrowed = Key::from(name);
        let owned = Key::from(name.to_owned());

        // assert
        assert_eq!(borrowed.name.as_deref(), Some(name));
        assert_eq!(borrowed.api_version, None);
        assert_eq!(borrowed, owned);
    }

    #[test]
    fn from_api_version_should_only_set_api_version() {
        // arrange
        let api_version = api_version!(1.0);

        // act
        let borrowed = Key::from(&api_version);
        let owned = Key::from(api_version.clone());

        // assert
        assert_eq!(borrowed.name, None);
        assert_eq!(borrowed.api_version, Some(api_version));
        assert_eq!(borrowed, owned);
    }

    #[test]
    fn keys_should_not_be_equal_for_different_scopes() {
        // arrange
        let api_version = api_version!(1.0);

        // act
        let specific = Key::new("test", api_version.clone());
        let api = Key::from("test");
        let version = Key::from(&api_version);

        // assert
        assert_ne!(specific, api);
        assert_ne!(specific, version);
        assert_ne!(api, version);
        assert_ne!(api, Key::default());
    }
}
