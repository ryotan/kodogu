#![forbid(unsafe_code)]

#[path = "support/registration/foundation.rs"]
mod foundation;
#[path = "support/registration/tools/mod.rs"]
mod tools;

#[cfg(test)]
mod tests {
    use super::foundation::{Host, Observation, Seed};
    use std::io;

    macro_rules! scenarios {
        ($($name:ident: $seed:expr => $first:expr, $second:expr;)*) => {
            $(#[tokio::test]
            async fn $name() {
                let host = Host::start(Ok(Seed($seed))).await.unwrap();
                let mut actual = host.invoke();
                actual.sort_by_key(|item| item.tool);
                assert_eq!(actual, vec![
                    Observation { tool: "first", value: $first },
                    Observation { tool: "second", value: $second },
                ]);
                assert_eq!(host.call_count(), 2);
            })*
        };
    }

    scenarios! {
        resolves_zero_input: 0 => 0, 0;
        resolves_first_context: 10 => 20, 30;
        resolves_changed_context: 20 => 40, 60;
    }

    #[tokio::test]
    async fn failed_context_construction_does_not_produce_a_host() {
        let failure = io::Error::new(io::ErrorKind::PermissionDenied, "fixture denied");
        let result = Host::start(Err(failure)).await;
        let error = match result {
            Ok(_) => panic!("incomplete initialization produced a host"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), "fixture denied");
    }

    #[tokio::test]
    async fn application_contexts_do_not_share_service_instances() {
        let first = Host::start(Ok(Seed(10))).await.unwrap();
        let second = Host::start(Ok(Seed(20))).await.unwrap();
        first.invoke();
        assert_eq!((first.call_count(), second.call_count()), (2, 0));
    }
}
