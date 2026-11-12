use std::fmt;
use std::str::FromStr;

const HASH_BYTES: usize = 32;
const HASH_HEX_LEN: usize = HASH_BYTES * 2;

/// The SHA-256 identity of an immutable file object.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectHash([u8; HASH_BYTES]);

impl ObjectHash {
    pub const fn from_bytes(bytes: [u8; HASH_BYTES]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; HASH_BYTES] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(HASH_HEX_LEN);
        for byte in self.0 {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }

    pub fn parse(value: &str) -> Result<Self, InvalidObjectHash> {
        if value.len() != HASH_HEX_LEN {
            return Err(InvalidObjectHash);
        }

        let mut bytes = [0; HASH_BYTES];
        let value = value.as_bytes();
        for index in 0..HASH_BYTES {
            let high = lower_hex_nibble(value[index * 2]).ok_or(InvalidObjectHash)?;
            let low = lower_hex_nibble(value[index * 2 + 1]).ok_or(InvalidObjectHash)?;
            bytes[index] = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for ObjectHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for ObjectHash {
    type Err = InvalidObjectHash;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectReference {
    pub hash: ObjectHash,
    pub byte_len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidObjectHash;

impl fmt::Display for InvalidObjectHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("object hash must be 64 lowercase hexadecimal characters")
    }
}

impl std::error::Error for InvalidObjectHash {}

fn lower_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ObjectHash;

    #[test]
    fn parses_and_formats_sha256_lowercase_hex() {
        let hash =
            ObjectHash::parse("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
                .unwrap();
        assert_eq!(
            hash.to_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(hash.to_string(), hash.to_hex());
    }

    #[test]
    fn rejects_noncanonical_object_hashes() {
        assert!(ObjectHash::parse("ba7816bf").is_err());
        assert!(
            ObjectHash::parse("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD")
                .is_err()
        );
        assert!(
            ObjectHash::parse("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ag")
                .is_err()
        );
    }
}
