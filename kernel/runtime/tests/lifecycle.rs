use futures::{FutureExt, StreamExt, channel::oneshot, future::LocalBoxFuture};
use kodogu_runtime::{
    ToolId,
    lifecycle::{Shutdown, shutdown_stream},
};
use std::{
    cell::Cell,
    collections::BTreeMap,
    future::{Ready, ready},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

fn id(value: &str) -> ToolId {
    ToolId::try_from(value).unwrap()
}

struct Immediate {
    calls: Rc<Cell<usize>>,
    result: Result<(), &'static str>,
}

impl Shutdown for Immediate {
    type Error = &'static str;
    type Future = Ready<Result<(), Self::Error>>;

    fn shutdown(self) -> Self::Future {
        self.calls.set(self.calls.get() + 1);
        ready(self.result)
    }
}

fn immediate(calls: &Rc<Cell<usize>>, result: Result<(), &'static str>) -> Immediate {
    Immediate { calls: Rc::clone(calls), result }
}

#[tokio::test]
async fn an_empty_registry_has_no_shutdown_results() {
    let stream = shutdown_stream(Vec::<(ToolId, Immediate)>::new());
    assert_eq!(stream.collect::<Vec<_>>().await, []);
}

#[test]
fn creating_a_stream_does_not_invoke_shutdown() {
    let calls = Rc::new(Cell::new(0));
    let stream = shutdown_stream([(id("tool"), immediate(&calls, Ok(())))]);
    assert_eq!(calls.get(), 0);
    drop(stream);
    assert_eq!(calls.get(), 0);
}

#[tokio::test]
async fn every_owned_tool_reports_exactly_one_shutdown_result() {
    let calls = Rc::new(Cell::new(0));
    let tools = [
        (id("first"), immediate(&calls, Ok(()))),
        (id("second"), immediate(&calls, Ok(()))),
        (id("third"), immediate(&calls, Ok(()))),
    ];
    let results = shutdown_stream(tools).collect::<Vec<_>>().await;
    assert_eq!(calls.get(), 3);
    assert_eq!(results.len(), 3);
    assert_eq!(
        results.into_iter().collect::<BTreeMap<_, _>>(),
        BTreeMap::from([(id("first"), Ok(())), (id("second"), Ok(())), (id("third"), Ok(())),])
    );
}

#[tokio::test]
async fn one_shutdown_failure_does_not_suppress_other_results() {
    let calls = Rc::new(Cell::new(0));
    let tools = [
        (id("failed"), immediate(&calls, Err("release failed"))),
        (id("successful"), immediate(&calls, Ok(()))),
    ];
    let results = shutdown_stream(tools).collect::<BTreeMap<_, _>>().await;
    assert_eq!(calls.get(), 2);
    assert_eq!(
        results,
        BTreeMap::from([(id("failed"), Err("release failed")), (id("successful"), Ok(())),])
    );
}

struct DropProbe<'a>(&'a Cell<usize>);

impl Drop for DropProbe<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

struct Controlled<'a> {
    started: &'a Cell<usize>,
    finished: &'a Cell<usize>,
    dropped: DropProbe<'a>,
    complete: oneshot::Receiver<Result<(), &'static str>>,
}

impl<'a> Shutdown for Controlled<'a> {
    type Error = &'static str;
    type Future = LocalBoxFuture<'a, Result<(), Self::Error>>;

    fn shutdown(self) -> Self::Future {
        self.started.set(self.started.get() + 1);
        async move {
            let _owned = self.dropped;
            let result = self.complete.await.unwrap();
            self.finished.set(self.finished.get() + 1);
            result
        }
        .boxed_local()
    }
}

#[tokio::test]
async fn completed_tools_report_without_waiting_for_an_earlier_pending_tool() {
    let (started, finished, dropped) = (Cell::new(0), Cell::new(0), Cell::new(0));
    let (first, first_done) = oneshot::channel();
    let (second, second_done) = oneshot::channel();
    let tool = |complete| Controlled {
        started: &started,
        finished: &finished,
        dropped: DropProbe(&dropped),
        complete,
    };
    let stream =
        shutdown_stream([(id("first"), tool(first_done)), (id("second"), tool(second_done))]);
    futures::pin_mut!(stream);
    second.send(Ok(())).unwrap();
    assert_eq!(stream.next().await, Some((id("second"), Ok(()))));
    first.send(Err("first failed")).unwrap();
    assert_eq!(stream.next().await, Some((id("first"), Err("first failed"))));
    assert_eq!(stream.next().await, None);
    assert_eq!((started.get(), finished.get(), dropped.get()), (2, 2, 2));
}

#[tokio::test]
async fn dropping_a_pending_stream_releases_owned_values_without_finishing_async_shutdown() {
    let (started, finished, dropped) = (Cell::new(0), Cell::new(0), Cell::new(0));
    let (_complete, receiver) = oneshot::channel();
    let tool = Controlled {
        started: &started,
        finished: &finished,
        dropped: DropProbe(&dropped),
        complete: receiver,
    };
    let mut stream = Box::pin(shutdown_stream([(id("tool"), tool)]));
    assert!(futures::poll!(stream.next()).is_pending());
    assert_eq!((started.get(), finished.get(), dropped.get()), (1, 0, 0));
    drop(stream);
    assert_eq!((started.get(), finished.get(), dropped.get()), (1, 0, 1));
}

#[test]
fn dropping_an_unpolled_stream_releases_the_owned_tool() {
    let (started, finished, dropped) = (Cell::new(0), Cell::new(0), Cell::new(0));
    let (_complete, receiver) = oneshot::channel();
    let tool = Controlled {
        started: &started,
        finished: &finished,
        dropped: DropProbe(&dropped),
        complete: receiver,
    };
    let stream = shutdown_stream([(id("tool"), tool)]);
    drop(stream);
    assert_eq!((started.get(), finished.get(), dropped.get()), (0, 0, 1));
}

struct Borrowed<'a>(&'a str);

impl<'a> Shutdown for Borrowed<'a> {
    type Error = &'a str;
    type Future = Ready<Result<(), Self::Error>>;

    fn shutdown(self) -> Self::Future {
        ready(Err(self.0))
    }
}

#[tokio::test]
async fn shutdown_can_borrow_application_local_values_and_error_data() {
    let detail = String::from("local detail");
    let stream = shutdown_stream([(id("tool"), Borrowed(&detail))]);
    assert_eq!(stream.collect::<Vec<_>>().await, [(id("tool"), Err(detail.as_str()))]);
}

struct SendTool(Arc<AtomicUsize>);

impl Shutdown for SendTool {
    type Error = &'static str;
    type Future = Ready<Result<(), Self::Error>>;

    fn shutdown(self) -> Self::Future {
        self.0.fetch_add(1, Ordering::SeqCst);
        ready(Ok(()))
    }
}

#[tokio::test]
async fn a_send_shutdown_stream_can_be_owned_by_a_spawned_task() {
    let calls = Arc::new(AtomicUsize::new(0));
    let stream = shutdown_stream([(id("tool"), SendTool(Arc::clone(&calls)))]);
    let results = tokio::spawn(async move { stream.collect::<Vec<_>>().await }).await.unwrap();
    assert_eq!(results, [(id("tool"), Ok(()))]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
