use std::io::{Read, Seek};

use super::io::Reader;
use crate::{Byteorder, Result};

/// Format specific data that extends the base container structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extension {
    /// 64-bit size extension for RF64 and BW64 containers.
    Ds64(Ds64),
    /// The file version and flags for CAF.
    CoreAudioHeader(CoreAudioHeader),
}

/// The `ds64` chunk, required in RF64 and BW64 files.
///
/// Stores the true 64-bit sizes of chunks whose size fields are set to [`u32::MAX`],
/// which is used as a sentinel value to indicate that the real size exceeds 32 bits.
///
/// EBU Tech 3306-2007
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ds64 {
    pub offset: u64,
    pub size: u32,
    /// True size of the RIFF container, replacing the outer header size field.
    pub riff_size: u64,
    /// True size of the `data` chunk payload.
    pub data_size: u64,
    /// True sample count, replacing the value in the `fact` chunk.
    pub sample_count: u64,
}

impl Ds64 {
    pub(crate) fn read<R: Read + Seek>(
        reader: &mut Reader<R>,
        byteorder: Byteorder,
    ) -> Result<Self> {
        // Account for the marker read.
        let offset = reader.tell()? - 4;
        let size = reader.read_u32(byteorder)?;
        let riff_size = reader.read_u64(byteorder)?;
        let data_size = reader.read_u64(byteorder)?;
        let sample_count = reader.read_u64(byteorder)?;
        let table_length = reader.read_u32(byteorder)?;
        if table_length > 0 {
            reader.skip(table_length as u64 * 12)?;
        }
        Ok(Self {
            offset,
            size,
            riff_size,
            data_size,
            sample_count,
        })
    }
}

/// The header for CAF.
///
/// Apple Core Audio Format Specification 1.0: https://developer.apple.com/library/archive/documentation/MusicAudio/Reference/CAFSpec/CAF_spec/CAF_spec.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreAudioHeader {
    // The file version. For CAF files conforming to this specification, the version must be set to 1.
    pub file_version: u16,
    // Flags reserved by Apple for future use. For CAF v1 files, must be set to 0.
    pub file_flags: u16,
}
