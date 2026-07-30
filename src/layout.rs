use std::fs::File;
use std::io::{Read, Seek};
use std::{collections::HashSet, fmt, path::Path};

use crate::Container;
use crate::Extension;
use crate::Mark;
use crate::Result;
use crate::{Block, Type};
use crate::{DynReader, Reader, Writer};
use crate::{ReadOptions, WriteOptions};

pub(crate) enum Source {
    Bound(DynReader),
    Unbound,
}

/// The parsed layout of a binary file.
///
/// Contains a complete index of all blocks found in the file, along with the
/// detected container and any format-specific metadata.
pub struct Layout {
    /// All indexed blocks, in the order they appear in the stream.
    pub blocks: Vec<Block>,
    /// The detected container.
    pub container: Container,
    /// The specific file-type or format.
    pub subtype: Option<Mark>,
    /// The size of the file in bytes.
    pub size: u64,
    /// Any extra data that is required.
    pub extension: Option<Extension>,

    pub(crate) source: Source,
}

impl Layout {
    /// Creates a new empty [`Layout`] for building a file from scratch.
    /// The [`Container`] and subtype determine the output format when saving.
    /// All blocks added to this layout must be [`Type::Custom`].
    pub fn new(container: Container, subtype: Mark) -> Self {
        Self {
            blocks: Vec::new(),
            container,
            subtype: Some(subtype),
            size: 0,
            extension: None,
            source: Source::Unbound,
        }
    }

    /// Parses the file at the given path into a [`Layout`].
    ///
    /// Uses [`ReadOptions::default()`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with(path, &ReadOptions::default())
    }

    /// Parses the file at the given path into a [`Layout`].
    ///
    /// Uses the provided [`ReadOptions`].
    pub fn open_with(path: impl AsRef<Path>, opts: &ReadOptions) -> Result<Self> {
        Self::from_reader_with(File::open(path)?, opts)
    }

    pub fn from_reader<R: Read + Seek + 'static>(reader: R) -> Result<Self> {
        Self::from_reader_with(reader, &ReadOptions::default())
    }

    pub fn from_reader_with<R: Read + Seek + 'static>(
        reader: R,
        opts: &ReadOptions,
    ) -> Result<Self> {
        let mut reader = Reader::boxed(reader)?;
        let container = Container::detect(&mut reader)?;

        let mut layout = Layout {
            blocks: Vec::new(),
            container: container,
            subtype: None,
            size: 0,
            extension: None,
            source: Source::Bound(reader),
        };

        container.read_into_layout(&mut layout, opts)?;

        Ok(layout)
    }

    /// Parses the file at the given path using the specified [`Container`].
    ///
    /// This will skip format detection.
    ///
    /// Uses [`ReadOptions::default()`].
    pub fn open_as(path: impl AsRef<Path>, container: Container) -> Result<Self> {
        Self::open_as_with(path, container, &ReadOptions::default())
    }

    /// Parses the file at the given path using the specified [`Container`].
    ///
    /// This will skip format detection.
    ///
    /// Uses the provided [`ReadOptions`].
    pub fn open_as_with(
        path: impl AsRef<Path>,
        container: Container,
        opts: &ReadOptions,
    ) -> Result<Self> {
        let reader = Reader::boxed(File::open(path)?)?;

        let mut layout = Layout {
            blocks: Vec::new(),
            container: container,
            subtype: None,
            size: 0,
            extension: None,
            source: Source::Bound(reader),
        };

        container.read_into_layout(&mut layout, opts)?;

        Ok(layout)
    }

    pub fn save(path: impl AsRef<Path>) -> Result<()> {
        // Self::save_with(path, &WriteOptions::default())
        todo!()
    }

    pub fn save_with(path: impl AsRef<Path>, opts: &WriteOptions) -> Result<()> {
        let mut writer = Writer::create(path)?;
        todo!()
    }

    /// Returns the raw byte payload of a block by seeking to its stored offset in the stream.
    ///
    /// If the layout contains duplicate blocks, the caller is responsible for ensuring
    /// the correct block is passed. Use [`ReadOptions::skip_duplicates`] when reading to
    /// prevent duplicates from being indexed.
    ///
    /// If the block provided is [`Type::Standard`](crate::Type::Standard), this function
    /// returns the payload read from the original stream.
    ///
    /// Do not call this function with a [`Type::Custom`](crate::Type::Custom) block.
    ///
    /// Avoid using this function on large payloads.
    pub fn get_payload(&mut self, index: usize) -> Result<Vec<u8>> {
        match &self.blocks[index]._type {
            Type::Standard { payload_offset, payload_size, .. } => {
                let reader = match &mut self.source {
                    Source::Bound(re) => re,
                    Source::Unbound => unreachable!(),
                };
                reader.seek(*payload_offset)?;
                Ok(reader.read_bytes(*payload_size as usize)?)
            }
            Type::Custom(payload) => Ok(payload.clone()),
        }
    }

    /// Returns a slice of all indexed blocks.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Returns the first block matching the given mark, or `None` if not found.
    pub fn find(&self, mark: Mark) -> Option<&Block> {
        self.blocks.iter().find(|block| block.mark == mark)
    }

    /// Returns all blocks matching the given mark.
    pub fn find_all(&self, mark: Mark) -> Vec<&Block> {
        self.blocks.iter().filter(|block| block.mark == mark).collect()
    }

    /// Returns the index position of the first block matching the given mark, or `None` if not found.
    pub fn position(&self, mark: Mark) -> Option<usize> {
        self.blocks.iter().position(|block| block.mark == mark)
    }

    /// Returns `true` if at least one block with the given mark exists.
    pub fn contains(&self, mark: Mark) -> bool {
        self.blocks.iter().any(|block| block.mark == mark)
    }

    /// Returns `true` if the block index is empty.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    // TODO: Make this return the Marks that are duplicated. Perhaps a Duplicate struct
    // that contains: mark: Mark, amount: u8, positions: Array or Vec
    /// Returns `true` if the layout contains duplicate block marks.
    pub fn has_duplicates(&self) -> bool {
        let mut set = HashSet::new();
        for block in &self.blocks {
            if !set.insert(block.mark) {
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

    /// Removes the first block matching the given mark.
    pub fn remove_block(&mut self, mark: Mark) {
        if let Some(index) = self.position(mark) {
            self.blocks.remove(index);
        }
    }

    /// Removes the block at the given index.
    pub fn remove_block_at(&mut self, index: usize) {
        self.blocks.remove(index);
    }

    /// Removes all blocks matching the given mark.
    pub fn remove_all(&mut self, mark: Mark) {
        self.blocks.retain(|block| block.mark != mark);
    }

    /// Retains only blocks satisfying the given predicate.
    pub fn retain<F: Fn(&Block) -> bool>(&mut self, predicate: F) {
        self.blocks.retain(|block| predicate(block));
    }

    /// Swaps two blocks by index.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.blocks.swap(a, b);
    }

    /// Reorders blocks to match the given mark sequence.
    /// Blocks not present in the sequence are appended in their original order.
    pub fn reorder_blocks(&mut self, order: &[Mark]) {
        todo!()
    }

    //
    // pub fn find_path(&self, path: &[Mark]) -> Option<&Block> { todo!() };
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}, size: {}", self.container, self.size)?;
        writeln!(f, "blocks ({}):", self.blocks.len())?;
        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, "  [{}] {}", i, block)?;
        }
        Ok(())
    }
}
