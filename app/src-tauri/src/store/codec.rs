//! Typed MessagePack records for redb, translated outside redb's infallible Value callbacks.

use redb::{TypeName, Value};
use serde::{Serialize, de::DeserializeOwned};
use std::{borrow::Cow, fmt, io::Cursor, marker::PhantomData};
use suzunari_error::*;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum CodecError {
    #[suzu(display("Failed to encode stored MessagePack data"))]
    Encode { source: rmp_serde::encode::Error },
    #[suzu(display("Failed to decode stored MessagePack data"))]
    Decode { source: rmp_serde::decode::Error },
    #[suzu(display("Stored record has trailing data after byte {consumed} of {length}"))]
    TrailingData { consumed: u64, length: usize },
}

/// A feature's stored record: its frozen on-disk identity and the data encoded into it.
pub(crate) trait Record: DeserializeOwned + 'static {
    /// redb persists this name; changing it makes the record's existing tables unreadable.
    const TYPE_NAME: &'static str;
    /// What `MessagePack::encode` accepts for this record; it may borrow the feature's data.
    type Encode<'a>: Serialize;
}

/// The MessagePack bytes of one `R` record, stored as a redb table value without Serde work.
pub(crate) struct MessagePack<'a, R>(Cow<'a, [u8]>, PhantomData<fn() -> R>);

impl<R: Record> MessagePack<'_, R> {
    pub(crate) fn encode(value: &R::Encode<'_>) -> Result<MessagePack<'static, R>, CodecError> {
        encode(value).map(|bytes| MessagePack(Cow::Owned(bytes), PhantomData))
    }

    pub(crate) fn decode(&self) -> Result<R, CodecError> {
        decode(&self.0)
    }
}

impl<R> fmt::Debug for MessagePack<'_, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessagePack").field("len", &self.0.len()).finish()
    }
}

impl<R: Record> Value for MessagePack<'_, R> {
    type SelfType<'a>
        = MessagePack<'a, R>
    where
        Self: 'a;
    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        MessagePack(Cow::Borrowed(data), PhantomData)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.0.as_ref()
    }

    fn type_name() -> TypeName {
        TypeName::new(R::TYPE_NAME)
    }
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, CodecError> {
    rmp_serde::to_vec_named(value).context(EncodeError)
}

pub(crate) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    let mut decoder = rmp_serde::Deserializer::new(Cursor::new(bytes));
    let decoded = T::deserialize(&mut decoder).context(DecodeError)?;
    ensure!(
        decoder.position() == bytes.len() as u64,
        TrailingDataError { consumed: decoder.position(), length: bytes.len() }
    );
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_values_round_trip_through_fallible_translation() {
        let original = vec![1_u32, 2, 3];
        assert_eq!(decode::<Vec<u32>>(&encode(&original).unwrap()).unwrap(), original);
    }

    #[test]
    fn serialization_rejection_preserves_its_typed_cause() {
        struct Rejected;
        impl Serialize for Rejected {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("Synthetic serializer rejection"))
            }
        }
        let error = encode(&Rejected).unwrap_err();
        assert!(
            matches!(error, CodecError::Encode { source, .. } if source.to_string() == "Synthetic serializer rejection")
        );
    }

    #[test]
    fn a_valid_prefix_is_not_an_entire_stored_record() {
        let error = decode::<u8>(&[7, 8]).unwrap_err();
        assert!(matches!(error, CodecError::TrailingData { consumed: 1, length: 2, .. }));
    }
}
