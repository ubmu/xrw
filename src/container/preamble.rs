use super::{Container, Descriptor, Format};
use crate::{CoreAudioHeader, Ds64, Error, Extension, Mark, Reader, Result};
use std::io::{Read, Seek};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Preamble {
    pub(super) subtype: Option<Mark>,
    pub(super) size: u64,
    pub(super) extension: Option<Extension>,
}

impl Container {
    pub(super) fn read_preamble_inter<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
    ) -> Result<Preamble> {
        let descriptor = self.descriptor();

        let master = reader.read_mark(&descriptor)?;
        let mut size = reader.read_size(&descriptor)?;
        let subtype = reader.read_mark(&descriptor)?;

        let extension = match master {
            Mark::RF64 | Mark::BW64 => {
                let ds64 = Self::read_extension_ds64(reader, &descriptor)?;
                if size == u32::MAX as u64 {
                    size = ds64.riff_size;
                }
                Some(Extension::Ds64(ds64))
            }
            _ => None,
        };

        let size = if self.format() == Format::SonyWave64 { size } else { size + 8 };

        Ok(Preamble { subtype: Some(subtype), size, extension })
    }

    pub(super) fn read_preamble_core<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
    ) -> Result<Preamble> {
        let descriptor = self.descriptor();

        let _file_type = reader.read_mark(&descriptor)?;
        let file_version = reader.read_u16(descriptor.byteorder)?;
        let file_flags = reader.read_u16(descriptor.byteorder)?;

        let extension =
            Extension::CoreAudioHeader(CoreAudioHeader { file_version, file_flags });
        let size = reader.size();

        Ok(Preamble { subtype: None, size, extension: Some(extension) })
    }

    pub(super) fn read_preamble_base<R: Read + Seek>(
        &self,
        reader: &mut Reader<R>,
    ) -> Result<Preamble> {
        todo!()
    }

    fn read_extension_ds64<R: Read + Seek>(
        reader: &mut Reader<R>,
        descriptor: &Descriptor,
    ) -> Result<Ds64> {
        let mark = reader.read_mark(descriptor)?;
        if mark != Mark::DS64 {
            return Err(Error::Read(format!(
                "expected a 'ds64' chunk, found '{mark}'"
            )));
        }
        Ds64::read(reader, descriptor.byteorder)
    }
}
