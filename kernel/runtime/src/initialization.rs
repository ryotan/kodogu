//! Definition collection and explicit typed initialization shared by KoDoGu applications.
//!
//! Applications define their contexts and initializer records. Tools implement their own factories
//! against those contracts. Collection supplies available definitions; it does not select application dependencies,
//! resolve a dependency graph, or determine a lifecycle policy.

#![forbid(unsafe_code)]

use std::future::Future;
use std::pin::Pin;

/// Declare a collection beside the application-owned record type.
pub use inventory::collect;
/// Submit a definition beside the implementation that supplies it.
///
/// A definition retains its factory's declared dependency context:
///
/// ```
/// use kodogu_runtime::{collect, registered, submit};
///
/// struct Context;
/// struct Definition {
///     assemble: fn(&Context),
/// }
/// collect!(Definition);
///
/// fn compatible(_: &Context) {}
/// submit!(Definition { assemble: compatible });
///
/// let definitions: Vec<_> = registered::<Definition>().collect();
/// assert_eq!(definitions.len(), 1);
/// (definitions[0].assemble)(&Context);
/// ```
///
/// The same registration rejects a factory with an incompatible context:
///
/// ```compile_fail,E0308
/// use kodogu_runtime::{collect, submit};
///
/// struct Context;
/// struct Definition {
///     assemble: fn(&Context),
/// }
/// collect!(Definition);
///
/// fn incompatible(_: &()) {}
/// submit!(Definition { assemble: incompatible });
/// ```
pub use inventory::submit;

/// A fallible asynchronous construction borrowing its definition and dependencies.
pub type Initialization<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>;

/// A boxed construction that can be moved between threads when the caller requires it.
pub type SendInitialization<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>;

/// A construction contract whose dependency context, result and failure remain explicit types.
///
/// Implementations acquire only their declared dependencies. They must not resolve other tools
/// from a global service locator. The application owns the context and returned instances.
/// The future type preserves each implementation's execution requirements; local execution does
/// not require thread transfer. Implementations may use a boxed local/send future or a concrete one.
pub trait Initializer {
    type Context: ?Sized;
    type Instance;
    type Error;
    type Future<'a>: Future<Output = Result<Self::Instance, Self::Error>> + 'a
    where
        Self: 'a,
        Self::Context: 'a;

    fn initialize<'a>(&'a self, context: &'a Self::Context) -> Self::Future<'a>;
}

/// Return linked definitions without promising an iteration order.
///
/// Registering a definition does not execute application initialization. Only the caller chooses
/// when to construct resources, supplies the context, and retains the returned instances.
pub fn registered<T: inventory::Collect>() -> impl Iterator<Item = &'static T> {
    inventory::iter::<T>.into_iter()
}

/// Construct instances sequentially in the supplied iterator's order.
///
/// On failure, return the original typed error without starting later initializers. Owned instances
/// from the completed prefix are dropped. Dropping this future also drops its owned partial results.
/// Neither case promises asynchronous shutdown, reversal of external effects, or a drop order.
/// If the iterator comes directly from a distributed collection, its order remains unspecified.
/// The context may be local even when registered definitions have static storage.
pub async fn initialize_all<'a, D: Initializer + 'a>(
    definitions: impl IntoIterator<Item = &'a D>,
    context: &D::Context,
) -> Result<Vec<D::Instance>, D::Error> {
    let mut instances = Vec::new();
    for definition in definitions {
        instances.push(definition.initialize(context).await?);
    }
    Ok(instances)
}
