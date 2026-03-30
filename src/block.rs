use crate::Marker;
use std::fmt;

/// The source of a block payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    /// Block originated from the parsed stream.
    Original {
        /// The absolute byte offset of the block header within the stream.
        block_offset: u64,
        /// The absolute byte offset of the payload within the stream.
        payload_offset: u64,
        /// The size of the payload in bytes, excluding any header or padding.
        payload_size: u64,
        /// The size of the payload in bytes, including any padding.
        payload_size_with_padding: u64,
    },
    /// Block was provided by the user.
    New { payload: Vec<u8> },
}

/// A single binary block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The identifying marker of this block.
    pub marker: Marker,
    pub block_type: BlockType,
}
impl Block {
    /// Creates a new user-provided block with the given marker and payload.
    ///
    /// ```
    /// let block = Block::new(Marker::FourCC(*b"bext"), payload_bytes);
    /// ```
    pub fn new(marker: Marker, payload: Vec<u8>) -> Self {
        Self {
            marker,
            block_type: BlockType::New { payload },
        }
    }

    /// Returns the absolute byte offset of the block header within the stream,
    /// or `None` if the block was provided by the user.
    pub fn block_offset(&self) -> Option<u64> {
        match &self.block_type {
            BlockType::Original { block_offset, .. } => Some(*block_offset),
            BlockType::New { .. } => None,
        }
    }

    /// Returns the absolute byte offset of the payload within the stream,
    /// or `None` if the block was provided by the user.
    pub fn payload_offset(&self) -> Option<u64> {
        match &self.block_type {
            BlockType::Original { payload_offset, .. } => Some(*payload_offset),
            BlockType::New { .. } => None,
        }
    }

    /// Returns the size of the payload in bytes, excluding any padding.
    pub fn payload_size(&self) -> u64 {
        match &self.block_type {
            BlockType::Original { payload_size, .. } => *payload_size,
            BlockType::New { payload } => payload.len() as u64,
        }
    }

    /// Returns the size of the payload in bytes, including any padding.
    /// If the block was provided by the user, this function will return `payload.len() as u64`.
    pub fn payload_size_with_padding(&self) -> u64 {
        match &self.block_type {
            BlockType::Original {
                payload_size_with_padding,
                ..
            } => *payload_size_with_padding,
            BlockType::New { payload } => payload.len() as u64,
        }
    }

    /// Returns `true` if this block was provided by the user rather than parsed from a stream.
    pub fn is_new(&self) -> bool {
        matches!(self.block_type, BlockType::New { .. })
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.block_type {
            BlockType::Original {
                block_offset,
                payload_offset,
                payload_size,
                ..
            } => match self.marker {
                Marker::FourCC(_) => write!(
                    f,
                    "{} [block: {}, payload: {}, size: {}]",
                    self.marker, block_offset, payload_offset, payload_size
                ),
                Marker::UUID(_) => write!(
                    f,
                    "{}\n\t\t[block: {}, payload: {}, size: {}]",
                    self.marker, block_offset, payload_offset, payload_size
                ),
            },
            BlockType::New { payload } => {
                write!(f, "{} [new, size: {}]", self.marker, payload.len())
            }
        }
    }
}
