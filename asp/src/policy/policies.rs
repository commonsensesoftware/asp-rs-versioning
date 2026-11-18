use super::{
    Builder, DeprecationPolicy, Key, SunsetPolicy,
    Target::{self, *},
};
use crate::policy::Policy;
use std::collections::HashMap;
use std::time::SystemTime;

mod sealed {
    use super::*;

    /// Filters the policy builders of a specific policy type.
    pub trait Type: Sized + 'static {
        fn of(policies: &Policies) -> &HashMap<Key, Builder<Self>>;
    }

    impl Type for DeprecationPolicy {
        #[inline]
        fn of(policies: &Policies) -> &HashMap<Key, Builder<Self>> {
            &policies.deprecation
        }
    }

    impl Type for SunsetPolicy {
        #[inline]
        fn of(policies: &Policies) -> &HashMap<Key, Builder<Self>> {
            &policies.sunset
        }
    }
}

/// Represents the supported API versioning policies.
#[derive(Clone, Debug, Default)]
pub struct Policies {
    deprecation: HashMap<Key, Builder<DeprecationPolicy>>,
    sunset: HashMap<Key, Builder<SunsetPolicy>>,
}

impl Policies {
    /// Creates and returns a new deprecation policy builder for the target API and/or version.
    ///
    /// # Arguments
    ///
    /// * `target` - the policy [target](Target)
    ///
    /// # Remarks
    ///
    /// Use [Target::Specific] or one of the tuple conversion variants to target a specific API and version combination.
    pub fn deprecate(&mut self, target: impl Into<Target>) -> &mut Builder<DeprecationPolicy> {
        match target.into() {
            Api(api) => self
                .deprecation
                .entry(Key::from(api.clone()))
                .or_insert_with(|| Builder::<DeprecationPolicy>::new(Some(api), None)),
            Version(api_version) => self
                .deprecation
                .entry(Key::from(api_version.clone()))
                .or_insert_with(|| Builder::<DeprecationPolicy>::new(None, Some(api_version))),
            Specific { api, api_version } => self
                .deprecation
                .entry(Key::new(api.clone(), api_version.clone()))
                .or_insert_with(|| Builder::<DeprecationPolicy>::new(Some(api), Some(api_version))),
        }
    }

    /// Creates and returns a new sunset policy builder for the target API and/or version.
    ///
    /// # Arguments
    ///
    /// * `target` - the policy [target](Target)
    ///
    /// # Remarks
    ///
    /// Use [Target::Specific] or one of the tuple conversion variants to target a specific API and version combination.
    pub fn sunset(&mut self, target: impl Into<Target>) -> &mut Builder<SunsetPolicy> {
        match target.into() {
            Api(api) => self
                .sunset
                .entry(Key::from(api.clone()))
                .or_insert_with(|| Builder::<SunsetPolicy>::new(Some(api), None)),
            Version(api_version) => self
                .sunset
                .entry(Key::from(api_version.clone()))
                .or_insert_with(|| Builder::<SunsetPolicy>::new(None, Some(api_version))),
            Specific { api, api_version } => self
                .sunset
                .entry(Key::new(api.clone(), api_version.clone()))
                .or_insert_with(|| Builder::<SunsetPolicy>::new(Some(api), Some(api_version))),
        }
    }

    /// Enumerates the [policy builders](Builder) of the specified type.
    pub fn of<T>(&self) -> impl Iterator<Item = Builder<T>> + '_
    where
        T: From<SystemTime> + Policy + sealed::Type,
    {
        T::of(self).values().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiVersion;
    use std::time::{Duration, UNIX_EPOCH};

    const V1: ApiVersion<'static> = ApiVersion::parse_literal("1.0");
    const V2: ApiVersion<'static> = ApiVersion::parse_literal("2.0");

    #[test]
    fn of_should_be_empty_by_default() {
        // arrange
        let policies = Policies::default();

        // act
        let deprecations = policies.of::<DeprecationPolicy>().count();
        let sunsets = policies.of::<SunsetPolicy>().count();

        // assert
        assert_eq!(deprecations, 0);
        assert_eq!(sunsets, 0);
    }

    #[test]
    fn deprecate_should_create_builder_for_api() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.deprecate("test");

        // assert
        let builders: Vec<_> = policies.of::<DeprecationPolicy>().collect();

        assert_eq!(builders.len(), 1);
        assert_eq!(builders[0].name(), Some("test"));
        assert_eq!(builders[0].api_version(), None);
    }

    #[test]
    fn deprecate_should_create_builder_for_api_version() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.deprecate(&V1);

        // assert
        let builders: Vec<_> = policies.of::<DeprecationPolicy>().collect();

        assert_eq!(builders.len(), 1);
        assert_eq!(builders[0].name(), None);
        assert_eq!(builders[0].api_version(), Some(&V1));
    }

    #[test]
    fn deprecate_should_create_builder_for_specific_api_and_version() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.deprecate(("test", &V1));

        // assert
        let builders: Vec<_> = policies.of::<DeprecationPolicy>().collect();

        assert_eq!(builders.len(), 1);
        assert_eq!(builders[0].name(), Some("test"));
        assert_eq!(builders[0].api_version(), Some(&V1));
    }

    #[test]
    fn sunset_should_create_builder_for_api() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.sunset("test");

        // assert
        let builders: Vec<_> = policies.of::<SunsetPolicy>().collect();

        assert_eq!(builders.len(), 1);
        assert_eq!(builders[0].name(), Some("test"));
        assert_eq!(builders[0].api_version(), None);
    }

    #[test]
    fn sunset_should_create_builder_for_api_version() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.sunset(&V1);

        // assert
        let builders: Vec<_> = policies.of::<SunsetPolicy>().collect();

        assert_eq!(builders.len(), 1);
        assert_eq!(builders[0].name(), None);
        assert_eq!(builders[0].api_version(), Some(&V1));
    }

    #[test]
    fn sunset_should_create_builder_for_specific_api_and_version() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.sunset(("test", &V1));

        // assert
        let builders: Vec<_> = policies.of::<SunsetPolicy>().collect();

        assert_eq!(builders.len(), 1);
        assert_eq!(builders[0].name(), Some("test"));
        assert_eq!(builders[0].api_version(), Some(&V1));
    }

    #[test]
    fn deprecate_should_return_existing_builder_for_same_target() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut policies = Policies::default();

        policies.deprecate(("test", &V1)).as_of(date);

        // act
        policies
            .deprecate(("test", &V1))
            .link("https://example.com/deprecation");

        // assert
        let results: Vec<_> = policies.of::<DeprecationPolicy>().map(Builder::build).collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].date(), Some(date));
        assert_eq!(results[0].links().len(), 1);
    }

    #[test]
    fn sunset_should_return_existing_builder_for_same_target() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut policies = Policies::default();

        policies.sunset(("test", &V1)).as_of(date);

        // act
        policies.sunset(("test", &V1)).link("https://example.com/sunset");

        // assert
        let results: Vec<_> = policies.of::<SunsetPolicy>().map(Builder::build).collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].date(), Some(date));
        assert_eq!(results[0].links().len(), 1);
    }

    #[test]
    fn deprecate_should_create_distinct_builders_for_distinct_targets() {
        // arrange
        let mut policies = Policies::default();

        // act
        policies.deprecate("test");
        policies.deprecate(&V1);
        policies.deprecate(("test", &V1));
        policies.deprecate(("test", &V2));

        // assert
        assert_eq!(policies.of::<DeprecationPolicy>().count(), 4);
    }

    #[test]
    fn of_should_only_return_builders_of_requested_type() {
        // arrange
        let mut policies = Policies::default();

        policies.deprecate(("test", &V1));
        policies.deprecate(("test", &V2));
        policies.sunset(("test", &V1));

        // act
        let deprecations = policies.of::<DeprecationPolicy>().count();
        let sunsets = policies.of::<SunsetPolicy>().count();

        // assert
        assert_eq!(deprecations, 2);
        assert_eq!(sunsets, 1);
    }

    #[test]
    fn of_should_build_deprecation_policy() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut policies = Policies::default();

        policies
            .deprecate(("test", &V1))
            .as_of(date)
            .link("https://example.com/deprecation");

        // act
        let results: Vec<_> = policies.of::<DeprecationPolicy>().map(Builder::build).collect();

        // assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].date(), Some(date));
        assert_eq!(results[0].links().len(), 1);
        assert_eq!(results[0].links()[0].target(), "https://example.com/deprecation");
        assert_eq!(results[0].links()[0].rel(), "deprecation");
    }

    #[test]
    fn of_should_build_sunset_policy() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut policies = Policies::default();

        policies
            .sunset(("test", &V1))
            .as_of(date)
            .link("https://example.com/sunset");

        // act
        let results: Vec<_> = policies.of::<SunsetPolicy>().map(Builder::build).collect();

        // assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].date(), Some(date));
        assert_eq!(results[0].links().len(), 1);
        assert_eq!(results[0].links()[0].target(), "https://example.com/sunset");
        assert_eq!(results[0].links()[0].rel(), "sunset");
    }

    #[test]
    fn of_should_not_consume_builders() {
        // arrange
        let mut policies = Policies::default();

        policies.sunset("test").link("https://example.com/sunset");

        let first = policies.of::<SunsetPolicy>().count();

        // act
        let second: Vec<_> = policies.of::<SunsetPolicy>().map(Builder::build).collect();

        // assert
        assert_eq!(first, 1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].links().len(), 1);
    }
}
