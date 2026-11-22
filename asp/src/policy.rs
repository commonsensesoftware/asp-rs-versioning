mod builder;
mod key;
mod link;
mod manager;
mod policies;
mod target;

pub use builder::Builder;
pub use key::Key;
pub use link::{LinkBuilder, LinkList};
pub use manager::Manager;
pub use policies::Policies;
pub use target::Target;

use crate::ApiVersion;
use crate::headers::Link;
use std::fmt::Debug;
use std::time::SystemTime;

/// Defines the behavior of an API versioning policy.
pub trait Policy: Clone + Debug + Default + Sized {
    /// Adds the specified link to the policy.
    ///
    /// # Arguments
    ///
    /// * `link` - the associated [link](Link)
    fn add_link(&mut self, link: Link);
}

macro_rules! policy {
    ($name:ident, $rel:literal) => {
        #[doc = concat!("Represents a ", $rel, " policy.")]
        #[derive(Clone, Debug, Default)]
        pub struct $name {
            date: Option<SystemTime>,
            links: Vec<Link>,
        }

        impl $name {
            #[doc = concat!("Initializes a new [`", stringify!($name), "`].")]
            ///
            /// # Arguments
            ///
            /// * `link` - the associated policy [link](Link)
            pub fn link(link: Link) -> Self {
                Self {
                    date: None,
                    links: vec![link],
                }
            }

            #[doc = concat!("Initializes a new [`", stringify!($name), "`].")]
            ///
            /// # Arguments
            ///
            /// * `date` - the policy effective [date and time](SystemTime)
            pub fn as_of(date: impl Into<SystemTime>) -> Self {
                Self {
                    date: Some(date.into()),
                    links: Vec::new(),
                }
            }

            #[doc = concat!("Initializes a new [`", stringify!($name), "`].")]
            ///
            /// # Arguments
            ///
            /// * `date` - the policy effective [date and time](SystemTime)
            /// * `link` - the associated policy [link](Link)
            pub fn new(date: SystemTime, link: Link) -> Self {
                Self {
                    date: Some(date),
                    links: vec![link],
                }
            }

            /// Gets the associated policy [links](Link).
            #[inline]
            pub fn links(&self) -> &[Link] {
                &self.links
            }

            /// Gets the policy effective [date and time](SystemTime), if any.
            #[inline]
            pub fn date(&self) -> Option<SystemTime> {
                self.date
            }

            /// Returns a value indicating if the policy is in effect for the specified date and time.
            ///
            /// # Arguments
            ///
            /// * `date` - the [date and time](SystemTime) to evaluate
            pub fn in_effect(&self, date: SystemTime) -> bool {
                if let Some(effective) = self.date {
                    date >= effective
                } else {
                    true
                }
            }
        }

        impl From<SystemTime> for $name {
            fn from(date: SystemTime) -> Self {
                Self::as_of(date)
            }
        }

        impl Policy for $name {
            fn add_link(&mut self, link: Link) {
                self.links.push(link);
            }
        }

        impl Builder<$name> {
            #[doc = concat!("Initializes a new [`", stringify!($name), "`] [builder](Builder).")]
            ///
            /// # Arguments
            ///
            /// * `name` - the optional policy name
            /// * `api_version` - the optional [API version](ApiVersion) the policy applies to
            pub fn new(name: Option<String>, api_version: Option<ApiVersion<'_>>) -> Self {
                Self::for_rel(name, api_version.map(|v| v.to_owned()), $rel)
            }
        }
    };
}

policy!(SunsetPolicy, "sunset");
policy!(DeprecationPolicy, "deprecation");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version;
    use std::time::{Duration, UNIX_EPOCH};

    macro_rules! policy_tests {
        ($module:ident, $name:ident, $rel:literal) => {
            mod $module {
                use super::*;

                fn link() -> Link {
                    Link::new("https://example.com/policy".parse().unwrap(), $rel)
                }

                #[test]
                fn default_should_have_no_date_or_links() {
                    // arrange

                    // act
                    let policy = $name::default();

                    // assert
                    assert_eq!(policy.date(), None);
                    assert!(policy.links().is_empty());
                }

                #[test]
                fn link_should_initialize_policy_without_date() {
                    // arrange
                    let link = link();

                    // act
                    let policy = $name::link(link.clone());

                    // assert
                    assert_eq!(policy.date(), None);
                    assert_eq!(policy.links(), &[link]);
                }

                #[test]
                fn as_of_should_initialize_policy_without_links() {
                    // arrange
                    let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);

                    // act
                    let policy = $name::as_of(date);

                    // assert
                    assert_eq!(policy.date(), Some(date));
                    assert!(policy.links().is_empty());
                }

                #[test]
                fn new_should_initialize_policy_with_date_and_link() {
                    // arrange
                    let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
                    let link = link();

                    // act
                    let policy = $name::new(date, link.clone());

                    // assert
                    assert_eq!(policy.date(), Some(date));
                    assert_eq!(policy.links(), &[link]);
                }

                #[test]
                fn from_should_initialize_policy_with_date() {
                    // arrange
                    let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);

                    // act
                    let policy = $name::from(date);

                    // assert
                    assert_eq!(policy.date(), Some(date));
                    assert!(policy.links().is_empty());
                }

                #[test]
                fn add_link_should_append_link() {
                    // arrange
                    let first = link();
                    let second = Link::new("https://example.com/other".parse().unwrap(), $rel);
                    let mut policy = $name::link(first.clone());

                    // act
                    policy.add_link(second.clone());

                    // assert
                    assert_eq!(policy.links(), &[first, second]);
                }

                #[test]
                fn builder_should_initialize_name_and_api_version() {
                    // arrange
                    let api_version = api_version!(1.0);

                    // act
                    let builder = Builder::<$name>::new(Some("test".into()), Some(api_version.clone()));

                    // assert
                    assert_eq!(builder.name(), Some("test"));
                    assert_eq!(builder.api_version(), Some(&api_version));
                }

                #[test]
                fn builder_should_create_links_with_policy_relation_type() {
                    // arrange
                    let mut builder = Builder::<$name>::new(None, None);

                    builder.link("https://example.com/policy");

                    // act
                    let policy = builder.build();

                    // assert
                    assert_eq!(policy.links().len(), 1);
                    assert_eq!(policy.links()[0].rel(), $rel);
                }
            }
        };
    }

    policy_tests!(deprecation, DeprecationPolicy, "deprecation");
    policy_tests!(sunset, SunsetPolicy, "sunset");
}
