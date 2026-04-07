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

// `Debug` bound: required because SelfType<'a> = T and redb's Value trait
// requires both `Value: Debug` (supertrait) and `SelfType<'a>: Debug`.
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
        // RATIONALE: redb::Value trait requires infallible conversion — Result is not supported.
        // Deserialization failure here means the database contains corrupted or incompatible data,
        // which is unrecoverable at this layer. Panic is intentional.
        rmp_serde::from_slice(data).expect("redb contains invalid MessagePack data")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8>
    where
        Self: 'a + 'b,
    {
        // RATIONALE: redb::Value trait requires infallible conversion — Result is not supported.
        // Serialization of a well-typed Rust value should never fail; panic here would indicate
        // a serde implementation bug.
        rmp_serde::to_vec_named(value).expect("MessagePack serialization failed")
    }

    fn type_name() -> TypeName {
        // Include the concrete type name so redb can distinguish MessagePack<A> from MessagePack<B>
        // and reject mismatched table accesses at runtime.
        TypeName::new(&format!("MessagePack<{}>", std::any::type_name::<T>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redb::Value;

    #[test]
    fn roundtrip_preserves_data() {
        let original = vec![1u32, 2, 3];
        let bytes = MessagePack::<Vec<u32>>::as_bytes(&original);
        let decoded = MessagePack::<Vec<u32>>::from_bytes(&bytes);
        assert_eq!(decoded, original);
    }

    #[test]
    fn type_name_contains_concrete_type() {
        let name = MessagePack::<Vec<u32>>::type_name();
        assert!(
            name.name().contains("Vec<u32>"),
            "type_name should contain concrete type: {:?}",
            name.name()
        );
    }

    #[test]
    fn type_name_differs_for_different_types() {
        let name_u32 = MessagePack::<u32>::type_name();
        let name_string = MessagePack::<String>::type_name();
        assert_ne!(name_u32.name(), name_string.name());
    }

    #[test]
    fn type_name_is_idempotent() {
        let first = MessagePack::<u64>::type_name();
        let second = MessagePack::<u64>::type_name();
        assert_eq!(first.name(), second.name());
    }

    #[test]
    fn fixed_width_returns_none() {
        assert!(MessagePack::<Vec<u8>>::fixed_width().is_none());
    }

    #[test]
    #[should_panic(expected = "redb contains invalid MessagePack data")]
    fn from_bytes_panics_on_corrupted_data() {
        let garbage = &[0xFF, 0xFE, 0xFD];
        MessagePack::<Vec<u32>>::from_bytes(garbage);
    }
}
