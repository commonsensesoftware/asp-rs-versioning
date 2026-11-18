use crate::ApiVersion;
use serde::de::{Deserialize, Deserializer, Error};
use std::cell::RefCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

thread_local! {
    static CURRENT: RefCell<Option<ApiVersion<'static>>> = const { RefCell::new(None) };
}

/// Swaps the current API version with the specified API version when created and swaps it back when dropped.
struct Swap<'a>(&'a mut Option<ApiVersion<'static>>);

impl<'a> Swap<'a> {
    fn new(version: &'a mut Option<ApiVersion<'static>>) -> Self {
        CURRENT.with(|current| mem::swap(&mut *current.borrow_mut(), version));
        Self(version)
    }
}

impl Drop for Swap<'_> {
    fn drop(&mut self) {
        CURRENT.with(|current| mem::swap(&mut *current.borrow_mut(), self.0));
    }
}

/// Invokes the specified function with the current API version.
///
/// # Arguments
///
/// * `f` - the function to invoke with the current [API version](ApiVersion), if any
///
/// # Remarks
///
/// The function must not call [scope] or poll a [Scoped] future.
pub fn with_current<R>(f: impl FnOnce(Option<&ApiVersion<'static>>) -> R) -> R {
    CURRENT.with(|current| f(current.borrow().as_ref()))
}

/// Invokes the specified function with the specified API version as the current API version.
///
/// # Arguments
///
/// * `version` - the [API version](ApiVersion) that is current while the function runs, if any
/// * `f` - the function to invoke
///
/// # Remarks
///
/// The previous API version is restored when the function returns, even if it panics.
pub fn scope<R>(version: Option<ApiVersion<'static>>, f: impl FnOnce() -> R) -> R {
    let mut version = version;
    let _swap = Swap::new(&mut version);
    f()
}

/// Determines whether a member is visible in the current API version.
///
/// # Arguments
///
/// * `contains` - the function that determines whether the member is visible in an [API version](ApiVersion)
///
/// # Remarks
///
/// A member is always visible when there is no current API version.
pub fn is_visible(contains: impl FnOnce(&ApiVersion<'static>) -> bool) -> bool {
    with_current(|version| version.is_none_or(contains))
}

/// Deserializes a member when it is visible in the current API version; otherwise, rejects it.
///
/// # Arguments
///
/// * `deserializer` - the [deserializer](Deserializer) to read the member from
/// * `contains` - the function that determines whether the member is visible in an [API version](ApiVersion)
///
/// # Remarks
///
/// A member that is not visible in the requested API version is not part of its contract. Accepting it would allow
/// over-posting, so a hidden member is an error, even when its value is the default value.
pub fn deserialize<'de, D, T>(
    deserializer: D,
    contains: impl FnOnce(&ApiVersion<'static>) -> bool,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let hidden = with_current(|version| version.filter(|version| !contains(version)).map(ToString::to_string));

    match hidden {
        Some(version) => Err(D::Error::custom(format_args!("not supported in API version {version}"))),
        None => T::deserialize(deserializer),
    }
}

/// Represents a future that has a current API version each time it is polled.
///
/// # Remarks
///
/// Work that happens outside of a poll of the future, such as a spawned task or a response body that is serialized
/// after the future completes, does not have the API version.
#[derive(Debug)]
pub struct Scoped<F> {
    version: Option<ApiVersion<'static>>,
    future: F,
}

impl<F> Scoped<F> {
    /// Initializes a new [Scoped] future.
    ///
    /// # Arguments
    ///
    /// * `version` - the [API version](ApiVersion) that is current while the future is polled, if any
    /// * `future` - the future to poll
    pub fn new(version: Option<ApiVersion<'static>>, future: F) -> Self {
        Self { version, future }
    }
}

impl<F: Future> Future for Scoped<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: `future` is structurally pinned. it is never moved out of `self`, no other method exposes it, and
        // `Scoped` does not implement `Drop`. `version` is not structurally pinned, so it can be moved freely.
        let (version, future) = unsafe {
            let this = self.get_unchecked_mut();
            (&mut this.version, Pin::new_unchecked(&mut this.future))
        };
        let _swap = Swap::new(version);
        future.poll(cx)
    }
}

#[cfg(test)]
mod tests;
