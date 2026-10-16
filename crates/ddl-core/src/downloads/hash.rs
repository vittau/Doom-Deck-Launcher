//! SHA-256 hashes as a small typed value (spec 006, R1/R3/R4).
//!
//! Every remote file DDL uses is pinned by its SHA-256, and the cache is keyed
//! by the same value. Wrapping the 32 raw bytes in a type keeps hashes from
//! being confused with arbitrary strings; [`Sha256::parse_hex`] rejects a value
//! that is not exactly 64 hex digits, so a malformed recipe hash fails early.

use std::fmt;

use sha2::{Digest, Sha256 as Sha256Digest};

/// A SHA-256 digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sha256([u8; 32]);

impl Sha256 {
    /// Wraps the 32 bytes of a digest.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Hashes `data`.
    #[must_use]
    pub fn of(data: &[u8]) -> Self {
        let mut hasher = Sha256Digest::new();
        hasher.update(data);
        Self::finish(hasher)
    }

    /// Parses 64 lowercase or uppercase hex digits. Returns `None` for any
    /// other length or a non-hex character, so callers can reject bad input.
    #[must_use]
    pub fn parse_hex(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, chunk) in bytes.as_chunks::<2>().0.iter().enumerate() {
            out[i] = (hex_value(chunk[0])? << 4) | hex_value(chunk[1])?;
        }
        Some(Self(out))
    }

    /// The raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lowercase hex, the canonical form used for cache file names.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut out = String::with_capacity(64);
        for byte in self.0 {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }

    /// Converts an in-progress [`Sha256Digest`] into a [`Sha256`].
    pub(crate) fn finish(hasher: Sha256Digest) -> Self {
        let digest = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&digest[..]);
        Self(bytes)
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sha256({})", self.to_hex())
    }
}

const HEX: &[u8; 16] = b"0123456789abcdef";

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_known_vectors() {
        // FIPS 180-4 / well-known test vectors.
        assert_eq!(
            Sha256::of(b"").to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            Sha256::of(b"abc").to_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn round_trips_through_hex() {
        let hash = Sha256::of(b"doom");
        let text = hash.to_hex();
        assert_eq!(Sha256::parse_hex(&text), Some(hash));
        assert_eq!(hash.to_string(), text);
    }

    #[test]
    fn parse_accepts_uppercase_and_rejects_malformed_input() {
        let lower = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let upper = lower.to_uppercase();
        assert_eq!(Sha256::parse_hex(&upper), Sha256::parse_hex(lower));

        assert_eq!(Sha256::parse_hex(""), None);
        assert_eq!(Sha256::parse_hex("abcd"), None);
        assert_eq!(Sha256::parse_hex(&"z".repeat(64)), None);
        // 63 digits: one short.
        assert_eq!(Sha256::parse_hex(&lower[..63]), None);
    }
}
