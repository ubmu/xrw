use std::collections::HashSet;
use std::fmt;
use std::io::{Read, Seek};

use super::block::Block;
use super::container::Container;
use super::descriptor::Descriptor;
use super::extension::Extension;
use super::io::Reader;
use super::marker::Marker;
use super::opts::ReadOptions;
use super::parser::Parser;
use crate::Result;

/// The parsed layout of a binary file.
///
/// Contains a complete index of all blocks found in the file, along with the
/// detected container family, descriptor, and any family specific metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// All indexed blocks, in the order they appear in     the stream.
    pub blocks: Vec<Block>,
    /// The structural layout descriptor for the detected container family.
    pub descriptor: Descriptor,
    /// The detected container family.
    pub container: Container,
    /// The specific file-type or format.
    pub subtype: Marker,
    /// The size of the file in bytes.
    pub size: u64,
    /// Any extra data that is required.
    pub extension: Option<Extension>,
}

impl Layout {
    /// Parses a reader into a `Layout`, indexing all blocks without reading their payloads.
    pub fn from_reader<R: Read + Seek>(reader: &mut Reader<R>, opts: &ReadOptions) -> Result<Self> {
        Parser::from_reader(reader, opts)
    }

    /// Parses a reader into a `Layout`, indexing all blocks without reading their payloads,
    /// using the specified container.
    pub fn from_reader_as<R: Read + Seek>(
        reader: &mut Reader<R>,
        container: Container,
        opts: &ReadOptions,
    ) -> Result<Self> {
        Parser::from_reader_as(reader, container, opts)
    }

    /// Returns the raw byte payload of a block by seeking to its stored offset in the stream.
    ///
    /// If the layout contains duplicate blocks, the caller is responsible for ensuring
    /// the correct block is passed. Use [`ReadOptions::skip_duplicates`] when reading to
    /// prevent duplicates from being indexed.
    ///
    /// If the block provided is of `BlockType::New` then this function will return a clone
    /// of the payload.
    ///
    /// Do not use this function on blocks with large payloads.
    pub fn payload_bytes<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
        block: &Block,
    ) -> Result<&[u8]> {
        todo!()
    }

    /// Returns a slice of all indexed blocks.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Returns the first block matching the given marker, or `None` if not found.
    pub fn find(&self, marker: Marker) -> Option<&Block> {
        // TODO: Consider checking if marker type provided matches those in blocks.
        self.blocks.iter().find(|block| block.marker == marker)
    }

    /// Returns all blocks matching the given marker.
    pub fn find_all(&self, marker: Marker) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|block| block.marker == marker)
            .collect()
    }

    /// Returns the index position of the first block matching the given marker, or `None` if not found.
    pub fn position(&self, marker: Marker) -> Option<usize> {
        self.blocks.iter().position(|block| block.marker == marker)
    }

    /// Returns `true` if at least one block with the given marker exists.
    pub fn contains(&self, marker: Marker) -> bool {
        self.blocks.iter().any(|block| block.marker == marker)
    }

    /// Returns `true` if the block index is empty.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns `true` if the layout contains duplicate block markers.
    pub fn has_duplicates(&self) -> bool {
        let mut set = HashSet::new();
        for block in &self.blocks {
            if !set.insert(block.marker) {
                return true;
            }
        }
        false
    }

    /// Appends a block to the end of the block index.
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    /// Inserts a block at the given index, shifting subsequent blocks.
    pub fn insert_block(&mut self, index: usize, block: Block) {
        self.blocks.insert(index, block);
    }

    /// Removes the first block matching the given marker.
    pub fn remove_block(&mut self, marker: Marker) {
        if let Some(index) = self.position(marker) {
            self.blocks.remove(index);
        }
    }

    /// Removes the block at the given index.
    pub fn remove_block_at(&mut self, index: usize) {
        self.blocks.remove(index);
    }

    /// Removes all blocks matching the given marker.
    pub fn remove_all(&mut self, marker: Marker) {
        self.blocks.retain(|block| block.marker != marker);
    }

    /// Retains only blocks satisfying the given predicate.
    pub fn retain<F: Fn(&Block) -> bool>(&mut self, predicate: F) {
        self.blocks.retain(|block| predicate(block));
    }

    /// Swaps two blocks by index.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.blocks.swap(a, b);
    }

    /// Reorders blocks to match the given marker sequence.
    /// Blocks not present in the sequence are appended in their original order.
    pub fn reorder_blocks(&mut self, order: &[Marker]) {
        todo!()
    }

    ///// Writes the layout to a stream from scratch. The caller provides all block payloads.
    ///// Size fields, padding, alignment, and endianness are handled automatically.
    // pub fn write<W: Write + Seek>(&self, writer: &mut W, opts: &WriteOptions) -> Result<()> {
    //    todo!()
    //}

    ///// Writes the layout to a stream by reading block payloads from their stored offsets
    ///// in the original stream. Size fields, padding, alignment, and endianness are handled automatically.
    //pub fn write_from<R: Read + Seek, W: Write + Seek>(
    //    &self,
    //    reader: &mut Reader<R>,
    //    writer: &mut W,
    //    opts: &WriteOptions,
    //) -> Result<()> {
    //    todo!()
    //}

    /// Converts the layout to the target container family, remapping identifiers where needed
    /// and preserving all blocks. Returns an error if the conversion is not supported.
    pub fn convert(&self, target: Container) -> Result<Layout> {
        todo!()
    }
}

// TODO: Handle this later.
impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "header [container: {}, size: {}]",
            self.container, self.size
        )?;
        if let Some(Extension::Ds64(ref ds64)) = self.extension {
            writeln!(
                f,
                "ds64   [riff_size: {}, data_size: {}, sample_count: {}]",
                ds64.riff_size, ds64.data_size, ds64.sample_count,
            )?;
        }
        writeln!(f, "blocks ({}):", self.blocks.len())?;
        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, "  [{}] {}", i, block)?;
        }
        Ok(())
    }
}
