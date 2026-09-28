//! Names for the CLI's own signing account, derived in one place.
//!
//! A transaction names its signer by the public key it carries in
//! `SignerInfo.public_key`, and every name for the signing account is derived
//! from those key bytes:
//!
//! - [`address`] / [`canonical`] — the canonical Morpheum address,
//!   `SHA256(key)[..20]`, as a `morm1…` string or as its raw 20 bytes. It is
//!   the account a signed transaction is attributed to; fields that take a
//!   `morm1` address carry it.
//! - [`id`] / [`id_hex`] — the 32-byte account id, `SHA256(key)`, whose
//!   leading 20 bytes are the canonical address. Nonce queries and fields that
//!   take a hex account id or an SDK `AccountId` carry it.
//!
//! The derivations live upstream — `morpheum-primitives` for the address,
//! `morpheum-signing` for the account id — and this module only composes
//! them. Every body field that names the signing account derives it here, in
//! the form that field takes. Agent-hash fields carry an agent identity rather
//! than an account name and are not derived here.

use morpheum_primitives::address::{address_from_bytes, canonical_from_bytes};
use morpheum_signing_native::signer::Signer;
#[cfg(feature = "_tx")]
use morpheum_signing_native::types::AccountId;
use morpheum_signing_native::NativeSigner;

/// The key bytes a transaction signed by `signer` carries in
/// `SignerInfo.public_key`.
fn key_bytes(signer: &NativeSigner) -> Vec<u8> {
    signer.public_key_proto().value
}

/// The signer's canonical `morm1…` address.
pub fn address(signer: &NativeSigner) -> String {
    address_from_bytes(&key_bytes(signer))
}

/// The signer's canonical address as its raw 20 bytes.
pub fn canonical(signer: &NativeSigner) -> [u8; 20] {
    canonical_from_bytes(&key_bytes(signer))
}

/// The signer's 32-byte account id.
#[cfg(feature = "_tx")]
pub fn id(signer: &NativeSigner) -> AccountId {
    signer.account_id()
}

/// The signer's account id as lowercase hex.
#[cfg(feature = "_tx")]
pub fn id_hex(signer: &NativeSigner) -> String {
    hex::encode(id(signer).0)
}

#[cfg(test)]
mod tests {
    use morpheum_primitives::address::{decode_address, encode_address};
    use morpheum_primitives::tx::TxWrapper;
    use morpheum_signing_native::proto::tx::v1::Nonce;
    use morpheum_signing_native::types::PublicKey;
    use morpheum_signing_native::Any;

    use super::*;

    /// RFC 8032 §7.1, TEST 1: an Ed25519 secret key and its public key.
    const SECRET_KEY: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const PUBLIC_KEY: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

    /// `SHA256(PUBLIC_KEY)` and the canonical address derived from it.
    /// Computed independently of this crate and of `morpheum-primitives`.
    const ACCOUNT_ID: &str = "21fe31dfa154a261626bf854046fd2271b7bed4b6abe45aa58877ef47f9721b9";
    const ADDRESS: &str = "morm1y8lrrhap2j3xzcntlp2qgm7jyudhhm2tz3ugjc";

    fn bytes<const N: usize>(hex_str: &str) -> [u8; N] {
        hex::decode(hex_str)
            .expect("fixture is hex")
            .try_into()
            .expect("fixture has the expected length")
    }

    fn signer() -> NativeSigner {
        NativeSigner::from_seed(&bytes(SECRET_KEY))
    }

    #[test]
    fn fixture_is_the_rfc8032_key() {
        assert_eq!(signer().public_key(), PublicKey::Ed25519(bytes(PUBLIC_KEY)));
    }

    /// The address is the one `morpheum-primitives` derives from the public
    /// key, pinned to a literal so neither side can drift unnoticed; the raw
    /// form is the same 20 bytes.
    #[test]
    fn address_is_the_primitives_derivation_of_the_public_key() {
        let derived = address(&signer());
        assert_eq!(derived, address_from_bytes(&bytes::<32>(PUBLIC_KEY)));
        assert_eq!(derived, ADDRESS);
        assert_eq!(decode_address(&derived), Some(canonical(&signer())));
    }

    /// The canonical address is the leading 20 bytes of the account id; the
    /// trailing 20 bytes spell a different address.
    #[test]
    fn address_is_the_leading_twenty_bytes_of_the_account_id() {
        let account_id = bytes::<32>(ACCOUNT_ID);
        assert_eq!(canonical(&signer())[..], account_id[..20]);
        assert_ne!(address(&signer()), encode_address(&account_id[12..]));
    }

    /// A transaction the signing builder produces for this key is attributed
    /// to [`address`]: the address is derived from exactly the key bytes the
    /// transaction carries.
    #[tokio::test]
    async fn a_signed_transaction_is_attributed_to_the_address() {
        let key = signer();
        let expected = address(&key);
        let signed = morpheum_signing_native::native(key)
            .chain_id("morpheum-test")
            .with_genesis_hash([1u8; 32])
            .with_nonce(Nonce {
                monotonic: 1,
                ts_ms: 0,
                sub: 0,
            })
            .add_message(Any {
                type_url: "/morpheum.test.v1.Msg".to_string(),
                value: Vec::new(),
            })
            .sign()
            .await
            .expect("sign a one-message transaction");
        assert_eq!(TxWrapper::from_proto(signed.tx).sender().0, expected);
    }

    /// The account id is `SHA256(public key)`.
    #[cfg(feature = "_tx")]
    #[test]
    fn id_is_the_sha256_of_the_public_key() {
        assert_eq!(id_hex(&signer()), ACCOUNT_ID);
    }
}
