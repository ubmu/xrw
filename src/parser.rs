use super::Kind;
use super::extension::{DataSize64, ExtendedData};
use crate::descriptor::MarkerWidth;
use crate::descriptor::SizeWidth;
use crate::{
    Block, BlockType, Descriptor, Error, Family, Marker, ReadOptions, Reader, Result, Structure,
};
use std::io::{Read, Seek};

pub struct Parser;

impl Parser {
    pub fn parse<R: Read + Seek>(reader: &mut Reader<R>, opts: &ReadOptions) -> Result<Structure> {
        let family = Self::detect_family(reader)?;
        let descriptor = Descriptor::try_from(&family)?;
        let (_marker, size, form, extension) = Self::parse_header(reader, &descriptor, &family)?;
        let kind = Kind::try_from(form).ok();
        let eof = Self::eof_offset(size, &family);
        let blocks = Self::index_blocks(reader, &descriptor, &family, &extension, eof, opts)?;

        let structure = Structure {
            blocks,
            descriptor,
            family,
            kind,
            size,
            extension,
        };
        Ok(structure)
    }

    /// Identifies the container family by attempting each detection function in sequence.
    /// Returns [`Error::UnknownContainer`] if no match is found.
    fn detect_family<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Family> {
        let checks: &[fn(&mut Reader<R>) -> Result<Family>] = &[Self::detect_interchange];
        for check in checks {
            reader.seek(0)?;
            if let Ok(family) = check(reader) {
                reader.seek(0)?;
                return Ok(family);
            }
        }
        Err(Error::UnknownFamily)
    }

    /// Identifies interchange variants by reading the first four magic bytes.
    fn detect_interchange<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Family> {
        let marker = Marker::try_from(reader.read_property_code()?)?;
        Family::try_from(marker)
    }

    /// Routes header parsing to the appropriate family-specific parser.
    fn parse_header<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
        family: &Family,
    ) -> Result<(Marker, u64, Marker, ExtendedData)> {
        match family {
            Family::Interchange
            | Family::ResourceInterchange
            | Family::ResourceInterchangeX
            | Family::ResourceInterchange64
            | Family::Wave64 => Self::parse_header_interchange(reader, descriptor),
        }
    }

    /// Parses the outer container header for any interchange variant, returning the container
    /// marker, size, form type, and an optional [`DataSize64`] for RF64 and BW64 files.
    fn parse_header_interchange<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<(Marker, u64, Marker, ExtendedData)> {
        let marker = Self::read_marker(reader, descriptor)?;
        let mut size = Self::read_size(reader, descriptor)?;
        let form = Self::read_marker(reader, descriptor)?;

        let extension = match marker {
            Marker::RF64 | Marker::BW64 => {
                let ds64 = Self::parse_ds64(reader, descriptor)?;
                if size == u32::MAX as u64 {
                    size = ds64.riff_size;
                }
                ExtendedData::DataSize64(ds64)
            }
            _ => ExtendedData::None,
        };

        Ok((marker, size, form, extension))
    }

    /// Parses the `ds64` chunk required by RF64 and BW64 files, which stores the true
    /// 64-bit sizes of chunks whose size fields are set to [`u32::MAX`].
    fn parse_ds64<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<DataSize64> {
        let _offset = reader.tell()?;
        let marker = Self::read_marker(reader, descriptor)?;
        if marker != Marker::DS64 {
            return Err(Error::MissingDS64);
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

    /// Routes block indexing to the appropriate family-specific parser.
    fn index_blocks<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
        family: &Family,
        extension: &ExtendedData,
        eof: u64,
        opts: &ReadOptions,
    ) -> Result<Vec<Block>> {
        match family {
            Family::Interchange
            | Family::ResourceInterchange
            | Family::ResourceInterchangeX
            | Family::ResourceInterchange64
            | Family::Wave64 => {
                Self::index_blocks_interchange(reader, descriptor, extension, eof, opts)
            }
        }
    }

    /// Indexes all blocks within an interchange container, recording offsets and sizes
    /// without reading payloads.
    fn index_blocks_interchange<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
        extension: &ExtendedData,
        eof: u64,
        opts: &ReadOptions,
    ) -> Result<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();
        let ds64 = if let ExtendedData::DataSize64(ds64) = extension {
            Some(ds64)
        } else {
            None
        };

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
            // Override size with the 64-bit one stored in `ds64`.
            if payload_size == u32::MAX as u64 {
                if marker == Marker::DATA {
                    if let ExtendedData::DataSize64(ds64) = extension {
                        payload_size = ds64.data_size;
                    } else {
                        // Marker::DATA with u32::MAX size requires a ds64 chunk.
                        return Err(Error::InvalidBlockSize {
                            offset: reader.tell()?,
                            size: payload_size,
                        });
                    }
                } else {
                    // Excluding Marker::DATA, chunks with u32::MAX sizes are either
                    // unsupported RF64 table entries or simply invalid.
                    return Err(Error::InvalidBlockSize {
                        offset: reader.tell()?,
                        size: payload_size,
                    });
                }
            }

            // Determine the minimum required size for payload to be valid.
            let minimum_size = if opts.validate_minimum_payload_size {
                marker.minimum_payload_size()
            } else {
                1
            };

            let payload_offset = reader.tell()?;
            // Ensure the payload meets the required size and fits within the file.
            if payload_size < minimum_size || payload_offset.saturating_add(payload_size) > eof {
                return Err(Error::InvalidBlockSize {
                    offset: payload_offset,
                    size: payload_size,
                });
            }

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
            let pad = Self::padding_size(descriptor, payload_size);

            let actual_pad = if opts.strict_alignment {
                Self::skip_padding(reader, pad)?;
                pad
            } else {
                if Self::padding_valid(reader, pad)? {
                    pad
                } else {
                    reader.rewind(pad)?;
                    0
                }
            };

            blocks.push(Block {
                marker,
                block_type: BlockType::Original {
                    block_offset,
                    payload_offset,
                    payload_size,
                    payload_size_with_padding: payload_size + actual_pad,
                },
            });
        }

        Ok(blocks)
    }

    /// The number of padding bytes needed to align a block to its boundary.
    fn padding_size(descriptor: &Descriptor, payload_size: u64) -> u64 {
        let alignment = descriptor.block_alignment as u64;
        let remainder = payload_size % alignment;
        if remainder != 0 {
            alignment - remainder
        } else {
            0
        }
    }

    /// Skip padding bytes.
    fn skip_padding<R: Read + Seek>(reader: &mut Reader<R>, pad: u64) -> Result<()> {
        if pad > 0 {
            reader.skip(pad)?;
        }
        Ok(())
    }

    /// Reads the expected padding bytes and returns whether all are null (`0x00`).
    /// Used by `strict_alignment: false` to detect chunks written without padding.
    fn padding_valid<R: Read + Seek>(reader: &mut Reader<R>, pad: u64) -> Result<bool> {
        if pad == 0 {
            return Ok(true);
        }
        let bytes = reader.read_bytes(pad as usize)?;
        let is_padding = bytes.iter().all(|&b| b == 0x00);
        Ok(is_padding)
    }

    /// Reads a block identifier marker of the width defined by the descriptor.
    fn read_marker<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<Marker> {
        match descriptor.width_marker {
            MarkerWidth::FourCC => Ok(Marker::FourCC(reader.read_property_code()?)),
            MarkerWidth::UUID => Ok(Marker::UUID(reader.read_property_uuid()?)),
        }
    }

    /// Reads a size field of the width defined by the descriptor.
    fn read_size<R: Read + Seek>(reader: &mut Reader<R>, descriptor: &Descriptor) -> Result<u64> {
        match descriptor.width_payload_size {
            SizeWidth::U32 => Ok(reader.read_u32(descriptor.byteorder)? as u64),
            SizeWidth::U64 => Ok(reader.read_u64(descriptor.byteorder)?),
        }
    }

    /// Reads a size field and subtracts any header overhead to return the actual payload size.
    fn read_payload_size<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<u64> {
        let offset = reader.tell()?;
        let size = Self::read_size(reader, descriptor)?;
        size.checked_sub(descriptor.header_overhead as u64)
            .ok_or(Error::InvalidBlockSize { offset, size })
    }

    /// The EOF offset.
    fn eof_offset(size: u64, family: &Family) -> u64 {
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
}
