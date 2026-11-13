#![forbid(unsafe_code)]

use kodogu_runtime::{Initialization, Initializer, initialize_all};
use proptest::prelude::*;
use std::future::Future;
use std::io;
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll, Waker};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Effect {
    Started(u8),
    Released(u8),
}

struct Dependencies {
    seed: u64,
    effects: Arc<Mutex<Vec<Effect>>>,
}

impl Dependencies {
    fn new(seed: u64) -> Self {
        Self { seed, effects: Arc::new(Mutex::new(Vec::new())) }
    }
}

struct Instance {
    index: u8,
    value: u128,
    effects: Arc<Mutex<Vec<Effect>>>,
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.effects.lock().unwrap().push(Effect::Released(self.index));
    }
}

struct Fixture {
    index: u8,
    fails: bool,
    waits: bool,
}

impl Initializer for Fixture {
    type Context = Dependencies;
    type Instance = Instance;
    type Error = io::Error;
    type Future<'a> = Initialization<'a, Instance, io::Error>;

    fn initialize<'a>(
        &'a self,
        dependencies: &'a Dependencies,
    ) -> Initialization<'a, Instance, io::Error> {
        Box::pin(async move {
            dependencies.effects.lock().unwrap().push(Effect::Started(self.index));
            if self.fails {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("denied at {}", self.index),
                ));
            }
            if self.waits {
                std::future::pending::<()>().await;
            }
            Ok(Instance {
                index: self.index,
                value: u128::from(dependencies.seed) + u128::from(self.index),
                effects: Arc::clone(&dependencies.effects),
            })
        })
    }
}

fn fixtures(count: u8, failure: Option<u8>) -> Vec<Fixture> {
    (0..count)
        .map(|index| Fixture { index, fails: failure == Some(index), waits: false })
        .collect()
}

fn observations(instances: &[Instance]) -> Vec<(u8, u128)> {
    instances.iter().map(|instance| (instance.index, instance.value)).collect()
}

fn effects(dependencies: &Dependencies) -> (Vec<u8>, Vec<u8>) {
    let mut started = Vec::new();
    let mut released = Vec::new();
    for effect in dependencies.effects.lock().unwrap().iter() {
        match effect {
            Effect::Started(index) => started.push(*index),
            Effect::Released(index) => released.push(*index),
        }
    }
    // Drop ordering is not part of the initialization contract.
    released.sort_unstable();
    (started, released)
}

async fn check_case(count: u8, failure: Option<u8>) {
    let dependencies = Dependencies::new(100);
    let definitions = fixtures(count, failure);
    let outcome = initialize_all(&definitions, &dependencies).await;
    let actual = outcome
        .as_ref()
        .map(|instances| observations(instances))
        .map_err(|error| (error.kind(), error.to_string()));
    let expected = match failure {
        Some(index) => Err((io::ErrorKind::PermissionDenied, format!("denied at {index}"))),
        None => Ok((0..count).map(|index| (index, 100 + u128::from(index))).collect()),
    };
    assert_eq!(actual, expected);
    drop(outcome);
    let started = failure.map_or(count, |index| index + 1);
    let released = failure.unwrap_or(count);
    assert_eq!(effects(&dependencies), ((0..started).collect(), (0..released).collect()));
}

macro_rules! construction_cases {
    ($($name:ident: $count:expr, $failure:expr;)*) => {
        $(#[tokio::test]
        async fn $name() {
            check_case($count, $failure).await;
        })*
    };
}

// Count and valid failure position have ten valid combinations for counts zero through three.
construction_cases! {
    empty_construction_returns_no_instances: 0, None;
    one_dependency_context_is_resolved: 1, None;
    first_failure_produces_no_instance: 1, Some(0);
    both_initializers_receive_the_context: 2, None;
    failure_skips_the_next_initializer: 2, Some(0);
    second_failure_releases_the_first_instance: 2, Some(1);
    all_three_instances_are_returned: 3, None;
    first_failure_skips_both_remaining_initializers: 3, Some(0);
    middle_failure_releases_the_prefix_and_skips_the_tail: 3, Some(1);
    last_failure_releases_both_previous_instances: 3, Some(2);
}

#[test]
fn dropping_unpolled_construction_does_not_start_initializers() {
    let dependencies = Dependencies::new(100);
    let definitions = fixtures(2, None);
    let future = initialize_all(&definitions, &dependencies);
    drop(future);
    assert_eq!(effects(&dependencies), (vec![], vec![]));
}

#[test]
fn dropping_pending_construction_releases_completed_instances() {
    let dependencies = Dependencies::new(100);
    let definitions = vec![
        Fixture { index: 0, fails: false, waits: false },
        Fixture { index: 1, fails: false, waits: true },
        Fixture { index: 2, fails: false, waits: false },
    ];
    let mut future = Box::pin(initialize_all(&definitions, &dependencies));
    assert!(matches!(
        future.as_mut().poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Pending
    ));
    assert_eq!(effects(&dependencies), (vec![0, 1], vec![]));
    drop(future);
    assert_eq!(effects(&dependencies), (vec![0, 1], vec![0]));
}

#[tokio::test]
async fn contexts_do_not_share_resource_ownership() {
    let first = Dependencies::new(1);
    let second = Dependencies::new(2);
    let definitions = fixtures(1, None);
    let instances = initialize_all(&definitions, &first).await.unwrap();
    assert_eq!(observations(&instances), vec![(0, 1)]);
    assert_eq!(effects(&first), (vec![0], vec![]));
    assert_eq!(effects(&second), (vec![], vec![]));
    drop(instances);
    assert_eq!(effects(&first), (vec![0], vec![0]));
    assert_eq!(effects(&second), (vec![], vec![]));
}

struct Failing(Arc<io::Error>);

impl Initializer for Failing {
    type Context = ();
    type Instance = ();
    type Error = Arc<io::Error>;
    type Future<'a> = Initialization<'a, (), Self::Error>;

    fn initialize<'a>(&'a self, _: &'a ()) -> Initialization<'a, (), Self::Error> {
        Box::pin(async move { Err(Arc::clone(&self.0)) })
    }
}

#[tokio::test]
async fn original_typed_failure_is_preserved() {
    let original = Arc::new(io::Error::new(io::ErrorKind::PermissionDenied, "typed failure"));
    let definition = Failing(Arc::clone(&original));
    let error = initialize_all([&definition], &()).await.unwrap_err();
    assert!(Arc::ptr_eq(&original, &error));
    assert_eq!(
        (error.kind(), error.to_string()),
        (io::ErrorKind::PermissionDenied, "typed failure".into())
    );
}

proptest! {
    #[test]
    fn every_result_reflects_the_injected_context(seed in any::<u64>(), count in 0_u8..20) {
        let dependencies = Dependencies::new(seed);
        let definitions = fixtures(count, None);
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let instances = runtime.block_on(initialize_all(&definitions, &dependencies)).unwrap();
        let expected: Vec<_> = (0..count).map(|index| (index, u128::from(seed) + u128::from(index))).collect();
        prop_assert_eq!(observations(&instances), expected);
        drop(instances);
        let expected_indices: Vec<_> = (0..count).collect();
        prop_assert_eq!(effects(&dependencies), (expected_indices.clone(), expected_indices));
    }
}
