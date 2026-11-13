//! Execution identity and W3C `traceparent` propagation without a tracer provider or recording policy.

use std::future::Future;

use opentelemetry::Context;
use opentelemetry::context::FutureExt;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator};
use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{IdGenerator, RandomIdGenerator};

/// Immutable execution identity, independent of enabled recording spans.
///
/// This value retains no provider, baggage, vendor state or application resource. Incoming identities
/// describe the remote span; call [`Self::child`] to create the local request identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceContext {
    trace_id: TraceId,
    span_id: SpanId,
    parent_span_id: Option<SpanId>,
    flags: TraceFlags,
}

impl TraceContext {
    /// Starts a new trace without installing a tracer or making a recording decision.
    pub fn root() -> Self {
        Self {
            trace_id: random_trace_id(),
            span_id: random_span_id(None),
            parent_span_id: None,
            flags: TraceFlags::NOT_SAMPLED,
        }
    }

    /// Returns the identity bound by this module's synchronous or future scope.
    pub fn current() -> Option<Self> {
        Context::current().get::<Self>().copied()
    }

    /// Creates an execution in the same trace with this execution as its parent.
    pub fn child(self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: random_span_id(Some(self.span_id)),
            parent_span_id: Some(self.span_id),
            flags: self.flags,
        }
    }

    /// Returns the canonical lowercase hexadecimal trace identifier.
    pub fn trace_id(&self) -> String {
        self.trace_id.to_string()
    }

    /// Returns the canonical lowercase hexadecimal execution span identifier.
    pub fn span_id(&self) -> String {
        self.span_id.to_string()
    }

    /// Returns the known parent span identifier; incoming headers do not encode their own parent.
    pub fn parent_span_id(&self) -> Option<String> {
        self.parent_span_id.map(|id| id.to_string())
    }

    /// Returns supported W3C flags; these do not control local failure diagnostics.
    pub fn trace_flags(&self) -> u8 {
        self.flags.to_u8()
    }

    /// Decodes remote identity without inheriting the current OpenTelemetry context.
    ///
    /// Only `traceparent` participates; vendor `tracestate` and baggage are not retained or forwarded.
    pub fn from_traceparent(value: &str) -> Option<Self> {
        let value = known_header(value)?;
        let context =
            TraceContextPropagator::new().extract_with_context(&Context::new(), &Incoming(value));
        let span = context.span();
        let remote = span.span_context();
        remote.is_valid().then(|| Self {
            trace_id: remote.trace_id(),
            span_id: remote.span_id(),
            parent_span_id: None,
            flags: remote.trace_flags(),
        })
    }

    /// Encodes the known fields using the SDK's supported W3C version and flags.
    pub fn to_traceparent(self) -> String {
        let mut header = Outgoing::default();
        TraceContextPropagator::new().inject_context(&self.context(), &mut header);
        header.0
    }

    /// Binds identity only while the callback runs, restoring the previous context on return or panic.
    pub fn in_scope<T>(self, callback: impl FnOnce() -> T) -> T {
        let _guard = self.context().attach();
        callback()
    }

    /// Binds identity for each future poll, without a thread-local guard across suspension.
    pub fn in_future<F: Future>(self, future: F) -> impl Future<Output = F::Output> {
        future.with_context(self.context())
    }

    fn context(self) -> Context {
        let span =
            SpanContext::new(self.trace_id, self.span_id, self.flags, false, TraceState::default());
        Context::new().with_remote_span_context(span).with_value(self)
    }
}

fn known_header(value: &str) -> Option<&str> {
    let value = value.trim_matches([' ', '\t']);
    let bytes = value.as_bytes();
    if bytes.len() < 55 || bytes[2] != b'-' || bytes[35] != b'-' || bytes[52] != b'-' {
        return None;
    }
    // The SDK's radix conversion accepts short fields; W3C requires their exact widths and lowercase hex.
    let fields = [&bytes[..2], &bytes[3..35], &bytes[36..52], &bytes[53..55]];
    if !fields.into_iter().all(lower_hex)
        || (&bytes[..2] == b"00" && bytes.len() != 55)
        || (bytes.len() > 55 && bytes[55] != b'-')
    {
        return None;
    }
    // Future-version fields are opaque. Only the known prefix belongs in the current SDK decoder.
    value.get(..55)
}

fn lower_hex(bytes: &[u8]) -> bool {
    bytes.iter().all(|&byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn random_trace_id() -> TraceId {
    loop {
        let id = RandomIdGenerator::default().new_trace_id();
        if id != TraceId::INVALID {
            return id;
        }
    }
}

fn random_span_id(parent: Option<SpanId>) -> SpanId {
    loop {
        let id = RandomIdGenerator::default().new_span_id();
        if id != SpanId::INVALID && Some(id) != parent {
            return id;
        }
    }
}

struct Incoming<'a>(&'a str);

impl Extractor for Incoming<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        (key == "traceparent").then_some(self.0)
    }

    fn keys(&self) -> Vec<&str> {
        vec!["traceparent"]
    }
}

#[derive(Default)]
struct Outgoing(String);

impl Injector for Outgoing {
    fn set(&mut self, key: &str, value: String) {
        if key == "traceparent" {
            self.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct ForeignValue;

    #[test]
    fn scopes_do_not_inherit_arbitrary_context_values() {
        let original = Context::new().with_value(ForeignValue);
        let _guard = original.attach();
        TraceContext::root().in_scope(|| {
            assert_eq!(Context::current().get::<ForeignValue>(), None);
            assert_eq!(
                Context::current().span().span_context().trace_state(),
                &TraceState::default()
            );
        });
        assert_eq!(Context::current().get::<ForeignValue>(), Some(&ForeignValue));
        assert_eq!(TraceContext::current(), None);
    }

    #[test]
    fn invalid_headers_do_not_reuse_an_existing_context() {
        TraceContext::root().in_scope(|| {
            assert_eq!(TraceContext::from_traceparent("invalid"), None);
        });
    }
}
