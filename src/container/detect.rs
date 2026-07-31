use super::Format;
use crate::{Error, Mark, Reader, Result};
use std::io::{Read, Seek};

impl Format {
    pub(crate) fn detect<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        if let Ok(format) = Self::detect_inter(reader) {
            return Ok(format);
        }
        if let Ok(format) = Self::detect_core_audio(reader) {
            return Ok(format);
        }
        Err(Error::Detect("failed to detect container format".into()))
    }

    fn detect_inter<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        reader.seek(0)?;
        match Mark::Four(reader.read_code()?) {
            Mark::FORM => Ok(Self::Inter),
            Mark::RIFF => Ok(Self::ResourceInter),
            Mark::RIFX => Ok(Self::ResourceInterBig),
            Mark::RF64 | Mark::BW64 => Ok(Self::ResourceInter64),
            Mark::SW64 => Ok(Self::SonyWave64),
            mark => Err(Error::Detect(format!(
                "not an interchange master marker: {mark}"
            ))),
        }
    }

    fn detect_core_audio<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        reader.seek(0)?;
        match Mark::Four(reader.read_code()?) {
            Mark::CAFF => Ok(Self::CoreAudio),
            mark => Err(Error::Read(format!("not a CAF marker: {mark}"))),
        }
    }

    // TODO: detect_base_media, and Sony Wave64 moved to detect_inter as all UUIDs have the original chunk identifier as the first four bytes.
}
