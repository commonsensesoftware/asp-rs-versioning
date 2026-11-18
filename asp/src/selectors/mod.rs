mod constant;
mod current;
mod lowest;

pub use constant::ConstantSelector;
pub use current::CurrentVersionSelector;
pub use lowest::LowestVersionSelector;

use crate::{ApiVersion, Model, RequestInfo};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

/// Represents the asynchronous result of [selecting](Selector::select) an API version.
pub type SelectedVersion<'a> = Pin<Box<dyn Future<Output = Option<ApiVersion<'a>>> + Send + 'a>>;

/// Defines the behavior of an API version selector.
pub trait Selector: Debug + Send + Sync {
    /// Selects an API version given the specified HTTP request and API version information.
    ///
    /// # Arguments
    ///
    /// * `request` - the current HTTP [request information](RequestInfo)
    /// * `model` - the [model](Model) to select the version from
    ///
    /// # Returns
    ///
    /// An asynchronous future that returns the selected [API version](ApiVersion), if any.
    fn select<'a>(&'a self, request: &'a (dyn RequestInfo + Sync), model: &'a Model) -> SelectedVersion<'a>;

    /// Clones the current selector.
    fn clone(&self) -> Box<dyn Selector>;
}

impl Clone for Box<dyn Selector> {
    fn clone(&self) -> Self {
        (**self).clone()
    }
}

/// Ensures the specified API version is a specific API version that a selector can select.
#[track_caller]
fn specific(version: ApiVersion<'static>) -> ApiVersion<'static> {
    assert!(
        !version.is_neutral(),
        "A selector cannot select ApiVersion::NEUTRAL; it must select a specific API version."
    );
    version
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::api_version;
    use http::Request;
    use rstest::rstest;
    use std::task::{Context, Poll, Waker};

    /// Selects an API version using a selector that completes without waiting.
    pub(crate) fn select(selector: &dyn Selector, model: &Model) -> Option<ApiVersion<'static>> {
        let request = Request::new(());
        let mut future = selector.select(&request, model);
        let mut context = Context::from_waker(Waker::noop());

        match future.as_mut().poll(&mut context) {
            Poll::Ready(version) => version.map(|version| version.to_owned()),
            Poll::Pending => panic!("the selector was expected to complete immediately"),
        }
    }

    fn model(versions: &[ApiVersion<'static>]) -> Model {
        Model::builder().add_supported(versions.to_vec()).build()
    }

    #[test]
    fn constant_selector_should_ignore_model() {
        // arrange
        let selector = ConstantSelector::new(api_version!(42.0));
        let model = model(&[api_version!(1.0), api_version!(2.0)]);

        // act
        let version = select(&selector, &model);

        // assert
        assert_eq!(version, Some(api_version!(42.0)));
    }

    #[rstest]
    #[case(&[api_version!(1.0)], api_version!(1.0))]
    #[case(&[api_version!(1.0), api_version!(2.0)], api_version!(2.0))]
    #[case(&[api_version!(1.0), api_version!(2.0), api_version!(3.0-beta)], api_version!(2.0))]
    #[case(&[], api_version!(42.0))]
    #[case(&[api_version!(3.0-beta)], api_version!(42.0))]
    #[case(&[api_version!(2.0-beta), api_version!(3.0-beta)], api_version!(42.0))]
    fn current_version_selector_should_select_highest_stable_version(
        #[case] versions: &[ApiVersion<'static>],
        #[case] expected: ApiVersion<'static>,
    ) {
        // arrange
        let selector = CurrentVersionSelector::new(api_version!(42.0));
        let model = model(versions);

        // act
        let version = select(&selector, &model);

        // assert
        assert_eq!(version, Some(expected));
    }

    #[rstest]
    #[case(&[api_version!(1.0)], api_version!(1.0))]
    #[case(&[api_version!(1.0), api_version!(2.0)], api_version!(1.0))]
    #[case(&[api_version!(1.0-beta), api_version!(2.0), api_version!(3.0)], api_version!(2.0))]
    #[case(&[], api_version!(42.0))]
    #[case(&[api_version!(3.0-beta)], api_version!(42.0))]
    #[case(&[api_version!(2.0-beta), api_version!(3.0-beta)], api_version!(42.0))]
    fn lowest_version_selector_should_select_lowest_stable_version(
        #[case] versions: &[ApiVersion<'static>],
        #[case] expected: ApiVersion<'static>,
    ) {
        // arrange
        let selector = LowestVersionSelector::new(api_version!(42.0));
        let model = model(versions);

        // act
        let version = select(&selector, &model);

        // assert
        assert_eq!(version, Some(expected));
    }

    #[test]
    #[should_panic(expected = "A selector cannot select ApiVersion::NEUTRAL; it must select a specific API version.")]
    fn constant_selector_should_panic_when_version_is_neutral() {
        // arrange
        let version = ApiVersion::NEUTRAL;

        // act
        ConstantSelector::new(version);

        // assert
    }

    #[test]
    #[should_panic(expected = "A selector cannot select ApiVersion::NEUTRAL; it must select a specific API version.")]
    fn current_version_selector_should_panic_when_fallback_is_neutral() {
        // arrange
        let fallback = ApiVersion::NEUTRAL;

        // act
        CurrentVersionSelector::new(fallback);

        // assert
    }

    #[test]
    #[should_panic(expected = "A selector cannot select ApiVersion::NEUTRAL; it must select a specific API version.")]
    fn lowest_version_selector_should_panic_when_fallback_is_neutral() {
        // arrange
        let fallback = ApiVersion::NEUTRAL;

        // act
        LowestVersionSelector::new(fallback);

        // assert
    }

    #[test]
    fn boxed_selector_should_clone() {
        // arrange
        let selector: Box<dyn Selector> = Box::new(ConstantSelector::new(api_version!(42.0)));
        let model = Model::empty();

        // act
        let clone = selector.clone();

        // assert
        assert_eq!(select(&*clone, &model), Some(api_version!(42.0)));
    }
}
