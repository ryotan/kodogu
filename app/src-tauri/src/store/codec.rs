//! Fallible MessagePack translation outside redb's infallible Value callbacks.

use serde::{Serialize, de::DeserializeOwned};
use std::io::Cursor;
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

pub(crate) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, CodecError> {
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
