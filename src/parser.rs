use std::io::{Read, Seek};

use super::block::{Block, Source};
use super::container::Container;
use super::descriptor::Descriptor;
use super::extension::*;
use super::form::Form;
use super::io::Reader;
use super::layout::Layout;
use super::marker::Marker;
use super::opts::ReadOptions;
use crate::{Error, Result};

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
        // Derive descriptor from container.
        let descriptor = Descriptor::from(&container);

        // Reset the stream before probing.
        reader.seek(0)?;

        // Probing is intentionally permissive and focuses only on extracting enough information
        // to parse the file structure. Specification validation is  deferred to the `Builder`,
        // which validates and repairs fields according to `WriteOptions` during writing.
        Self::route_container_probe(reader, container, descriptor, opts)
    }

    /// Route to the container-specific probe function.
    fn route_container_probe<R: Read + Seek>(
        reader: &mut Reader<R>,
        container: Container,
        descriptor: Descriptor,
        opts: &ReadOptions,
    ) -> Result<Layout> {
        match container {
            Container::Interchange
            | Container::ResourceInterchange
            | Container::ResourceInterchangeBE
            | Container::ResourceInterchange64
            | Container::SonyWave64 => {
                Self::probe_interchange(reader, &container, &descriptor, opts)
            }
            Container::CoreAudio => Self::probe_coreaudio(reader, &container, &descriptor, opts),
        }
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
        // The master chunk header follows the format:

        // The container identifying marker. This is either a FourCC or UUID.
        let master = reader.read_marker(descriptor)?;
        // The number of remaining bytes in the container for all variants excluding Sony Wave64.
        // This is the same as filesize + 8 bytes as this size value does not include the 4-byte
        // FourCC identifying marker and the 4-byte size field.
        // For Sony Wave64, this value is the filesize.
        let mut size = reader.read_size(descriptor)?;
        // The specific file or form type. This is either a FourCC or UUID.
        let subtype = reader.read_marker(descriptor)?;

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

        let end_offset = reader.size();
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
            // boundary (or 8-byte for W64). The specification requires padding bytes to be null (0x00).
            //
            // When `assume_strict_alignment` is false, rather than blindly seeking past the calculated
            // padding, the pad bytes are read and verified to be 0x00. If they are null, the
            // padding is accepted and the reader is already positioned at the next block. If any
            // byte is non-null, then the chunk was written without padding and the reader seeks back by
            // the pad amount and the next block is read from the unpadded position instead.
            //
            // This approach handles the two most common cases: chunks that are correctly
            // padded with null bytes and chunks that omit the required padding entirely.
            //
            // Chunks with incorrect (non-null) padding are treated as though no padding
            // were present and are therefore not handled correctly.
            let pad = descriptor.padding_after(payload_size);
            let actual_pad = if opts.assume_strict_alignment {
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
                marker: marker,
                source: Source::Original {
                    offset: block_offset,
                    payload_offset: payload_offset,
                    payload_size: payload_size,
                    padding: actual_pad,
                },
            });
        }

        Ok(Layout {
            blocks: blocks,
            container: *container,
            descriptor: *descriptor,
            subtype: subtype,
            size: size,
            extension: extension,
        })
    }

    fn probe_coreaudio<R: Read + Seek>(
        reader: &mut Reader<R>,
        container: &Container,
        descriptor: &Descriptor,
        opts: &ReadOptions,
    ) -> Result<Layout> {
        // Reference:
        // https://developer.apple.com/library/archive/documentation/MusicAudio/Reference/CAFSpec/CAF_spec/CAF_spec.html

        // Semantic validation (for example, ensuring the file type and version match the CAF specification) is
        // deferred to the `Builder`, which always validates or repairs these fields according to `WriteOptions`.

        // The header follows the format:

        // The file-type [FourCC] which must be 'caff'.
        let _file_type = reader.read_marker(descriptor)?;
        // The file version [u16] which must be set to 1.
        let _file_version = reader.read_u16(descriptor.byteorder)?;
        // The file flags [u16] which is reserved by the specification. Must be set to 0.
        let _file_flags = reader.read_u16(descriptor.byteorder)?;

        let header = CoreAudioHeader {
            file_flags: _file_flags,
            file_version: _file_version,
        };

        let extension = Extension::CoreAudioHeader(header);

        // As a size value is not provided, we will default to reader.size()
        let size = reader.size();

        let mut blocks: Vec<Block> = Vec::new();

        // The CAF specification requires the `desc` chunk to immediately follow the file header.
        // Since probing only validates anything that can interfere with parsing, this requirement is not
        // enforced here.
        loop {
            let offset = reader.tell()?;
            if (offset + descriptor.header_width as u64) > size {
                break;
            }

            let marker = reader.read_marker(descriptor)?;
            let raw_payload_size = reader.read_i64(descriptor.byteorder)?;
            let payload_offset = reader.tell()?;

            // Valid for ['data']: payload extends to EOF.
            let payload_size = if raw_payload_size == -1 {
                if marker != Marker::DATA {
                    return Err(Error::NegativeSize {
                        offset: payload_offset,
                    });
                }
                reader.size() - payload_offset
            } else {
                raw_payload_size as u64
            };

            let minimum_size = if opts.validate_minimum_payload_size {
                Self::minimum_payload_size(marker)
            } else {
                0
            };

            if payload_size < minimum_size || payload_offset.saturating_add(payload_size) > size {
                return Err(Error::InvalidBlockSize {
                    offset: payload_offset,
                    size: payload_size,
                });
            }

            reader.seek(payload_offset + payload_size)?;

            if opts.skip_duplicates && blocks.iter().any(|b| b.marker == marker) {
                continue;
            }

            blocks.push(Block {
                marker: marker,
                source: Source::Original {
                    offset: offset,
                    payload_offset: payload_offset,
                    payload_size: payload_size,
                    padding: 0,
                },
            });
        }

        Ok(Layout {
            blocks: blocks,
            container: *container,
            descriptor: *descriptor,
            subtype: Marker::CAFF,
            size: size,
            extension: Some(extension),
        })
    }

    fn parse_ds64<R: Read + Seek>(reader: &mut Reader<R>, descriptor: &Descriptor) -> Result<Ds64> {
        let offset = reader.tell()?;
        let marker = reader.read_marker(descriptor)?;
        if marker != Marker::DS64 {
            return Err(Error::MissingDs64 { got: marker });
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
            offset: offset,
            size: size,
            riff_size: riff_size,
            data_size: data_size,
            sample_count: sample_count,
        })
    }
}
