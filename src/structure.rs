use crate::{Block, Descriptor, Error, Family, Marker, Reader, Result};
use std::fmt;
use std::io::{Read, Seek};

/// Options for controlling how a binary stream is read and indexed.
#[derive(Debug, Clone, Copy)]
pub struct ReadOptions {
    /// When `true`, only the first occurrence of each block identifier is indexed.
    /// Subsequent duplicates are skipped. Defaults to `false`.
    pub skip_duplicates: bool,
    /// When `true`, strictly enforces block alignment boundaries without validation and assumes
    /// all chunks are correctly padded. When `false`, peeks at the expected padding byte to verify
    /// it is a null byte before skipping, adding minor overhead but ensuring block indexing remains
    /// successful even when chunks are not correctly padded. Defaults to `true`.
    pub strict_alignment: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            skip_duplicates: false,
            strict_alignment: true,
        }
    }
}

/// Options for controlling how a structure is written to a stream.
#[derive(Debug, Clone, Copy)]
pub struct WriteOptions {}

/// The `ds64` chunk, required by RF64 and BW64 files.
///
/// Stores the true 64-bit sizes of chunks whose size fields are set to [`u32::MAX`],
/// which is used as a sentinel value to indicate that the real size exceeds 32 bits.
///
/// EBU Tech 3306-2007
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSize64 {
    pub _offset: u64,
    /// Size of the `ds64` chunk payload in bytes.
    pub _size: u32,
    /// True size of the RIFF container, replacing the outer header size field.
    pub riff_size: u64,
    /// True size of the `data` chunk payload.
    pub data_size: u64,
    /// True sample count, replacing the value in the `fact` chunk.
    pub sample_count: u64,
}

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
    /// The 64-bit size metadata for RF64 and BW64 files.
    pub ds64: Option<DataSize64>,
}

impl Structure {
    const MARK_RIFF: [u8; 4] = *b"RIFF";
    const MARK_RIFX: [u8; 4] = *b"RIFX";
    const MARK_FFIR: [u8; 4] = *b"FFIR";
    const MARK_RF64: [u8; 4] = *b"RF64";
    const MARK_BW64: [u8; 4] = *b"BW64";
    /// First four bytes of the Sony Wave64 container UUID.
    /// W64 stores the FourCC in the first four bytes of its UUID identifier,
    /// allowing detection without reading the full UUID.
    const MARK_SW64: [u8; 4] = *b"riff";
    const MARK_DS64: [u8; 4] = *b"ds64";

    /// Identifies the container family by attempting each detection function in sequence.
    /// Returns [`Error::UnknownContainer`] if no match is found.
    fn determine_container<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Family> {
        let checks: &[fn(&mut Reader<R>) -> Result<Family>] = &[Self::magic_interchange_variant];
        for check in checks {
            reader.seek(0)?;
            if let Ok(family) = check(reader) {
                reader.seek(0)?;
                return Ok(family);
            }
        }
        Err(Error::UnknownContainer)
    }

    /// Identifies interchange variants by reading the first four magic bytes.
    fn magic_interchange_variant<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Family> {
        let marker = Marker::try_from(reader.read_property_code()?)?;
        Family::try_from(marker)
    }

    /// Reads a block identifier marker of the width defined by the descriptor.
    fn read_marker<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<Marker> {
        match descriptor.width_identifier {
            4 => Ok(Marker::FourCC(reader.read_property_code()?)),
            16 => Ok(Marker::UUID(reader.read_property_uuid()?)),
            _ => Err(Error::UnknownContainer),
            //_ => Err(Error::MalformedHeader),
        }
    }

    /// Reads a size field of the width defined by the descriptor.
    fn read_size<R: Read + Seek>(reader: &mut Reader<R>, descriptor: &Descriptor) -> Result<u64> {
        match descriptor.width_payload_size {
            4 => Ok(reader.read_u32(descriptor.byteorder)? as u64),
            8 => Ok(reader.read_u64(descriptor.byteorder)?),
            _ => Err(Error::UnknownContainer),
            //_ => Err(Error::MalformedHeader),
        }
    }

    /// Reads a size field and subtracts any header overhead to return the actual payload size.
    fn read_payload_size<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<u64> {
        let size = Self::read_size(reader, descriptor)?;
        size.checked_sub(descriptor.header_overhead as u64)
            .ok_or(Error::UnknownContainer)
        //.ok_or(Error::InvalidBlockSize)
    }

    /// Parses the outer container header for any interchange variant, returning the container
    /// marker, size, form type, and an optional [`DataSize64`] for RF64 and BW64 files.
    fn parse_interchange_header<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<(Marker, u64, Marker, Option<DataSize64>)> {
        let marker = Self::read_marker(reader, descriptor)?;
        let mut size = Self::read_size(reader, descriptor)?;
        let form = Self::read_marker(reader, descriptor)?;

        let ds64 = match marker {
            Marker::FourCC(Self::MARK_RF64) | Marker::FourCC(Self::MARK_BW64) => {
                let data_size = Self::parse_data_size_chunk(reader, descriptor)?;
                if size == u32::MAX as u64 {
                    size = data_size.riff_size;
                }
                Some(data_size)
            }
            _ => None,
        };

        Ok((marker, size, form, ds64))
    }

    /// Parses the `ds64` chunk required by RF64 and BW64 files, which stores the true
    /// 64-bit sizes of chunks whose size fields are set to [`u32::MAX`].
    fn parse_data_size_chunk<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<DataSize64> {
        let _offset = reader.tell()?;
        let marker = Self::read_marker(reader, descriptor)?;
        if marker != Marker::FourCC(Self::MARK_DS64) {
            // fix error msg
            return Err(Error::UnknownContainer);
            //return Err(Error::MalformedHeader);
        }
        let _size = reader.read_u32(descriptor.byteorder)?;
        let riff_size = reader.read_u64(descriptor.byteorder)?;
        let data_size = reader.read_u64(descriptor.byteorder)?;
        let sample_count = reader.read_u64(descriptor.byteorder)?;
        let table_length = reader.read_u32(descriptor.byteorder)?;
        // NOTE: The table entries track 64-bit sizes for non-data chunks, but no standard
        // chunk other than `data` is realistically expected to exceed 4GB, so they are skipped.
        if table_length > 0 {
            reader.skip(table_length as u64 * 12)?;
        }

        Ok(DataSize64 {
            _offset,
            _size,
            riff_size,
            data_size,
            sample_count,
        })
    }

    /// Computes the EOF offset for the container.
    fn compute_eof(size: u64, family: &Family) -> u64 {
        let eof = match family {
            // Size excludes the 12-byte header fields (marker, size, form).
            Family::Interchange
            | Family::ResourceInterchange
            | Family::ResourceInterchange64
            | Family::ResourceInterchangeX => size + 12,
            // Size includes the full container.
            Family::Wave64 => size,
        };
        eof
    }

    /// Calculates the number of padding bytes needed to align a block to its boundary.
    fn calculate_padding(descriptor: &Descriptor, payload_size: u64) -> u64 {
        let alignment = descriptor.block_alignment as u64;
        let remainder = payload_size % alignment;
        if remainder != 0 {
            alignment - remainder
        } else {
            0
        }
    }

    /// Seeks past padding bytes.
    fn apply_padding<R: Read + Seek>(reader: &mut Reader<R>, pad: u64) -> Result<()> {
        if pad > 0 {
            reader.skip(pad)?;
        }
        Ok(())
    }

    /// Reads the expected padding bytes and returns whether all are null (`0x00`).
    /// Used by `strict_alignment: false` to detect chunks written without padding.
    fn verify_padding<R: Read + Seek>(reader: &mut Reader<R>, pad: u64) -> Result<bool> {
        if pad == 0 {
            return Ok(true);
        }
        let bytes = reader.read_bytes(pad as usize)?;
        let is_padding = bytes.iter().all(|&b| b == 0x00);
        Ok(is_padding)
    }

    /// Indexes all blocks within an interchange container, recording offsets and sizes
    /// without reading payloads.
    fn index_interchange_blocks<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
        eof: u64,
        opts: &ReadOptions,
        ds64: Option<&DataSize64>,
    ) -> Result<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();

        loop {
            if reader.tell()? >= eof {
                break;
            }

            let block_offset = reader.tell()?;
            let marker = match Self::read_marker(reader, descriptor) {
                Ok(m) => m,
                Err(_) => break,
            };

            let mut payload_size = Self::read_payload_size(reader, descriptor)?;

            // RF64/BW64: if chunk size is `u32::MAX` then real size is in `ds64`.
            if payload_size == u32::MAX as u64 {
                if let Some(ds64) = ds64 {
                    println!("here");
                    payload_size = ds64.data_size;
                }
            }

            let payload_offset = reader.tell()?;
            if opts.skip_duplicates && blocks.iter().any(|b| b.marker == marker) {
                reader.seek(payload_offset + payload_size)?;
                continue;
            }

            reader.seek(payload_offset + payload_size)?;

            // Chunk alignment in IFF-based formats requires chunks to be padded to an even-byte
            // boundary (or 8-byte for W64). Padding bytes SHOULD always be null (0x00) by specification.
            //
            // When `strict_alignment` is false, rather than blindly seeking past the calculated
            // padding, the pad bytes are read and verified to be 0x00. If they are null, the
            // padding is accepted and the reader is already positioned at the next block. If any
            // byte is non-null, then the chunk was written without padding and the reader seeks back by
            // the pad amount and the next block is read from the unpadded position instead.
            //
            // This approach handles the two most common cases: chunks incorrectly written without padding,
            // and chunks correctly padded with null bytes. Chunks padded with non-null bytes are not handled.
            let pad = Self::calculate_padding(descriptor, payload_size);

            let actual_pad = if opts.strict_alignment {
                Self::apply_padding(reader, pad)?;
                pad
            } else {
                if Self::verify_padding(reader, pad)? {
                    pad
                } else {
                    reader.rewind(pad)?;
                    0
                }
            };

            blocks.push(Block {
                marker,
                block_offset,
                payload_offset,
                payload_size,
                payload_size_with_padding: payload_size + actual_pad,
            });
        }

        Ok(blocks)
    }

    /// Routes block indexing to the appropriate family-specific parser.
    fn route_index_blocks<R: Read + Seek>(
        reader: &mut Reader<R>,
        family: &Family,
        descriptor: &Descriptor,
        eof: u64,
        opts: &ReadOptions,
        ds64: Option<&DataSize64>,
    ) -> Result<Vec<Block>> {
        match family {
            Family::Interchange
            | Family::ResourceInterchange
            | Family::ResourceInterchangeX
            | Family::ResourceInterchange64
            | Family::Wave64 => Self::index_interchange_blocks(reader, descriptor, eof, opts, ds64),
        }
    }

    /// Routes header parsing to the appropriate family-specific parser.
    fn route_header_parse<R: Read + Seek>(
        reader: &mut Reader<R>,
        family: &Family,
        descriptor: &Descriptor,
    ) -> Result<(Marker, u64, Marker, Option<DataSize64>)> {
        match family {
            Family::Interchange
            | Family::ResourceInterchange
            | Family::ResourceInterchangeX
            | Family::ResourceInterchange64
            | Family::Wave64 => Self::parse_interchange_header(reader, descriptor),
        }
    }

    pub fn test_int<R: Read + Seek>(
        reader: &mut Reader<R>,
        opts: &ReadOptions,
    ) -> Result<Structure> {
        let family = Self::determine_container(reader)?;
        let descriptor = Descriptor::try_from(&family)?;

        let (_marker, size, _form, ds64) = Self::route_header_parse(reader, &family, &descriptor)?;
        // TODO: Figure out how to handle Kind as more formats are supported.
        // let kind = Kind::try_from(form).ok();
        // TODO: function to compute EOF
        // Probably need some EOF::try_from to make this easier
        let eof = Self::compute_eof(size, &family);
        let blocks =
            Self::route_index_blocks(reader, &family, &descriptor, eof, opts, ds64.as_ref())?;

        let structure = Self {
            blocks,
            descriptor,
            ds64,
            family,
            //kind,
            size,
        };

        println!("{}", structure);

        Ok(structure)
    }

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
        let family = Self::determine_container(reader)?;
        let descriptor = Descriptor::try_from(&family)?;

        let (marker, size, form, ds64) = Self::route_header_parse(reader, &family, &descriptor)?;

        //let kind = Kind::try_from(form).ok();
        let blocks =
            Self::route_index_blocks(reader, &family, &descriptor, u64::MAX, opts, ds64.as_ref())?;
        Ok(Self {
            blocks,
            descriptor,
            ds64,
            family,
            //kind,
            size,
        })
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
        todo!()
    }

    /// Returns the first block matching the given marker, or `None` if not found.
    ///
    /// ```
    /// let fmt = structure.find(Marker::FourCC(*b"fmt "));
    /// ```
    pub fn find(&self, marker: Marker) -> Option<&Block> {
        todo!()
    }

    /// Returns all blocks matching the given marker.
    ///
    ///```
    /// let data_all = structure.find_all(Marker::FourCC(*b"data"));
    /// ```
    pub fn find_all(&self, marker: Marker) -> Vec<&Block> {
        todo!()
    }

    ///// Returns the block at the given index, or `None` if out of bounds.
    /////
    /////```
    ///// let first = structure.find_at(0);
    ///// ```
    //pub fn find_at(&self, index: usize) -> Option<&Block> {
    //    todo!()
    //}

    /// Returns the index position of the first block matching the given marker, or `None` if not found.
    ///
    /// ```
    /// let index = structure.position(Marker::FourCC(*b"bext"))
    /// ```
    pub fn position(&self, marker: Marker) -> Option<usize> {
        todo!()
    }

    /// Returns `true` if at least one block with the given marker exists.
    ///
    /// ```
    /// if structure.contains(Marker::FourCC(*b"bext")) { ... }
    /// ```
    pub fn contains(&self, marker: Marker) -> bool {
        todo!()
    }

    /// Returns `true` if the block index is empty.
    ///
    /// ```
    /// if structure.is_empty() { ... }
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Returns `true` if the structure contains duplicate block markers.
    ///
    /// ```
    /// if structure.has_duplicates() { ... }
    pub fn has_duplicates(&self) -> bool {
        todo!()
    }

    ///// Returns the total number of indexed blocks.
    /////
    ///// ```
    ///// let count = structure.block_count();
    ///// ```
    //pub fn block_count(&self) -> usize {
    //    todo!()
    //}

    /// Returns a slice of all indexed blocks.
    ///
    /// ```
    /// for block in structure.blocks() { ... }
    /// ```
    pub fn blocks(&self) -> &[Block] {
        todo!()
    }

    /// Appends a block to the end of the block index.
    ///
    /// ```
    /// structure.add_block(block);
    /// ```
    pub fn add_block(&mut self, block: Block) {
        todo!()
    }

    /// Inserts a block at the given index, shifting subsequent blocks.
    ///
    /// ```
    /// structure.add_block_at(0, block);
    /// ```
    pub fn add_block_at(&mut self, index: usize, block: Block) {
        todo!()
    }

    /// Removes the first block matching the given marker.
    ///
    /// ```
    /// structure.remove_block(Marker::FourCC(*b"LIST"));
    /// ```
    pub fn remove_block(&mut self, marker: Marker) {
        todo!()
    }

    /// Removes the block at the given index.
    ///
    /// ```
    /// structure.remove_block_at(0);
    /// ```
    pub fn remove_block_at(&mut self, index: usize) {
        todo!()
    }

    /// Removes all blocks matching the given marker.
    ///
    /// ```
    /// structure.remove_all(Marker::FourCC(*b"LIST"));
    /// ```
    pub fn remove_all(&mut self, marker: Marker) {
        todo!()
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

    /// Retains only blocks satisfying the given predicate.
    ///
    /// ```
    /// // keep only known metadata blocks, discarding anything unrecognized
    /// let known = [Marker::FourCC(*b"fmt "), Marker::FourCC(*b"data"), Marker::FourCC(*b"bext")];
    /// structure.retain(|block| known.contains(&block.marker));
    /// ```
    pub fn retain<F: Fn(&Block) -> bool>(&mut self, predicate: F) {
        todo!()
    }

    /// Swaps two blocks by index.
    ///
    /// ```
    /// structure.swap(0, 1);
    /// ```
    pub fn swap(&mut self, a: usize, b: usize) {
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
