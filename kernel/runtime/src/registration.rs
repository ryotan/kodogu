//! Validated tool identities and application-owned registration.

use std::collections::{BTreeMap, btree_map};
use std::fmt;
use suzunari_error::{ensure, suzunari_error};

/// A stable tool namespace, kept separate from transport routes and storage paths.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ToolId(String);

#[suzunari_error]
#[suzu(display("Invalid tool identifier {value:?}; expected lower-kebab-case"))]
pub struct InvalidToolId {
    pub value: String,
}

impl TryFrom<String> for ToolId {
    type Error = InvalidToolId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure!(valid_identifier(&value), InvalidToolIdSnafu { value });
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ToolId {
    type Error = InvalidToolId;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl AsRef<str> for ToolId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_ref())
    }
}

fn valid_identifier(value: &str) -> bool {
    value.split('-').all(|segment| {
        !segment.is_empty()
            && segment.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

/// Application-owned records or instances; registration neither constructs nor resolves dependencies.
#[derive(Debug)]
pub struct ToolRegistry<T> {
    entries: BTreeMap<ToolId, T>,
}

#[suzunari_error]
#[suzu(display("Duplicate tool identifier {id}"))]
pub struct DuplicateToolId {
    pub id: ToolId,
}

impl<T> ToolRegistry<T> {
    /// Validate uniqueness before the application invokes any registered factory.
    pub fn try_from_iter(
        entries: impl IntoIterator<Item = (ToolId, T)>,
    ) -> Result<Self, DuplicateToolId> {
        let mut registered = BTreeMap::new();
        for (id, payload) in entries {
            ensure!(!registered.contains_key(&id), DuplicateToolIdSnafu { id });
            registered.insert(id, payload);
        }
        Ok(Self { entries: registered })
    }

    pub fn get(&self, id: &ToolId) -> Option<&T> {
        self.entries.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ToolId, &T)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> IntoIterator for ToolRegistry<T> {
    type Item = (ToolId, T);
    type IntoIter = btree_map::IntoIter<ToolId, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[cfg(test)]
mod tests;
