use crate::Marker;
use std::fmt;

/// A single binary block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    /// The identifying marker of this block.
    pub marker: Marker,
    /// The absolute byte offset of the block header within the stream.
    pub block_offset: u64,
    /// The absolute byte offset of the payload within the stream.
    pub payload_offset: u64,
    /// The size of the payload in bytes, excluding any header or padding.
    pub payload_size: u64,
    /// The size of the payload in bytes, including any padding.
    pub payload_size_with_padding: u64,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.marker {
            Marker::FourCC(_) => {
                write!(
                    f,
                    "{} [offset: {}, payload_offset: {}, payload_size: {}]",
                    self.marker, self.block_offset, self.payload_offset, self.payload_size,
                )
            }
            Marker::UUID(_) => {
                write!(
                    f,
                    "{}\n\t\t[offset: {}, payload_offset: {}, payload_size: {}]",
                    self.marker, self.block_offset, self.payload_offset, self.payload_size,
                )
            }
        }
    }
}
