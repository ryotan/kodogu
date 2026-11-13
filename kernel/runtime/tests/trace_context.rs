use std::cell::Cell;
use std::future::{Future, poll_fn};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use futures::FutureExt;
use kodogu_runtime::trace_context::TraceContext;
use proptest::prelude::*;

const TRACE_ID: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
const SPAN_ID: &str = "00f067aa0ba902b7";
const SAMPLED: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

#[test]
fn roots_have_valid_identity_without_installing_a_tracer() {
    let root = TraceContext::root();
    assert_eq!(root.trace_id().len(), 32);
    assert_eq!(root.span_id().len(), 16);
    assert_ne!(root.trace_id(), "0".repeat(32));
    assert_ne!(root.span_id(), "0".repeat(16));
    assert_eq!(root.parent_span_id(), None);
    assert_eq!(TraceContext::current(), None);
    assert_eq!(TraceContext::from_traceparent(&root.to_traceparent()), Some(root));
}

macro_rules! decoded_headers {
    ($($name:ident: $header:expr => $flags:expr),+ $(,)?) => {$ (
        #[test]
        fn $name() {
            let identity = TraceContext::from_traceparent($header).unwrap();
            assert_eq!(identity.trace_id(), TRACE_ID);
            assert_eq!(identity.span_id(), SPAN_ID);
            assert_eq!(identity.parent_span_id(), None);
            assert_eq!(identity.trace_flags(), $flags);
            assert_eq!(identity.to_traceparent(), format!("00-{TRACE_ID}-{SPAN_ID}-{:02x}", $flags));
        }
    )+};
}

decoded_headers! {
    unsampled_headers_preserve_identity: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00" => 0,
    sampled_headers_preserve_identity: SAMPLED => 1,
    unknown_flags_are_cleared: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-02" => 0,
    sampling_is_preserved_with_unknown_flags: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-03" => 1,
    reserved_flags_are_cleared: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-04" => 0,
    known_flags_survive_reserved_flags: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-ff" => 1,
    future_versions_use_the_known_fields: "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01" => 1,
    future_extensions_are_not_interpreted: "fe-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-vendor_data" => 1,
    future_empty_extensions_do_not_change_known_fields: "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-" => 1,
    surrounding_http_whitespace_is_accepted: " \t00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01\t " => 1,
}

macro_rules! rejected_headers {
    ($($name:ident: $header:expr),+ $(,)?) => {$ (
        #[test]
        fn $name() {
            assert_eq!(TraceContext::from_traceparent($header), None);
        }
    )+};
}

rejected_headers! {
    empty_headers_are_rejected: "",
    one_digit_versions_are_rejected: "0-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    one_digit_flags_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1",
    oversized_versions_are_rejected: "000-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    oversized_flags_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-001",
    forbidden_versions_are_rejected: "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    zero_trace_ids_are_rejected: "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
    zero_span_ids_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
    short_trace_ids_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e473-00f067aa0ba902b7-01",
    short_span_ids_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b-01",
    uppercase_trace_ids_are_rejected: "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
    uppercase_span_ids_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00F067AA0BA902B7-01",
    uppercase_versions_are_rejected: "0A-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    uppercase_flags_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0F",
    version_zero_extensions_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra",
    version_zero_trailing_delimiters_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-",
    future_extensions_require_a_delimiter: "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01extra",
    joined_headers_are_rejected: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01,00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00",
    line_breaks_are_rejected: "\n00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01\n",
    unicode_whitespace_is_rejected: "\u{a0}00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
}

#[test]
fn children_keep_the_trace_and_record_the_parent_identity() {
    let parent = TraceContext::from_traceparent(SAMPLED).unwrap();
    let child = parent.child();
    assert_eq!(child.trace_id(), parent.trace_id());
    assert_ne!(child.span_id(), parent.span_id());
    assert_eq!(child.parent_span_id(), Some(parent.span_id()));
    assert_eq!(child.trace_flags(), parent.trace_flags());
    assert_eq!(TraceContext::current(), None);
}

#[test]
fn nested_scopes_restore_the_previous_identity() {
    let parent = TraceContext::root();
    let child = parent.child();
    parent.in_scope(|| {
        assert_eq!(TraceContext::current(), Some(parent));
        child.in_scope(|| assert_eq!(TraceContext::current(), Some(child)));
        assert_eq!(TraceContext::current(), Some(parent));
    });
    assert_eq!(TraceContext::current(), None);
}

#[test]
fn synchronous_scopes_accept_borrowed_results() {
    let value = String::from("borrowed");
    let result = TraceContext::root().in_scope(|| value.as_str());
    assert_eq!(result, "borrowed");
}

#[test]
fn panicking_scopes_restore_the_previous_identity() {
    let parent = TraceContext::root();
    parent.in_scope(|| {
        let failure = catch_unwind(|| parent.child().in_scope(|| panic!("scope failure")));
        assert!(failure.is_err());
        assert_eq!(TraceContext::current(), Some(parent));
    });
    assert_eq!(TraceContext::current(), None);
}

#[tokio::test]
async fn asynchronous_scopes_bind_identity_for_each_poll() {
    let identity = TraceContext::root();
    let result = identity
        .in_future(async {
            assert_eq!(TraceContext::current(), Some(identity));
            tokio::task::yield_now().await;
            TraceContext::current()
        })
        .await;
    assert_eq!(result, Some(identity));
    assert_eq!(TraceContext::current(), None);
}

#[tokio::test]
async fn concurrent_futures_keep_their_own_identity() {
    let first = TraceContext::root();
    let second = TraceContext::root();
    let observe = async |identity: TraceContext| {
        assert_eq!(TraceContext::current(), Some(identity));
        tokio::task::yield_now().await;
        TraceContext::current()
    };
    let result = futures::join!(first.in_future(observe(first)), second.in_future(observe(second)));
    assert_eq!(result, (Some(first), Some(second)));
    assert_eq!(TraceContext::current(), None);
}

#[tokio::test]
async fn asynchronous_scopes_accept_borrowed_non_send_futures() {
    let value = Rc::new(Cell::new(0));
    let borrowed = &value;
    TraceContext::root()
        .in_future(async {
            tokio::task::yield_now().await;
            borrowed.set(42);
        })
        .await;
    assert_eq!(value.get(), 42);
    assert_eq!(TraceContext::current(), None);
}

#[test]
fn pending_futures_restore_the_outer_scope_before_cancellation() {
    let outer = TraceContext::root();
    let inner = outer.child();
    let mut future = Box::pin(inner.in_future(poll_fn(|_| {
        assert_eq!(TraceContext::current(), Some(inner));
        Poll::<()>::Pending
    })));
    outer.in_scope(|| {
        let mut task = Context::from_waker(Waker::noop());
        assert_eq!(future.as_mut().poll(&mut task), Poll::Pending);
        assert_eq!(TraceContext::current(), Some(outer));
        drop(future);
        assert_eq!(TraceContext::current(), Some(outer));
    });
    assert_eq!(TraceContext::current(), None);
}

#[tokio::test]
async fn panicking_futures_restore_the_outer_identity() {
    let identity = TraceContext::root();
    let result = AssertUnwindSafe(identity.in_future(async {
        tokio::task::yield_now().await;
        assert_eq!(TraceContext::current(), Some(identity));
        panic!("future failure");
    }))
    .catch_unwind()
    .await;
    assert!(result.is_err());
    assert_eq!(TraceContext::current(), None);
}

proptest! {
    #[test]
    fn valid_identity_round_trips_without_backend_types(
        trace in 1u128..=u128::MAX,
        span in 1u64..=u64::MAX,
        flags in 0u8..=1,
    ) {
        let header = format!("00-{trace:032x}-{span:016x}-{flags:02x}");
        let identity = TraceContext::from_traceparent(&header).unwrap();
        prop_assert_eq!(identity.trace_id(), format!("{trace:032x}"));
        prop_assert_eq!(identity.span_id(), format!("{span:016x}"));
        prop_assert_eq!(identity.trace_flags(), flags);
        prop_assert_eq!(identity.parent_span_id(), None);
        prop_assert_eq!(identity.to_traceparent(), header);
        let child = identity.child();
        prop_assert_eq!(child.trace_id(), identity.trace_id());
        prop_assert_eq!(child.parent_span_id(), Some(identity.span_id()));
        prop_assert_ne!(child.span_id(), identity.span_id());
        prop_assert_eq!(child.trace_flags(), identity.trace_flags());
    }

    #[test]
    fn arbitrary_headers_are_rejected_or_canonicalized_without_panicking(header in any::<String>()) {
        if let Some(identity) = TraceContext::from_traceparent(&header) {
            let canonical = identity.to_traceparent();
            prop_assert_eq!(canonical.len(), 55);
            prop_assert!(canonical.is_ascii());
            prop_assert_eq!(TraceContext::from_traceparent(&canonical), Some(identity));
        }
    }
}
