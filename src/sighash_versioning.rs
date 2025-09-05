//! Sighash versioning.
//!
//! This module defines [`VersionedSig`], which pairs a signature with a [`SighashVersion`],
//! as specified in [ZIP-246].
//!
//! [ZIP-246]: https://zips.z.cash/zip-0246

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// A versioned signature as per [ZIP-246].
///
/// [ZIP-246]: https://zips.z.cash/zip-0246
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedSig<S> {
    version: SighashVersion,
    sig: S,
}

/// The sighash version and associated data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SighashVersion {
    version: u8,
    associated_data: Vec<u8>,
}

/// The `SighashVersion` V0 for Transparent, Sapling, Orchard and issuance.
pub const SIGHASH_V0: SighashVersion = SighashVersion {
    version: 0x00,
    associated_data: vec![],
};

impl<S> VersionedSig<S> {
    /// Constructs a new `VersionedSig` with the given version and signature.
    pub fn new(version: SighashVersion, sig: S) -> Self {
        Self { version, sig }
    }

    /// Returns the version.
    pub fn version(&self) -> &SighashVersion {
        &self.version
    }

    /// Returns the signature.
    pub fn sig(&self) -> &S {
        &self.sig
    }
}

impl SighashVersion {
    /// Constructs a `SighashVersion` from raw bytes.
    ///
    /// Returns `None` if `bytes` is empty.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes.split_first().map(|(&version, info)| Self {
            version,
            associated_data: info.to_vec(),
        })
    }

    /// Returns the raw bytes of the `SighashVersion`.
    pub fn to_bytes(&self) -> Vec<u8> {
        [vec![self.version], self.associated_data.clone()].concat()
    }
}

/// Encodes a size in the CompactSize format.
///
/// This implementation is inspired from `zcash_encoding::CompactSize::write` [code]
/// We cannot use zcash_encoding crate to avoid circular dependency.
///
/// [code]: https://github.com/zcash/librustzcash/blob/8be259c579762f1b0f569453a20c0d0dbeae6c07/components/zcash_encoding/src/lib.rs#L93
pub fn get_compact_size(size: usize) -> Vec<u8> {
    match size {
        s if s < 253 => vec![s as u8],
        s if s <= 0xFFFF => [&[253_u8], &(s as u16).to_le_bytes()[..]].concat(),
        s if s <= 0xFFFFFFFF => [&[254_u8], &(s as u32).to_le_bytes()[..]].concat(),
        s => [&[255_u8], &(s as u64).to_le_bytes()[..]].concat(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sighash_version_encoding_roundtrip() {
        let bytes: [u8; 10] = [2u8; 10];
        let sighash_version = SighashVersion::from_bytes(&bytes).unwrap();
        assert_eq!(bytes[0], sighash_version.version);
        assert_eq!(bytes[1..], sighash_version.associated_data);

        let sighash_version_bytes = sighash_version.to_bytes();
        assert_eq!(bytes, sighash_version_bytes.as_slice());
    }

    #[test]
    fn test_compact_size() {
        assert_eq!(get_compact_size(0), vec![0]);
        assert_eq!(get_compact_size(1), vec![1]);
        assert_eq!(get_compact_size(252), vec![252]);
        assert_eq!(get_compact_size(253), vec![253, 253, 0]);
        assert_eq!(get_compact_size(254), vec![253, 254, 0]);
        assert_eq!(get_compact_size(255), vec![253, 255, 0]);
        assert_eq!(get_compact_size(65535), vec![253, 255, 255]);
        assert_eq!(get_compact_size(65536), vec![254, 0, 0, 1, 0]);
        assert_eq!(get_compact_size(65537), vec![254, 1, 0, 1, 0]);
        assert_eq!(get_compact_size(33554432), vec![254, 0, 0, 0, 2]);
    }
}
