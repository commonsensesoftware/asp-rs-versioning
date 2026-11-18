use super::{Scoped, deserialize, is_visible, scope, with_current};
use crate::{ApiVersion, api_version, test::Result};
use rstest::rstest;
use serde_json::Value;
use std::future::{Future, poll_fn};
use std::pin::pin;
use std::task::{Context, Poll, Waker};

fn current() -> Option<ApiVersion<'static>> {
    with_current(|version| version.cloned())
}

fn contains(version: &ApiVersion<'static>) -> bool {
    (api_version!(2.0)..).contains(version)
}

#[rstest]
#[case(None, true)]
#[case(Some(api_version!(1.0)), false)]
#[case(Some(api_version!(2.0)), true)]
#[case(Some(api_version!(2026-10-01)), false)]
fn is_visible_should_evaluate_current_version(#[case] version: Option<ApiVersion<'static>>, #[case] expected: bool) {
    // arrange

    // act
    let visible = scope(version, || is_visible(contains));

    // assert
    assert_eq!(visible, expected);
}

#[rstest]
#[case(None)]
#[case(Some(api_version!(2.0)))]
fn deserialize_should_read_visible_member(#[case] version: Option<ApiVersion<'static>>) -> Result {
    // arrange
    let value = Value::from("Bill");

    // act
    let member: String = scope(version, || deserialize(value, contains))?;

    // assert
    assert_eq!(member, "Bill");
    Ok(())
}

#[rstest]
#[case(Value::from("Bill"))]
#[case(Value::Null)]
fn deserialize_should_reject_hidden_member(#[case] value: Value) {
    // arrange
    let version = Some(api_version!(1.0));

    // act
    let result = scope(version, || deserialize::<_, Option<String>>(value, contains));

    // assert
    assert_eq!(result.unwrap_err().to_string(), "not supported in API version 1.0");
}

#[test]
fn scope_should_restore_previous_version() {
    // arrange
    let outer = Some(api_version!(1.0));
    let inner = Some(api_version!(2.0));

    // act
    let (during, after) = scope(outer.clone(), || (scope(inner.clone(), current), current()));

    // assert
    assert_eq!(during, inner);
    assert_eq!(after, outer);
    assert_eq!(current(), None);
}

#[test]
fn scope_should_restore_previous_version_when_function_panics() {
    // arrange
    let version = Some(api_version!(1.0));

    // act
    let result = std::panic::catch_unwind(|| scope(version, || panic!("expected")));

    // assert
    assert!(result.is_err());
    assert_eq!(current(), None);
}

#[test]
fn scoped_future_should_have_version_only_while_polled() {
    // arrange
    let mut seen = Vec::new();
    let mut future = pin!(Scoped::new(
        Some(api_version!(2.0)),
        poll_fn(|_| {
            seen.push(current());
            if seen.len() == 1 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
    ));
    let mut context = Context::from_waker(Waker::noop());

    // act
    let first = future.as_mut().poll(&mut context);
    let between = current();
    let second = future.as_mut().poll(&mut context);

    // assert
    assert_eq!(first, Poll::Pending);
    assert_eq!(between, None);
    assert_eq!(second, Poll::Ready(()));
    assert_eq!(seen, [Some(api_version!(2.0)), Some(api_version!(2.0))]);
}

#[test]
fn scoped_future_should_support_future_that_is_not_unpin() {
    // arrange
    let mut future = pin!(Scoped::new(Some(api_version!(1.0)), async {
        std::future::ready(()).await;
        current()
    }));
    let mut context = Context::from_waker(Waker::noop());

    // act
    let poll = future.as_mut().poll(&mut context);

    // assert
    assert_eq!(poll, Poll::Ready(Some(api_version!(1.0))));
}
