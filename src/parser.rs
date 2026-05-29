use std::io::{Read, Seek};

use crate::block::{Block, Source};
use crate::extension::*;
use crate::form::Form;
use crate::marker::Marker;
use crate::{Container, Descriptor, Error, Layout, ReadOptions, Reader, Result};

pub(crate) struct Parser;

impl Parser {
    /// Probes a reader, detecting the container automatically.
    pub(crate) fn from_reader<R: Read + Seek>(
        reader: &mut Reader<R>,
        opts: &ReadOptions,
    ) -> Result<Layout> {
        let container = Container::detect(reader)?;
        Self::from_reader_as(reader, container, opts)
    }

    /// Probes a reader with a known container, skipping detection.
    pub(crate) fn from_reader_as<R: Read + Seek>(
        reader: &mut Reader<R>,
        container: Container,
        opts: &ReadOptions,
    ) -> Result<Layout> {
        let descriptor = Descriptor::from(&container);
        // Reset the stream before probing.
        reader.seek(0)?;
        match container {
            Container::IFF
            | Container::RIFF
            | Container::RIFX
            | Container::RF64
            | Container::SW64 => Self::probe_interchange(reader, &container, &descriptor, opts),
        }
    }

    /// Returns the EOF offset for a given container and size.
    fn end_offset(size: u64, container: &Container) -> u64 {
        let end_offset = match container {
            // Size excludes the 8-byte marker and size fields.
            Container::IFF | Container::RIFF | Container::RIFX | Container::RF64 => size + 8,
            // Size includes the full container.
            Container::SW64 => size,
        };
        end_offset
    }

    /// Returns the minimum payload size for a given marker.
    fn minimum_payload_size(marker: Marker) -> u64 {
        match marker {
            Marker::FMT => 16,
            _ => 0,
        }
    }
}

impl Parser {
    fn probe_interchange<R: Read + Seek>(
        reader: &mut Reader<R>,
        container: &Container,
        descriptor: &Descriptor,
        opts: &ReadOptions,
    ) -> Result<Layout> {
        // The master chunk header follows the format: identifier, size, form type.
        // The identifier determines the interchange variant, size covers the remaining
        // container body (see end_offset), and form type identifies the file format.
        let master = reader.read_marker(descriptor)?;
        let mut size = reader.read_size(descriptor)?;
        let form = reader.read_marker(descriptor)?;

        // For 64-bit variants, parsing the 'ds64' chunk is required and needed later on.
        let extension = match master {
            Marker::RF64 | Marker::BW64 => {
                let ds64 = Self::parse_ds64(reader, descriptor)?;
                if size == u32::MAX as u64 {
                    size = ds64.riff_size;
                }
                Some(Extension::Ds64(ds64))
            }
            _ => None,
        };

        let end_offset = Self::end_offset(size, container);
        let mut blocks: Vec<Block> = Vec::new();

        loop {
            let block_offset = reader.tell()?;
            // Break if there is not enough room for a chunk header.
            if (block_offset + descriptor.header_width as u64) > end_offset {
                break;
            }

            let marker = reader.read_marker(descriptor)?;
            let mut payload_size = reader.read_payload_size(descriptor)?;
            // Override size with the 64-bit one stored in `ds64`.
            if payload_size == u32::MAX as u64 {
                if marker == Marker::DATA {
                    if let Some(Extension::Ds64(ref ds64)) = extension {
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
                Self::minimum_payload_size(marker)
            } else {
                0
            };

            let payload_offset = reader.tell()?;

            // Ensure payload meets the required size and fits within the file.
            if payload_size < minimum_size
                || payload_offset.saturating_add(payload_size) > end_offset
            {
                return Err(Error::InvalidBlockSize {
                    offset: payload_offset,
                    size: payload_size,
                });
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
            let pad = descriptor.padding_after(payload_size);
            let actual_pad = if opts.strict_alignment {
                reader.skip_padding(pad)?;
                pad
            } else if reader.padding_valid(pad)? {
                pad
            } else {
                reader.rewind(pad)?;
                0
            };

            // Skip duplicates by not adding them to block vector.
            if opts.skip_duplicates && blocks.iter().any(|block| block.marker == marker) {
                continue;
            }

            blocks.push(Block {
                marker,
                source: Source::Original {
                    offset: block_offset,
                    payload_offset,
                    payload_size,
                    padding: actual_pad,
                },
            });
        }

        Ok(Layout {
            blocks,
            container: *container,
            descriptor: *descriptor,
            form: Form::try_from(form).ok(),
            size,
            extension,
        })
    }

    fn parse_ds64<R: Read + Seek>(reader: &mut Reader<R>, descriptor: &Descriptor) -> Result<Ds64> {
        let offset = reader.tell()?;
        let marker = reader.read_marker(descriptor)?;
        if marker != Marker::DS64 {
            return Err(Error::MissingDS64);
        }

        let size = reader.read_u32(descriptor.byteorder)?;
        let riff_size = reader.read_u64(descriptor.byteorder)?;
        let data_size = reader.read_u64(descriptor.byteorder)?;
        let sample_count = reader.read_u64(descriptor.byteorder)?;
        let table_length = reader.read_u32(descriptor.byteorder)?;
        // NOTE: The table entries track 64-bit sizes for non-data chunks, but no standard
        // chunk other than `data` is realistically expected to exceed 4GB, so they are skipped.
        if table_length > 0 {
            reader.skip(table_length as u64 * 12)?;
        }

        Ok(Ds64 {
            offset,
            size,
            riff_size,
            data_size,
            sample_count,
        })
    }
}
