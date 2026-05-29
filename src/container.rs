use std::fmt;
use std::io::{Read, Seek};

use super::descriptor::Descriptor;
use super::io::Reader;
use super::marker::Marker;
use crate::{Error, Result};

/// The container format of a structured binary file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// Interchange File Format
    IFF,
    /// Resource Interchange File Format
    RIFF,
    /// RIFF, any big-endian variant
    RIFX,
    /// RIFF 64-bit
    RF64,
    /// Sony Wave64
    SW64,
}

impl Container {
    /// Detects the container format by reading the stream from the beginning.
    /// Returns [`Error::UnknownContainer`] if the format cannot be identified.
    pub(crate) fn detect<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        if let Ok(container) = Self::try_detect_interchange(reader) {
            return Ok(container);
        }

        // Add other try_detect_ functions as they come.

        Err(Error::UnknownContainer)
    }

    /// Attempts to identify an interchange format variant.
    fn try_detect_interchange<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        reader.seek(0)?;
        let marker = Marker::from(reader.read_property_code()?);
        Self::try_from(marker)
    }
}

impl From<&Container> for Descriptor {
    fn from(container: &Container) -> Self {
        match container {
            Container::IFF | Container::RIFX => Descriptor::IFF,
            Container::RIFF | Container::RF64 => Descriptor::RIFF,
            Container::SW64 => Descriptor::SW64,
        }
    }
}

impl TryFrom<Marker> for Container {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::FourCC(b) => match &b {
                b"FORM" => Ok(Container::IFF),
                b"RIFF" => Ok(Container::RIFF),
                b"RIFX" | b"FFIR" | b"XFIR" => Ok(Container::RIFX),
                b"RF64" | b"BW64" => Ok(Container::RF64),
                b"riff" => Ok(Container::SW64),

                _ => Err(Error::UnknownContainer),
            },
            _ => Err(Error::UnknownContainer),
        }
    }
}

// TODO: figure out how I want to display container
// Probably use long-form names.
impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Container::IFF => write!(f, "IFF"),
            Container::RIFF => write!(f, "RIFF"),
            Container::RIFX => write!(f, "RIFX"),
            Container::RF64 => write!(f, "RF64"),
            Container::SW64 => write!(f, "SW64"),
        }
    }
}
