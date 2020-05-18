mod ecdh_nistp;
mod x25519;

use crate::HpkeError;

use digest::generic_array::{typenum::marker_traits::Unsigned, ArrayLength, GenericArray};
use rand::{CryptoRng, RngCore};

/// Implemented by types that have a fixed-length byte representation
pub trait Marshallable {
    type OutputSize: ArrayLength<u8>;

    fn marshal(&self) -> GenericArray<u8, Self::OutputSize>;

    /// Returns the size (in bytes) of this type when marshalled
    fn size() -> usize {
        Self::OutputSize::to_usize()
    }
}

/// Implemented by types that can be deserialized from byte representation
pub trait Unmarshallable: Marshallable + Sized {
    fn unmarshal(encoded: &[u8]) -> Result<Self, HpkeError>;
}

/// This trait captures the requirements of a DH-based KEM (draft02 §5.1). It must have a way to
/// generate keypairs, perform the DH computation, and marshall/umarshall DH pubkeys
pub trait KeyExchange {
    type PublicKey: Clone + Marshallable + Unmarshallable;
    type PrivateKey: Clone + Marshallable + Unmarshallable;
    type KexResult: Marshallable;

    fn gen_keypair<R: CryptoRng + RngCore>(csprng: &mut R) -> (Self::PrivateKey, Self::PublicKey);

    fn sk_to_pk(sk: &Self::PrivateKey) -> Self::PublicKey;

    fn kex(sk: &Self::PrivateKey, pk: &Self::PublicKey) -> Result<Self::KexResult, HpkeError>;
}

pub use ecdh_nistp::DhP256;
pub use x25519::X25519;
