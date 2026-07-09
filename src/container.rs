use std::fmt;
use std::io::{Read, Seek};

use super::descriptor::Descriptor;
use super::io::Reader;
use super::marker::Marker;
use crate::{Error, Result};

/// The container format of a structured binary file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    Interchange,
    ResourceInterchange,
    /// RIFF, any big-endian variant.
    ResourceInterchangeBE,
    /// 64-bit RIFF.
    ResourceInterchange64,
    SonyWave64,
    CoreAudio,
}

impl Container {
    /// Detects the container format by using file signatures.
    /// Returns [`Error::UnknownContainer`] if the format cannot be identified.
    pub(crate) fn detect<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        if let Ok(container) = Self::try_detect_interchange(reader) {
            return Ok(container);
        }

        if let Ok(container) = Self::try_detect_coreaudio(reader) {
            return Ok(container);
        }

        Err(Error::UnknownContainer)
    }

    /// Attempts to identify an interchange format variant.
    fn try_detect_interchange<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        reader.seek(0)?;
        let marker = Marker::from(reader.read_property_code()?);
        Self::try_from(marker)
    }

    /// Attempts to identify Core Audio Format.
    fn try_detect_coreaudio<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        reader.seek(0)?;
        let marker = Marker::from(reader.read_property_code()?);
        Self::try_from(marker)
    }
}

impl From<&Container> for Descriptor {
    fn from(container: &Container) -> Self {
        match container {
            Container::Interchange | Container::ResourceInterchangeBE => Descriptor::INTERCHANGE,
            Container::ResourceInterchange | Container::ResourceInterchange64 => {
                Descriptor::RESOURCE_INTERCHANGE
            }
            Container::SonyWave64 => Descriptor::SONY_WAVE64,
            Container::CoreAudio => Descriptor::CORE_AUDIO,
        }
    }
}

impl TryFrom<Marker> for Container {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::FourCC(b) => match &b {
                b"FORM" => Ok(Container::Interchange),
                b"RIFF" => Ok(Container::ResourceInterchange),
                b"RIFX" | b"FFIR" | b"XFIR" => Ok(Container::ResourceInterchangeBE),
                b"RF64" | b"BW64" => Ok(Container::ResourceInterchange64),
                b"riff" => Ok(Container::SonyWave64),
                b"caff" => Ok(Container::CoreAudio),

                _ => Err(Error::UnknownContainer),
            },
            _ => Err(Error::UnknownContainer),
        }
    }
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Container::Interchange => write!(f, "Interchange File Format"),
            Container::ResourceInterchange => write!(f, "Resource Interchange File Format"),
            Container::ResourceInterchangeBE => {
                write!(f, "Resource Interchange File Format (big-endian)")
            }
            Container::ResourceInterchange64 => {
                write!(f, "Resource Interchange File Format (64-bit)")
            }
            Container::SonyWave64 => write!(f, "Sony Wave64"),
            Container::CoreAudio => write!(f, "Core Audio Format"),
        }
    }
}
