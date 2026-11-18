use super::{SelectedVersion, Selector, specific};
use crate::{ApiVersion, Model, RequestInfo};
use std::future::ready;

/// Represents a selector that always returns a constant [API version](ApiVersion).
///
/// # Remarks
///
/// This selector returns the [default API version](crate::Options::default_version) unless another
/// [selector](crate::Options::selector) is configured.
#[derive(Clone, Debug)]
pub struct ConstantSelector(ApiVersion<'static>);

impl ConstantSelector {
    /// Initializes a new [ConstantSelector].
    ///
    /// # Arguments
    ///
    /// * `version` - the constant [API version](ApiVersion) to always return
    ///
    /// # Panics
    ///
    /// Panics if the API version is [neutral](ApiVersion::NEUTRAL).
    #[track_caller]
    pub fn new(version: ApiVersion<'static>) -> Self {
        Self(specific(version))
    }
}

impl Selector for ConstantSelector {
    fn select<'a>(&'a self, _request: &'a (dyn RequestInfo + Sync), _model: &'a Model) -> SelectedVersion<'a> {
        Box::pin(ready(Some(self.0.clone())))
    }

    fn clone(&self) -> Box<dyn Selector> {
        Box::new(Clone::clone(self))
    }
}
