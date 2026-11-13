//! Completion results for application-owned asynchronous shutdown.

use crate::registration::ToolId;
use futures::{Stream, stream::FuturesUnordered};
use std::future::Future;

/// Consumes an owned tool or resource using its own error and future types.
///
/// A local implementation can borrow application data without requiring thread transfer or boxing.
pub trait Shutdown {
    type Error;
    type Future: Future<Output = Result<(), Self::Error>>;

    fn shutdown(self) -> Self::Future;
}

/// Start each shutdown when polled and report results in completion order.
///
/// An error is one tool's result; it does not discard the remaining shutdowns. The caller owns
/// deadlines and result collection. Dropping the stream drops uncompleted tools and futures, without
/// promising asynchronous cleanup or termination of tasks represented by detached join handles.
pub fn shutdown_stream<T: Shutdown>(
    tools: impl IntoIterator<Item = (ToolId, T)>,
) -> impl Stream<Item = (ToolId, Result<(), T::Error>)> {
    tools
        .into_iter()
        .map(|(id, tool)| async move { (id, tool.shutdown().await) })
        .collect::<FuturesUnordered<_>>()
}
