//! Shared application and tool contracts, independent of concrete applications and execution policy.

#![forbid(unsafe_code)]

pub mod initialization;
pub mod lifecycle;
pub mod messages;
pub mod panic;
pub mod registration;
pub mod trace_context;

pub use initialization::{
    Initialization, Initializer, SendInitialization, collect, initialize_all, registered, submit,
};
pub use lifecycle::{Shutdown, shutdown_stream};
pub use registration::{DuplicateToolId, InvalidToolId, ToolId, ToolRegistry};
