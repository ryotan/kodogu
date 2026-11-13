#![forbid(unsafe_code)]

use kodogu_runtime::{Initialization, Initializer, SendInitialization, initialize_all};
use std::{
    cell::Cell,
    convert::Infallible,
    future::{Ready, ready},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

struct Local;

impl Initializer for Local {
    type Context = Rc<Cell<u32>>;
    type Instance = Rc<Cell<u32>>;
    type Error = Infallible;
    type Future<'a> = Initialization<'a, Self::Instance, Infallible>;

    fn initialize<'a>(
        &'a self,
        counter: &'a Self::Context,
    ) -> Initialization<'a, Self::Instance, Infallible> {
        Box::pin(async move {
            counter.set(counter.get() + 1);
            tokio::task::yield_now().await;
            counter.set(counter.get() + 1);
            Ok(Rc::clone(counter))
        })
    }
}

#[tokio::test]
async fn local_construction_accepts_non_send_dependencies() {
    let counter = Rc::new(Cell::new(0));
    let definitions = [Local, Local];
    let values = initialize_all(&definitions, &counter).await.unwrap();
    assert_eq!(counter.get(), 4);
    assert!(values.iter().all(|value| Rc::ptr_eq(value, &counter)));
    assert_eq!(Rc::strong_count(&counter), 3);
    drop(values);
    assert_eq!(Rc::strong_count(&counter), 1);
}

struct Portable;

impl Initializer for Portable {
    type Context = AtomicU32;
    type Instance = u32;
    type Error = Infallible;
    type Future<'a> = SendInitialization<'a, u32, Infallible>;

    fn initialize<'a>(&'a self, counter: &'a AtomicU32) -> Self::Future<'a> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            Ok(counter.fetch_add(1, Ordering::SeqCst))
        })
    }
}

#[tokio::test]
async fn whole_construction_can_move_to_a_task_when_its_inputs_and_futures_allow_it() {
    let counter = Arc::new(AtomicU32::new(0));
    let task_counter = Arc::clone(&counter);
    let task = tokio::spawn(async move {
        let definitions = [Portable, Portable];
        initialize_all(&definitions, task_counter.as_ref()).await.unwrap()
    });
    assert_eq!(task.await.unwrap(), [0, 1]);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

struct Immediate;

impl Initializer for Immediate {
    type Context = u32;
    type Instance = u32;
    type Error = Infallible;
    type Future<'a> = Ready<Result<u32, Infallible>>;

    fn initialize<'a>(&'a self, value: &'a u32) -> Self::Future<'a> {
        ready(Ok(*value))
    }
}

#[tokio::test]
async fn synchronous_constructor_needs_no_boxed_future() {
    assert_eq!(initialize_all([&Immediate], &42).await.unwrap(), [42]);
}
