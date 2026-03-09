use std::fmt::Debug;
use std::marker::PhantomData;

use redb::{TypeName, Value};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A redb [`Value`] adapter that serializes `T` as MessagePack using [`rmp_serde`].
///
/// The map format (`to_vec_named`) is used so that [`serde(default)`] works correctly
/// when fields are added or removed — see ADR-0004.
#[derive(Debug)]
pub(crate) struct MessagePack<T>(PhantomData<T>);

impl<T: Debug + Serialize + DeserializeOwned + Send + Sync + 'static> Value for MessagePack<T> {
    type SelfType<'a>
        = T
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> T
    where
        Self: 'a,
    {
        rmp_serde::from_slice(data).expect("redb contains invalid MessagePack data")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8>
    where
        Self: 'a + 'b,
    {
        rmp_serde::to_vec_named(value).expect("MessagePack serialization failed")
    }

    fn type_name() -> TypeName {
        TypeName::new("MessagePack")
    }
}
