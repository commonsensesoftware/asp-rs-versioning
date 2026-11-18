use super::{LinkList, Policy};
use crate::{ApiVersion, policy::LinkBuilder};
use http::{Uri, uri::InvalidUri};
use std::time::SystemTime;

/// Represents an API versioning policy builder.
#[derive(Clone, Debug)]
pub struct Builder<T> {
    name: Option<String>,
    api_version: Option<ApiVersion<'static>>,
    as_of: Option<SystemTime>,
    policy: Option<T>,
    links: LinkList,
}

impl<T> Builder<T> {
    pub(super) fn for_rel(name: Option<String>, api_version: Option<ApiVersion<'static>>, rel: &'static str) -> Self {
        Self {
            name,
            api_version,
            as_of: None,
            policy: None,
            links: LinkList::new(rel),
        }
    }

    /// Gets the associated API name, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Gets the associated [API version](ApiVersion), if any.
    pub fn api_version(&self) -> Option<&ApiVersion<'_>> {
        self.api_version.as_ref()
    }

    /// Sets the effective policy date and time.
    ///
    /// # Arguments
    ///
    /// * `as_of` - the policy effective [date and time](SystemTime)
    pub fn as_of(&mut self, date: SystemTime) -> &mut Self {
        self.as_of = Some(date);
        self
    }

    /// Configures the builder per the specified policy.
    ///
    /// # Arguments
    ///
    /// * `per` - the applied policy
    pub fn per(&mut self, policy: T) -> &mut Self {
        self.policy = Some(policy);
        self
    }

    /// Gets or creates a new link builder.
    ///
    /// # Arguments
    ///
    /// * `target` - the target [URL](Uri)
    ///
    /// # Panics
    ///
    /// This function panics if the the specified `target` [URL](Uri) is invalid.
    pub fn link(&mut self, target: impl TryInto<Uri, Error = InvalidUri>) -> LinkBuilder<'_> {
        self.links.get_or_add(target)
    }
}

impl<T> Builder<T>
where
    T: Default + From<SystemTime> + Policy,
{
    /// Builds and returns a new policy.
    pub fn build(self) -> T {
        if let Some(policy) = self.policy {
            return policy;
        }

        let mut policy = if let Some(date) = self.as_of {
            T::from(date)
        } else {
            T::default()
        };

        for link in self.links {
            policy.add_link(link);
        }

        policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::DeprecationPolicy;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn new_builder_should_have_no_name_or_api_version() {
        // arrange

        // act
        let builder = Builder::<DeprecationPolicy>::new(None, None);

        // assert
        assert_eq!(builder.name(), None);
        assert_eq!(builder.api_version(), None);
    }

    #[test]
    fn build_should_return_default_policy_when_unconfigured() {
        // arrange
        let builder = Builder::<DeprecationPolicy>::new(Some("test".into()), Some(ApiVersion::DEFAULT));

        // act
        let policy = builder.build();

        // assert
        assert_eq!(policy.date(), None);
        assert!(policy.links().is_empty());
    }

    #[test]
    fn build_should_use_as_of_date() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut builder = Builder::<DeprecationPolicy>::new(None, None);

        builder.as_of(date);

        // act
        let policy = builder.build();

        // assert
        assert_eq!(policy.date(), Some(date));
    }

    #[test]
    fn as_of_should_replace_previous_date() {
        // arrange
        let first = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let second = UNIX_EPOCH + Duration::from_secs(1_800_000_000);
        let mut builder = Builder::<DeprecationPolicy>::new(None, None);

        builder.as_of(first);

        // act
        builder.as_of(second);

        // assert
        assert_eq!(builder.build().date(), Some(second));
    }

    #[test]
    fn build_should_add_links_in_order() {
        // arrange
        let mut builder = Builder::<DeprecationPolicy>::new(None, None);

        builder.link("https://example.com/first");
        builder.link("https://example.com/second");

        // act
        let policy = builder.build();

        // assert
        let targets: Vec<_> = policy.links().iter().map(|link| link.target().to_string()).collect();

        assert_eq!(targets, ["https://example.com/first", "https://example.com/second"]);
    }

    #[test]
    fn link_should_return_existing_link_for_same_target() {
        // arrange
        let mut builder = Builder::<DeprecationPolicy>::new(None, None);

        builder.link("https://example.com/policy").title("Policy");

        // act
        builder.link("https://example.com/policy").media_type("text/html");

        // assert
        let policy = builder.build();

        assert_eq!(policy.links().len(), 1);
        assert_eq!(policy.links()[0].title, "Policy");
        assert_eq!(policy.links()[0].media_type, "text/html");
    }

    #[test]
    #[should_panic]
    fn link_should_panic_for_invalid_target() {
        // arrange
        let mut builder = Builder::<DeprecationPolicy>::new(None, None);

        // act
        builder.link("not a url");

        // assert
    }

    #[test]
    fn build_should_prefer_explicit_policy() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let other = UNIX_EPOCH + Duration::from_secs(1_800_000_000);
        let mut builder = Builder::<DeprecationPolicy>::new(None, None);

        builder.as_of(other).per(DeprecationPolicy::as_of(date));
        builder.link("https://example.com/policy");

        // act
        let policy = builder.build();

        // assert
        assert_eq!(policy.date(), Some(date));
        assert!(policy.links().is_empty());
    }
}
