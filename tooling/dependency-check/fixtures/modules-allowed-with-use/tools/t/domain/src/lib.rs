//! A domain crate.
#![forbid(unsafe_code)]

pub mod domain;
/// The use cases.
pub mod use_cases;

pub use domain::{rule as domain_rule};
