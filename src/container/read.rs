use super::{Container, Family, preamble::Preamble};
use crate::{Block, Layout, ReadOptions, Reader, Result, Source};
use std::io::{Read, Seek};

impl Container {
    pub(crate) fn read_into_layout(
        &self,
        layout: &mut Layout,
        opts: &ReadOptions,
    ) -> Result<()> {
        let reader = match &mut layout.source {
            Source::Bound(reader) => reader,
            _ => unreachable!(),
        };

        reader.seek(0)?;

        let preamble = self.read_preamble(reader)?;
        let blocks = self.collect_blocks(reader, &preamble, opts)?;

        layout.blocks = blocks;
        layout.container = *self;
        layout.subtype = preamble.subtype;
        layout.size = preamble.size;
        layout.extension = preamble.extension;

        Ok(())
    }

    fn read_preamble<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
    ) -> Result<Preamble> {
        match self.family() {
            Family::Inter => self.read_preamble_inter(reader),
            Family::CoreAudio => self.read_preamble_core(reader),
            Family::BaseMedia => self.read_preamble_base(reader),
        }
    }

    fn collect_blocks<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
        preamble: &Preamble,
        opts: &ReadOptions,
    ) -> Result<Vec<Block>> {
        match self.family() {
            Family::Inter => self.collect_blocks_inter(reader, preamble, opts),
            Family::CoreAudio => self.collect_blocks_core(reader, preamble, opts),
            Family::BaseMedia => self.collect_blocks_base(reader, preamble, opts),
        }
    }
}
