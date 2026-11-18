use super::{SelectedVersion, Selector, specific};
use crate::{ApiVersion, Model, RequestInfo, prelude::ApiVersionStatusExt};
use std::future::ready;

/// Represents a selector that selects the lowest implemented [API version](ApiVersion).
#[derive(Clone, Debug)]
pub struct LowestVersionSelector(ApiVersion<'static>);

impl LowestVersionSelector {
    /// Initializes a new [LowestVersionSelector].
    ///
    /// # Arguments
    ///
    /// * `fallback` - the [API version](ApiVersion) selected when there is no implemented, stable API version
    ///
    /// # Panics
    ///
    /// Panics if the fallback API version is [neutral](ApiVersion::NEUTRAL).
    #[track_caller]
    pub fn new(fallback: ApiVersion<'static>) -> Self {
        Self(specific(fallback))
    }
}

impl Selector for LowestVersionSelector {
    fn select<'a>(&'a self, _request: &'a (dyn RequestInfo + Sync), model: &'a Model) -> SelectedVersion<'a> {
        Box::pin(ready(Some(match model.implemented().len() {
            1 if model.implemented()[0].status().is_none() => model.implemented()[0].to_owned(),
            0 | 1 => self.0.clone(),
            _ => model
                .implemented()
                .iter()
                .filter(|v| v.status().is_none())
                .min_by(|a, b| a.total_cmp(b))
                .map(ApiVersion::to_owned)
                .unwrap_or_else(|| self.0.clone()),
        })))
    }

    fn clone(&self) -> Box<dyn Selector> {
        Box::new(Clone::clone(self))
    }
}
