use crate::{prelude::*, util::static_zeros};

use core::u16;

use byteorder::{BigEndian, WriteBytesExt};
use digest::{generic_array::GenericArray, BlockInput, Digest, FixedOutput, Input, Reset};
use sha2::{Sha256, Sha384, Sha512};

// This has a space because LabeledExtract calls for a space between the RFC string and the label
const RFC_STR: &[u8] = b"RFCXXXX ";

// Pretty much all the KDF functionality is covered by the hkdf crate

/// Represents key derivation functionality
pub trait Kdf {
    /// The underlying hash function
    type HashImpl: Digest + Input + BlockInput + FixedOutput + Reset + Default + Clone;

    /// The algorithm identifier for a KDF implementation
    const KDF_ID: u16;
}

/// The implementation of HKDF-SHA256
pub struct HkdfSha256 {}

impl Kdf for HkdfSha256 {
    type HashImpl = Sha256;

    // draft02 §8.2: HKDF-SHA256
    const KDF_ID: u16 = 0x0001;
}

/// The implementation of HKDF-SHA384
pub struct HkdfSha384 {}

impl Kdf for HkdfSha384 {
    type HashImpl = Sha384;

    // draft02 §8.2: HKDF-SHA384
    const KDF_ID: u16 = 0x0002;
}

/// The implementation of HKDF-SHA512
pub struct HkdfSha512 {}

impl Kdf for HkdfSha512 {
    type HashImpl = Sha512;

    // draft02 §8.2: HKDF-SHA512
    const KDF_ID: u16 = 0x0003;
}

// def ExtractAndExpand(dh, kemContext):
//   prk = LabeledExtract(zero(Nh), "dh", dh)
//   return LabeledExpand(prk, "prk", kemContext, Nzz)
/// Uses the given IKM to extract a secret, and then uses that secret, plus the given info string,
/// to expand to the output buffer
pub(crate) fn extract_and_expand<K: Kdf>(
    ikm: &[u8],
    info: &[u8],
    out: &mut [u8],
) -> Result<(), hkdf::InvalidLength> {
    // The salt is a zero array of length Nh
    let salt = static_zeros::<K>();
    // Extract using given IKM
    let (_, hkdf_ctx) = hkdf::Hkdf::<K::HashImpl>::extract(Some(&salt), ikm);
    // Expand using given info string
    hkdf_ctx.expand(info, out)
}

// def LabeledExtract(salt, label, IKM):
//   labeledIKM = concat("RFCXXXX ", label, IKM)
//   return Extract(salt, labeledIKM)
/// Returns the HKDF context derived from `(salt=salt, ikm= "RFCXXXX"||label||ikm)`
pub(crate) fn labeled_extract<K: Kdf>(
    salt: &[u8],
    label: &[u8],
    ikm: &[u8],
) -> (
    GenericArray<u8, <<K as Kdf>::HashImpl as FixedOutput>::OutputSize>,
    hkdf::Hkdf<K::HashImpl>,
) {
    // Concat the inputs to create a new IKM
    let labeled_ikm: Vec<u8> = [RFC_STR, label, ikm].concat();
    // Extract and the HKDF context
    hkdf::Hkdf::<K::HashImpl>::extract(Some(&salt), &labeled_ikm)
}

// This trait only exists so I can implement it for hkdf::Hkdf
pub(crate) trait LabeledExpand {
    fn labeled_expand(
        &self,
        label: &[u8],
        info: &[u8],
        out: &mut [u8],
    ) -> Result<(), hkdf::InvalidLength>;
}

impl<D: Input + BlockInput + FixedOutput + Reset + Default + Clone> LabeledExpand
    for hkdf::Hkdf<D>
{
    // def LabeledExpand(PRK, label, info, L):
    //   labeledInfo = concat(encode_big_endian(L, 2),
    //                         "RFCXXXX ", label, info)
    //   return Expand(PRK, labeledInfo, L)
    fn labeled_expand(
        &self,
        label: &[u8],
        info: &[u8],
        out: &mut [u8],
    ) -> Result<(), hkdf::InvalidLength> {
        assert!(out.len() <= u16::MAX as usize);

        // Encode the output length in the info string
        let mut len_buf = [0u8; 2];
        (&mut len_buf[..])
            .write_u16::<BigEndian>(out.len() as u16)
            .unwrap();

        let labeled_info: Vec<u8> = [&len_buf, RFC_STR, label, info].concat();
        self.expand(&labeled_info, out)
    }
}
