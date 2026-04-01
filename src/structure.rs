use super::extension::ExtendedData;
use super::parser::Parser;
use crate::{
    Block, BlockType, Descriptor, Family, Marker, ReadOptions, Reader, Result, WriteOptions,
};

use std::fmt;
use std::io::{Read, Seek};

/// The parsed structure of a structured binary file.
///
/// Contains a complete index of all blocks found in the stream, along with the
/// detected container family, descriptor, and any family specific metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Structure {
    /// All indexed blocks, in the order they appear in the stream.
    pub blocks: Vec<Block>,
    /// The structural layout descriptor for the detected container family.
    pub descriptor: Descriptor,
    /// The detected container family.
    pub family: Family,
    /// The size of the container payload in bytes.
    pub size: u64,
    // pub kind: Option<Kind>,
    pub extension: ExtendedData,
}

impl Structure {
    /// Parses a binary file into a `Structure`, indexing all blocks without reading their payloads.
    ///
    /// ```
    /// let structure = Structure::read(&mut reader, &ReadOptions::default())?;
    /// ```
    ///
    /// ```
    /// let structure = Structure::read(&mut reader, &ReadOptions { skip_duplicates: true, ..Default::default()}?;
    /// ```
    pub fn read<R: Read + Seek>(reader: &mut Reader<R>, opts: &ReadOptions) -> Result<Self> {
        Parser::parse(reader, opts)
    }

    /// Reads the raw payload of a block by seeking to its stored offset in the stream.
    ///
    ///```
    /// let payload = structure.read_payload(&block, &mut reader)?;
    /// ```
    ///
    /// If the structure contains duplicate blocks, the caller is responsible for ensuring
    /// the correct block is passed. Use [`ReadOptions::skip_duplicates`] when reading to
    /// prevent duplicates from being indexed.
    pub fn read_payload<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
        block: &Block,
    ) -> Result<Vec<u8>> {
        match &block.block_type {
            BlockType::Original {
                payload_offset,
                payload_size,
                ..
            } => {
                reader.seek(*payload_offset)?;
                Ok(reader.read_bytes(*payload_size as usize)?)
            }
            BlockType::New { payload } => Ok(payload.clone()),
        }
    }

    /// Returns a slice of all indexed blocks.
    ///
    /// ```
    /// for block in structure.blocks() { ... }
    /// ```
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Returns the first block matching the given marker, or `None` if not found.
    ///
    /// ```
    /// let fmt = structure.find(Marker::FourCC(*b"fmt "));
    /// ```
    pub fn find(&self, marker: Marker) -> Option<&Block> {
        // TODO: Consider checking if marker type provided matches those in blocks.
        self.blocks.iter().find(|block| block.marker == marker)
    }

    /// Returns all blocks matching the given marker.
    ///
    ///```
    /// let data_all = structure.find_all(Marker::FourCC(*b"data"));
    /// ```
    pub fn find_all(&self, marker: Marker) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|block| block.marker == marker)
            .collect()
    }

    /// Returns the index position of the first block matching the given marker, or `None` if not found.
    ///
    /// ```
    /// let index = structure.position(Marker::FourCC(*b"bext"))
    /// ```
    pub fn position(&self, marker: Marker) -> Option<usize> {
        self.blocks.iter().position(|block| block.marker == marker)
    }

    /// Returns `true` if at least one block with the given marker exists.
    ///
    /// ```
    /// if structure.contains(Marker::FourCC(*b"bext")) { ... }
    /// ```
    pub fn contains(&self, marker: Marker) -> bool {
        self.blocks.iter().any(|block| block.marker == marker)
    }

    /// Returns `true` if the block index is empty.
    ///
    /// ```
    /// if structure.is_empty() { ... }
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns `true` if the structure contains duplicate block markers.
    ///
    /// ```
    /// if structure.has_duplicates() { ... }
    pub fn has_duplicates(&self) -> bool {
        self.blocks
            .iter()
            .enumerate()
            .any(|(i, a)| self.blocks[i + 1..].iter().any(|b| b.marker == a.marker))
    }

    /// Appends a block to the end of the block index.
    ///
    /// ```
    /// structure.add_block(block);
    /// ```
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    /// Inserts a block at the given index, shifting subsequent blocks.
    ///
    /// ```
    /// structure.add_block_at(0, block);
    /// ```
    pub fn insert_block(&mut self, index: usize, block: Block) {
        self.blocks.insert(index, block);
    }

    /// Removes the first block matching the given marker.
    ///
    /// ```
    /// structure.remove_block(Marker::FourCC(*b"LIST"));
    /// ```
    pub fn remove_block(&mut self, marker: Marker) {
        if let Some(index) = self.position(marker) {
            self.blocks.remove(index);
        }
    }

    /// Removes the block at the given index.
    ///
    /// ```
    /// structure.remove_block_at(0);
    /// ```
    pub fn remove_block_at(&mut self, index: usize) {
        self.blocks.remove(index);
    }

    /// Removes all blocks matching the given marker.
    ///
    /// ```
    /// structure.remove_all(Marker::FourCC(*b"LIST"));
    /// ```
    pub fn remove_all(&mut self, marker: Marker) {
        self.blocks.retain(|block| block.marker != marker);
    }

    /// Retains only blocks satisfying the given predicate.
    ///
    /// ```
    /// // keep only known metadata blocks, discarding anything unrecognized
    /// let known = [Marker::FourCC(*b"fmt "), Marker::FourCC(*b"data"), Marker::FourCC(*b"bext")];
    /// structure.retain(|block| known.contains(&block.marker));
    /// ```
    pub fn retain<F: Fn(&Block) -> bool>(&mut self, predicate: F) {
        self.blocks.retain(|block| predicate(block));
    }

    /// Swaps two blocks by index.
    ///
    /// ```
    /// structure.swap(0, 1);
    /// ```
    pub fn swap(&mut self, a: usize, b: usize) {
        self.blocks.swap(a, b);
    }

    /// Reorders blocks to match the given marker sequence.
    /// Blocks not present in the sequence are appended in their original order.
    ///
    /// ```
    /// structure.reorder_blocks(&[Marker::FourCC(*b"fmt "), Marker::FourCC(*b"data")]);
    /// ```
    pub fn reorder_blocks(&mut self, order: &[Marker]) {
        todo!()
    }

    /// Promotes the container to a 64-bit variant when the declared size exceeds [`u32::MAX`].
    /// For RIFF this means promoting to RF64 and inserting a [`ds64`] chunk.
    ///
    /// ```
    /// structure.promote_container()?;
    /// ```
    ///
    /// TODO: Should this even be public?
    pub fn promote_container(&mut self) -> Result<()> {
        todo!()
    }

    ///// Writes the structure to a stream from scratch. The caller provides all block payloads.
    ///// Size fields, padding, alignment, and endianness are handled automatically.
    // pub fn write<W: Write + Seek>(&self, writer: &mut W, opts: &WriteOptions) -> Result<()> {
    //    todo!()
    //}

    ///// Writes the structure to a stream by reading block payloads from their stored offsets
    ///// in the original stream. Size fields, padding, alignment, and endianness are handled automatically.
    //pub fn write_from<R: Read + Seek, W: Write + Seek>(
    //    &self,
    //    reader: &mut Reader<R>,
    //    writer: &mut W,
    //    opts: &WriteOptions,
    //) -> Result<()> {
    //    todo!()
    //}

    /// Converts the structure to the target container family, remapping identifiers where needed
    /// and preserving all blocks. Returns an error if the conversion is not supported.
    ///
    /// ```
    /// let rf64 = structure.convert(Family::ResourceInterchange64)?;
    /// ```
    pub fn convert(&self, target: Family) -> Result<Structure> {
        todo!()
    }
}

impl fmt::Display for Structure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // looks good enough. TODO: maybe come back to this when doing
        // display for coreav
        writeln!(f, "header [family: {}, size: {}]", self.family, self.size)?;
        if let Some(ref ds64) = self.ds64 {
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
