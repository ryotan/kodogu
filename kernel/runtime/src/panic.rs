//! Application-installed panic reporting without exposing the panic payload to its reporter.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

/// Source metadata available to an application's panic reporter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanicLocation<'a> {
    pub file: &'a str,
    pub line: u32,
    pub column: u32,
}

/// Replace the process hook at the application entry point, without chaining the previous hook.
///
/// The reporter must not panic. It chooses whether and where to report this metadata; caught panic
/// payloads remain owned by their existing unwind boundary. This does not recover aborting panics.
pub fn install_hook(reporter: impl for<'a> Fn(Option<PanicLocation<'a>>) + Send + Sync + 'static) {
    std::panic::set_hook(Box::new(move |information| {
        reporter(information.location().map(|location| PanicLocation {
            file: location.file(),
            line: location.line(),
            column: location.column(),
        }));
    }));
}

/// Dispose of a caught unwind's payload at an application's recovery boundary.
///
/// A panic during disposal with a plain string payload can be contained. An opaque secondary
/// payload resumes unwinding so the enclosing application boundary can handle it.
pub fn dispose_caught_payload(payload: Box<dyn Any + Send>) {
    if let Err(secondary) = catch_unwind(AssertUnwindSafe(|| drop(payload)))
        && !secondary.is::<String>()
        && !secondary.is::<&'static str>()
    {
        resume_unwind(secondary);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::panic_any;

    struct Secondary;

    struct PanicsOnDrop;

    impl Drop for PanicsOnDrop {
        fn drop(&mut self) {
            panic_any(Secondary);
        }
    }

    #[test]
    fn an_opaque_secondary_payload_reaches_the_application_boundary() {
        let result = catch_unwind(|| dispose_caught_payload(Box::new(PanicsOnDrop)));

        let payload = result.unwrap_err();
        assert!(payload.is::<Secondary>());
    }
}
