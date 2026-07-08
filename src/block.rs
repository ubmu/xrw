use std::fmt;

use super::marker::Marker;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A block parsed from a stream.
    Original {
        offset: u64,
        payload_offset: u64,
        payload_size: u64,
        padding: u64,
        // sub_blocks: Vec<Block>,
    },
    /// A block with a caller-provided payload.
    Custom(Vec<u8>),
}

/// A block within a structured binary file.
///
/// A block is the fundamental unit of structured binary formats. Depending on
/// the format, blocks may be referred to as chunks, atoms, elements, or boxes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The identifying marker for this block.
    pub marker: Marker,
    /// The origin of a block.
    pub source: Source,
}

impl Block {
    /// Creates a block with a custom payload to be inserted into a file.
    pub fn custom(marker: Marker, payload: Vec<u8>) -> Self {
        Self {
            marker,
            source: Source::Custom(payload),
        }
    }

    /// Returns `true` if this block was parsed from a stream.
    pub fn is_original(&self) -> bool {
        matches!(self.source, Source::Original { .. })
    }

    /// Returns `true` if this is a custom block.
    pub fn is_custom(&self) -> bool {
        matches!(self.source, Source::Custom(_))
    }

    /// Returns the offset of the block header within the stream,
    /// or `None` if this block carries a custom payload.
    pub fn offset(&self) -> Option<u64> {
        match &self.source {
            Source::Original { offset, .. } => Some(*offset),
            Source::Custom { .. } => None,
        }
    }

    /// Returns the offset of the block payload within the stream,
    /// or `None` if this block carries a custom payload.
    pub fn payload_offset(&self) -> Option<u64> {
        match &self.source {
            Source::Original { payload_offset, .. } => Some(*payload_offset),
            Source::Custom { .. } => None,
        }
    }

    /// Returns the size of the payload in bytes.
    pub fn payload_size(&self) -> u64 {
        match &self.source {
            Source::Original { payload_size, .. } => *payload_size,
            Source::Custom(payload) => payload.len() as u64,
        }
    }

    /// Returns the number of padding bytes following the block payload.
    pub(crate) fn padding(&self) -> u64 {
        match &self.source {
            Source::Original { padding, .. } => *padding,
            Source::Custom(..) => 0,
        }
    }

    /// Returns the custom payload bytes, or `None` if this block is original.
    pub(crate) fn raw_payload(&self) -> Option<&[u8]> {
        match &self.source {
            Source::Original { .. } => None,
            Source::Custom(payload) => Some(payload),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Source::Original {
                offset,
                payload_offset,
                payload_size,
                ..
            } => {
                write!(
                    f,
                    "{} [block: {}, payload: {}, size: {}]",
                    self.marker, offset, payload_offset, payload_size
                )
            }
            Source::Custom(payload) => {
                write!(f, "{} [custom, size: {}]", self.marker, payload.len())
            }
        }
    }
}
