//! Owned message identity and safe arguments, rendered independently of failure meaning.

use crate::ToolId;
use std::{collections::BTreeMap, sync::Arc};

mod catalog;
mod locale;

pub use catalog::{Catalog, CatalogError, ResourceIssues};
pub use locale::{InvalidLocale, Locale, Preferences};

/// Application subsystems and registered tools have distinct catalog namespaces.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Namespace {
    Application(String),
    Tool(ToolId),
}

/// Only arguments deliberately approved for public presentation belong in a message.
#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    Text(String),
    /// Integers outside ±(2^53-1) select the safe descriptor fallback.
    Signed(i64),
    /// Integers above 2^53-1 select the safe descriptor fallback.
    Unsigned(u64),
    /// Nonfinite values select the descriptor's safe fallback instead of formatter output.
    Decimal(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    data: Arc<MessageData>,
}

#[derive(Clone, Debug, PartialEq)]
enum MessageData {
    Localized {
        namespace: Namespace,
        id: String,
        arguments: BTreeMap<String, Argument>,
        fallback_english: String,
    },
    /// Already rendered by another owner; its language is unknown and its text is preserved.
    Opaque { text: String },
}

impl Message {
    /// Retain an identity and literal English fallback until a presentation boundary selects a locale.
    pub fn localized(
        namespace: Namespace,
        id: impl Into<String>,
        fallback_english: impl Into<String>,
    ) -> Self {
        Self::localized_with_arguments(namespace, id, fallback_english, [])
    }

    /// Construct a localizable message with the values approved for presentation.
    pub fn localized_with_arguments(
        namespace: Namespace,
        id: impl Into<String>,
        fallback_english: impl Into<String>,
        arguments: impl IntoIterator<Item = (String, Argument)>,
    ) -> Self {
        Self {
            data: Arc::new(MessageData::Localized {
                namespace,
                id: id.into(),
                arguments: arguments.into_iter().collect(),
                fallback_english: fallback_english.into(),
            }),
        }
    }

    /// Preserve text already rendered by another owner without inferring its language or identity.
    ///
    /// ```compile_fail,E0599
    /// use kodogu_runtime::messages::{Argument, Message};
    /// let remote = Message::opaque("Already rendered");
    /// remote.with_argument("count", Argument::Unsigned(1));
    /// ```
    pub fn opaque(text: impl Into<String>) -> Self {
        Self { data: Arc::new(MessageData::Opaque { text: text.into() }) }
    }
}

/// Resources are owned data; their lifetime is not restricted to built-in static files.
#[derive(Clone, Debug)]
pub struct CatalogResource {
    pub namespace: Namespace,
    pub locale: Locale,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedMessage {
    pub text: String,
    /// The actual catalog/fallback language, or None for opaque pre-rendered text.
    pub locale: Option<Locale>,
}
