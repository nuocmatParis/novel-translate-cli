//! Closed common wire records. These are data, not validated source positions.
use nt_application::volume_import::source_range as source;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::CodecError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Digest(pub [u8; 32]);

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut bytes = [0_u8; 64];
        for (byte, output) in self.0.iter().zip(bytes.as_chunks_mut::<2>().0) {
            output[0] = HEX[(byte >> 4) as usize];
            output[1] = HEX[(byte & 15) as usize];
        }
        serializer.serialize_str(std::str::from_utf8(&bytes).expect("ASCII hex"))
    }
}
impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Hex;
        impl de::Visitor<'_> for Hex {
            type Value = Digest;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("64 lowercase hex characters")
            }
            fn visit_str<E: de::Error>(self, text: &str) -> Result<Self::Value, E> {
                if text.len() != 64 {
                    return Err(E::custom("Invalid stored digest."));
                }
                fn nibble(byte: u8) -> Option<u8> {
                    match byte {
                        b'0'..=b'9' => Some(byte - b'0'),
                        b'a'..=b'f' => Some(byte - b'a' + 10),
                        _ => None,
                    }
                }
                let mut hash = [0; 32];
                for (pair, out) in text.as_bytes().as_chunks::<2>().0.iter().zip(&mut hash) {
                    *out = (nibble(pair[0]).ok_or_else(|| E::custom("Invalid stored digest."))?
                        << 4)
                        | nibble(pair[1]).ok_or_else(|| E::custom("Invalid stored digest."))?;
                }
                Ok(Digest(hash))
            }
        }
        deserializer.deserialize_str(Hex)
    }
}

/// Accounting for new owned records. The caller additionally reserves all live
/// producer/parser/writer allocations, then shares this one budget per volume.
pub(crate) struct Budget {
    working: usize,
    records: usize,
}
impl Budget {
    pub fn new(reserved: usize) -> Result<Self, CodecError> {
        if reserved > 256 * 1024 * 1024 {
            return Err(CodecError::ResourceLimit);
        }
        Ok(Self {
            working: reserved,
            records: 0,
        })
    }
    pub fn charge(&mut self, bytes: usize, records: usize) -> Result<(), CodecError> {
        self.working = self
            .working
            .checked_add(bytes)
            .filter(|n| *n <= 256 * 1024 * 1024)
            .ok_or(CodecError::ResourceLimit)?;
        self.records = self
            .records
            .checked_add(records)
            .filter(|n| *n <= 32_000_000)
            .ok_or(CodecError::ResourceLimit)?;
        Ok(())
    }
    pub fn remaining_bytes(&self) -> usize {
        256 * 1024 * 1024 - self.working
    }
    pub fn used_bytes(&self) -> usize {
        self.working
    }
    pub fn text(&mut self, text: &str) -> Result<String, CodecError> {
        self.charge(text.len(), 0)?;
        let mut result = String::new();
        result
            .try_reserve_exact(text.len())
            .map_err(|_| CodecError::ResourceLimit)?;
        result.push_str(text);
        Ok(result)
    }
    pub fn optional_text(&mut self, text: &Option<String>) -> Result<Option<String>, CodecError> {
        text.as_deref().map(|text| self.text(text)).transpose()
    }
    pub fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, CodecError> {
        self.charge(
            count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(CodecError::ResourceLimit)?,
            count,
        )?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| CodecError::ResourceLimit)?;
        Ok(result)
    }
    pub fn ordinals(&mut self, values: &[u32]) -> Result<Vec<u32>, CodecError> {
        let mut result = self.vector(values.len())?;
        result.extend_from_slice(values);
        Ok(result)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum BlockLocator {
    UniqueXhtmlId {
        id: String,
    },
    NodePath {
        element_ordinals: Vec<u32>,
        order: u32,
    },
}
impl BlockLocator {
    pub(super) fn capture(
        value: &source::BlockLocator,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(match value {
            source::BlockLocator::UniqueXhtmlId(id) => Self::UniqueXhtmlId {
                id: budget.text(id)?,
            },
            source::BlockLocator::NodePath {
                element_ordinals,
                order,
            } => Self::NodePath {
                element_ordinals: budget.ordinals(element_ordinals)?,
                order: *order,
            },
        })
    }
    pub fn source_locator(&self) -> Result<source::BlockLocator, CodecError> {
        match self {
            Self::UniqueXhtmlId { id } if !id.is_empty() && id.len() <= 16 * 1024 * 1024 => {
                Ok(source::BlockLocator::UniqueXhtmlId(id.clone()))
            }
            Self::NodePath {
                element_ordinals,
                order,
            } if element_ordinals.len() <= 256 => Ok(source::BlockLocator::NodePath {
                element_ordinals: element_ordinals.clone(),
                order: *order,
            }),
            _ => Err(CodecError::InvalidRecord),
        }
    }
    pub(super) fn matches(&self, value: &source::BlockLocator) -> bool {
        match (self, value) {
            (Self::UniqueXhtmlId { id }, source::BlockLocator::UniqueXhtmlId(other)) => id == other,
            (
                Self::NodePath {
                    element_ordinals,
                    order,
                },
                source::BlockLocator::NodePath {
                    element_ordinals: other,
                    order: other_order,
                },
            ) => element_ordinals == other && order == other_order,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElementLocator {
    pub block: BlockLocator,
    pub element_ordinals: Vec<u32>,
}
impl ElementLocator {
    pub(super) fn capture(
        value: &source::ElementLocator,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            block: BlockLocator::capture(&value.block, budget)?,
            element_ordinals: budget.ordinals(&value.element_ordinals)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextLocator {
    pub block: BlockLocator,
    pub text_node_order: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub node: TextLocator,
    pub start_byte: u32,
    pub end_byte: u32,
}
impl Span {
    pub(super) fn capture(
        value: &source::SourceSpan,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            node: TextLocator {
                block: BlockLocator::capture(&value.node().block, budget)?,
                text_node_order: value.node().text_node_order,
            },
            start_byte: value.start_byte(),
            end_byte: value.end_byte(),
        })
    }
    pub(super) fn list(
        values: &[source::SourceSpan],
        budget: &mut Budget,
    ) -> Result<Vec<Self>, CodecError> {
        let mut result = budget.vector(values.len())?;
        for value in values {
            result.push(Self::capture(value, budget)?);
        }
        Ok(result)
    }
    pub(super) fn matches(&self, value: &source::SourceSpan) -> bool {
        self.node.block.matches(&value.node().block)
            && self.node.text_node_order == value.node().text_node_order
            && self.start_byte == value.start_byte()
            && self.end_byte == value.end_byte()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitRange {
    pub start: u32,
    pub end: u32,
}

pub(super) fn safe_path(path: &str) -> bool {
    nt_epub::canonical_archive_path(path).is_ok_and(|canonical| canonical == path)
}

pub(super) fn hash_bytes(bytes: &[u8]) -> Result<Digest, CodecError> {
    use std::io::Write;
    let mut sink = nt_storage::ImportDigestSink::new(512 * 1024 * 1024)
        .map_err(|_| CodecError::ResourceLimit)?;
    sink.write_all(bytes)
        .map_err(|_| CodecError::ResourceLimit)?;
    Ok(Digest(
        *sink
            .finish()
            .map_err(|_| CodecError::ResourceLimit)?
            .hash
            .as_bytes(),
    ))
}

// Retained allocation accounting.
crate::import_codec::memory::heap_enum!(BlockLocator {UniqueXhtmlId {id}, NodePath {element_ordinals, order}});
crate::import_codec::memory::heap_struct!(ElementLocator {
    block,
    element_ordinals
});
crate::import_codec::memory::heap_struct!(TextLocator {
    block,
    text_node_order
});
crate::import_codec::memory::heap_struct!(Span {
    node,
    start_byte,
    end_byte
});
crate::import_codec::memory::heap_struct!(UnitRange { start, end });
