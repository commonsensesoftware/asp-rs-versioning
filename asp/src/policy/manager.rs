use super::Key;
use crate::policy::{DeprecationPolicy, Policy, SunsetPolicy};
use crate::{ApiVersion, Options};
use std::collections::HashMap;

/// Represents an API versioning policy manager.
#[derive(Clone, Debug, Default)]
pub struct Manager<T: Policy>(HashMap<Key, T>);

impl<T: Policy> Manager<T> {
    /// Gets the policy defined for the specific API and version, if any.
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the API to get the policy for
    /// * `api_version` - the [API version](ApiVersion) to get the policy for
    pub fn get(&self, name: &str, api_version: &ApiVersion<'_>) -> Option<&T> {
        self.0.get(&Key::new(name, api_version.to_owned()))
    }

    /// Gets the policy defined for any version of the specified API, if any.
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the API to get the policy for
    pub fn for_api(&self, name: &str) -> Option<&T> {
        self.0.get(&Key::from(name))
    }

    /// Gets the policy defined for any API with the specified version, if any.
    ///
    /// # Arguments
    ///
    /// * `api_version` - the [API version](ApiVersion) to get the policy for
    pub fn for_api_version(&self, api_version: &ApiVersion<'_>) -> Option<&T> {
        self.0.get(&Key::from(api_version))
    }

    /// Attempts to resolve a policy for the specified name and API version combination.
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the API to resolve the policy for, if any
    /// * `api_version` - the [API version](ApiVersion) to resolve the policy for, if any
    ///
    /// # Remarks
    ///
    /// The resolution follows this order of precedence:
    /// 1. Policy defined for the specific API and version.
    /// 2. Policy defined for any version of the specified API.
    /// 3. Policy defined for any API with the specified version.
    /// 4. None if no matching policy is found.
    pub fn resolve(&self, name: Option<&str>, api_version: Option<&ApiVersion<'_>>) -> Option<&T> {
        if let Some(name) = name {
            if let Some(api_version) = api_version {
                self.get(name, api_version)
                    .or_else(|| self.for_api(name))
                    .or_else(|| self.for_api_version(api_version))
            } else {
                self.for_api(name)
            }
        } else if let Some(api_version) = api_version {
            self.for_api_version(api_version)
        } else {
            None
        }
    }
}

macro_rules! manager {
    ($policy:ident) => {
        impl Manager<$policy> {
            /// Initializes a new policy [Manager].
            ///
            /// # Arguments
            ///
            /// * `options` - the API versioning [options](Options)
            pub fn new(options: &Options) -> Self {
                let mut map = HashMap::new();

                for builder in options.policies.of::<$policy>() {
                    let key = Key {
                        name: builder.name().map(Into::into),
                        api_version: builder.api_version().map(ApiVersion::to_owned),
                    };
                    let policy = builder.build();

                    map.insert(key, policy);
                }

                Self(map)
            }
        }
    };
}

manager!(DeprecationPolicy);
manager!(SunsetPolicy);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn date(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn new_should_be_empty_without_policies() {
        // arrange
        let options = Options::default();
        let api_version = api_version!(1.0);

        // act
        let manager = Manager::<DeprecationPolicy>::new(&options);

        // assert
        assert!(manager.get("test", &api_version).is_none());
        assert!(manager.for_api("test").is_none());
        assert!(manager.for_api_version(&api_version).is_none());
    }

    #[test]
    fn get_should_return_policy_for_specific_api_and_version() {
        // arrange
        let api_version = api_version!(1.0);
        let mut options = Options::default();

        options.policies.deprecate(("test", &api_version)).as_of(date(1));

        let manager = Manager::<DeprecationPolicy>::new(&options);

        // act
        let policy = manager.get("test", &api_version);

        // assert
        assert_eq!(policy.and_then(DeprecationPolicy::date), Some(date(1)));
        assert!(manager.for_api("test").is_none());
        assert!(manager.for_api_version(&api_version).is_none());
    }

    #[test]
    fn for_api_should_return_policy_for_all_versions_of_api() {
        // arrange
        let api_version = api_version!(1.0);
        let mut options = Options::default();

        options.policies.sunset("test").as_of(date(2));

        let manager = Manager::<SunsetPolicy>::new(&options);

        // act
        let policy = manager.for_api("test");

        // assert
        assert_eq!(policy.and_then(SunsetPolicy::date), Some(date(2)));
        assert!(manager.get("test", &api_version).is_none());
        assert!(manager.for_api("other").is_none());
    }

    #[test]
    fn for_api_version_should_return_policy_for_all_apis_with_version() {
        // arrange
        let api_version = api_version!(1.0);
        let mut options = Options::default();

        options.policies.sunset(&api_version).as_of(date(3));

        let manager = Manager::<SunsetPolicy>::new(&options);

        // act
        let policy = manager.for_api_version(&api_version);

        // assert
        assert_eq!(policy.and_then(SunsetPolicy::date), Some(date(3)));
        assert!(manager.get("test", &api_version).is_none());
        assert!(manager.for_api_version(&api_version!(2.0)).is_none());
    }

    #[test]
    fn new_should_only_include_policies_of_manager_type() {
        // arrange
        let api_version = api_version!(1.0);
        let mut options = Options::default();

        options.policies.deprecate(("test", &api_version)).as_of(date(1));
        options.policies.sunset(("test", &api_version)).as_of(date(2));

        // act
        let deprecations = Manager::<DeprecationPolicy>::new(&options);
        let sunsets = Manager::<SunsetPolicy>::new(&options);

        // assert
        assert_eq!(
            deprecations.get("test", &api_version).and_then(DeprecationPolicy::date),
            Some(date(1))
        );
        assert_eq!(
            sunsets.get("test", &api_version).and_then(SunsetPolicy::date),
            Some(date(2))
        );
    }

    #[test]
    fn get_should_match_api_version_status() {
        // arrange
        let release = api_version!(1.0);
        let preview = api_version!(1.0 - preview);
        let mut options = Options::default();

        options.policies.deprecate(("test", &preview)).as_of(date(1));

        let manager = Manager::<DeprecationPolicy>::new(&options);

        // act
        let policy = manager.get("test", &preview);

        // assert
        assert!(policy.is_some());
        assert!(manager.get("test", &release).is_none());
    }
}
