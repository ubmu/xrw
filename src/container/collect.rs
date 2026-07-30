use super::{Container, preamble::Preamble};
use crate::{Block, Error, Extension, Mark, ReadOptions, Reader, Result, Type};
use std::io::{Read, Seek};

impl Container {
    pub(super) fn collect_blocks_inter<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
        preamble: &Preamble,
        opts: &ReadOptions,
    ) -> Result<Vec<Block>> {
        let descriptor = self.descriptor();
        let mut blocks: Vec<Block> = Vec::new();

        loop {
            let block_offset = reader.tell()?;
            if (block_offset + descriptor.header_width() as u64) > preamble.size {
                break;
            }

            let mark = reader.read_mark(&descriptor)?;
            let mut payload_size = reader.read_payload_size(&descriptor)?;
            if payload_size == u32::MAX as u64 {
                if mark == Mark::DATA {
                    if let Some(Extension::Ds64(ds64)) = &preamble.extension {
                        payload_size = ds64.data_size;
                    } else {
                        return Err(Error::Read(format!(
                            "'data' chunk at offset {block_offset} declares a 64-bit size but no 'ds64' chunk was found"
                        )));
                    }
                } else {
                    return Err(Error::Read(format!(
                        "chunk '{mark}' at offset {block_offset} has an unsupported 32-bit sentinel size"
                    )));
                }
            }

            let payload_offset = reader.tell()?;
            // REMOVE
            ensure_minimum_payload_size(
                preamble.size,
                payload_offset,
                payload_size,
                minimum_payload_size(mark),
                opts,
            )?;

            reader.seek(payload_offset + payload_size)?;
            let pad = descriptor.padding_after(payload_size);
            let padding = if opts.assume_strict_alignment {
                reader.skip_padding(pad)?;
                pad
            } else if reader.padding_valid(pad)? {
                pad
            } else {
                reader.rewind(pad)?;
                0
            };

            if opts.skip_duplicates && blocks.iter().any(|block| block.mark == mark) {
                continue;
            }

            blocks.push(Block {
                mark,
                _type: Type::Standard {
                    offset: block_offset,
                    payload_offset,
                    payload_size,
                    padding,
                },
            });
        }

        Ok(blocks)
    }

    pub(super) fn collect_blocks_core<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
        preamble: &Preamble,
        opts: &ReadOptions,
    ) -> Result<Vec<Block>> {
        let descriptor = self.descriptor();
        let mut blocks: Vec<Block> = Vec::new();

        loop {
            let offset = reader.tell()?;
            if (offset + descriptor.header_width() as u64) > preamble.size {
                break;
            }

            let mark = reader.read_mark(&descriptor)?;
            let raw_payload_size = reader.read_i64(descriptor.byteorder)?;
            let payload_offset = reader.tell()?;

            let payload_size = if raw_payload_size == -1 {
                if mark != Mark::DATA {
                    return Err(Error::Read(format!(
                        "chunk '{mark}' at offset {payload_offset} declared a negative size"
                    )));
                }
                reader.size() - payload_offset
            } else {
                raw_payload_size as u64
            };

            ensure_minimum_payload_size(
                preamble.size,
                payload_offset,
                payload_size,
                minimum_payload_size(mark),
                opts,
            )?;

            // Move this last portion into a separate general function.
            // e.g. seek_and_push()
            reader.seek(payload_offset + payload_size)?;
            if opts.skip_duplicates && blocks.iter().any(|b| b.mark == mark) {
                continue;
            }

            blocks.push(Block {
                mark,
                _type: Type::Standard {
                    offset,
                    payload_offset,
                    payload_size,
                    padding: 0,
                },
            });
        }

        Ok(blocks)
    }

    pub(super) fn collect_blocks_base<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
        preamble: &Preamble,
        opts: &ReadOptions,
    ) -> Result<Vec<Block>> {
        todo!()
    }
}

// TODO: After pushing changes, remove this and state that this check is being moved to adhere.
fn minimum_payload_size(mark: Mark) -> u64 {
    match mark {
        Mark::FMT_ => 16,
        _ => 0,
    }
}

fn ensure_minimum_payload_size(
    size: u64,
    payload_offset: u64,
    payload_size: u64,
    minimum_size: u64,
    opts: &ReadOptions,
) -> Result<()> {
    let minimum_size =
        if opts.validate_minimum_payload_size { minimum_size } else { 0 };
    if payload_size < minimum_size
        || payload_offset.saturating_add(payload_size) > size
    {
        return Err(Error::Read(format!(
            "invalid block size {payload_size} at offset {payload_offset}"
        )));
    }
    Ok(())
}
