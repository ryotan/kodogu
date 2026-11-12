use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit};
use crypto::{
    Aes256GcmKey, Gmac, GmacKeyWithNonce, MacAlgorithms, MacError, MacTag, MacVerifier, Nonce,
};
use proptest::prelude::*;

fn assert_verification_failed(result: Result<(), MacError>) {
    let error = result.expect_err("invalid GMAC tag must be rejected");
    assert!(matches!(
        error,
        MacError::VerificationFailed { algorithm: MacAlgorithms::Gmac, .. }
    ));
    assert_eq!(error.to_string(), "Gmac: verification failed");
}

fn assert_rejected_by_both_entrypoints(
    message: &[u8],
    bytes: Vec<u8>,
    key: [u8; 32],
    nonce: [u8; 12],
) {
    let key = GmacKeyWithNonce::new(Aes256GcmKey::from(key), Nonce::from(nonce.to_vec()));
    let tag = MacTag::from(bytes);
    assert_verification_failed(Gmac.verify_with_nonce(message, &tag, &key.key, &key.nonce));
    assert_verification_failed(MacVerifier::verify(&Gmac, message, &tag, &key));
}

macro_rules! rejects_wrong_length {
    ($($name:ident: $length:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert_rejected_by_both_entrypoints(b"message", vec![0; $length], rand::random(), rand::random());
            }
        )+
    };
}

rejects_wrong_length! {
    rejects_empty_tag: 0,
    rejects_one_byte_tag: 1,
    rejects_fifteen_byte_tag: 15,
    rejects_seventeen_byte_tag: 17,
    rejects_thirty_two_byte_tag: 32,
}

#[test]
fn rejects_authenticated_ciphertext_as_a_seventeen_byte_tag() {
    let key = rand::random::<[u8; 32]>();
    let nonce = rand::random::<[u8; 12]>();
    let message = b"message";
    let cipher = Aes256Gcm::new((&key).into());
    let bytes = cipher.encrypt((&nonce).into(), Payload { msg: &[0x41], aad: message }).unwrap();
    assert_eq!(bytes.len(), 17);
    assert_rejected_by_both_entrypoints(message, bytes, key, nonce);
}

proptest! {
    #[test]
    fn rejects_authenticated_ciphertexts_with_nonempty_plaintext(
        key in any::<[u8; 32]>(),
        nonce in any::<[u8; 12]>(),
        message in prop::collection::vec(any::<u8>(), 0..256),
        plaintext in prop::collection::vec(any::<u8>(), 1..65),
    ) {
        let cipher = Aes256Gcm::new((&key).into());
        let bytes = cipher.encrypt((&nonce).into(), Payload {msg: &plaintext, aad: &message}).unwrap();
        assert_rejected_by_both_entrypoints(&message, bytes, key, nonce);
    }

    #[test]
    fn accepts_generated_sixteen_byte_tags(
        key in any::<[u8; 32]>(),
        nonce in any::<[u8; 12]>(),
        message in prop::collection::vec(any::<u8>(), 0..256),
    ) {
        let key = GmacKeyWithNonce::new(Aes256GcmKey::from(key), Nonce::from(nonce.to_vec()));
        let tag = Gmac.generate_with_nonce(&message, &key.key, &key.nonce).unwrap();
        prop_assert_eq!(tag.len(), 16);
        prop_assert!(Gmac.verify_with_nonce(&message, &tag, &key.key, &key.nonce).is_ok());
        prop_assert!(MacVerifier::verify(&Gmac, &message, &tag, &key).is_ok());
    }
}
