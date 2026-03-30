use crate::{Descriptor, Error, Marker, Result};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// Interchange file format (EA IFF 85)
    Interchange,
    /// Resource interchange file format (RIFF)
    ResourceInterchange,
    /// Big-endian variant of RIFF
    ResourceInterchangeX,
    /// 64-bit variant of RIFF
    ResourceInterchange64,
    /// Sony Wave64, 64-bit variant of RIFF
    Wave64,
}

impl TryFrom<&Family> for Descriptor {
    type Error = Error;

    fn try_from(family: &Family) -> Result<Self> {
        match family {
            Family::Interchange => Ok(Descriptor::INTERCHANGE),
            Family::ResourceInterchange => Ok(Descriptor::RESOURCE_INTERCHANGE),
            Family::ResourceInterchangeX => Ok(Descriptor::INTERCHANGE),
            Family::ResourceInterchange64 => Ok(Descriptor::RESOURCE_INTERCHANGE),
            Family::Wave64 => Ok(Descriptor::WAVE_64),
        }
    }
}

impl TryFrom<Marker> for Family {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::FourCC(b) => match &b {
                b"FORM" => Ok(Family::Interchange),
                b"RIFF" => Ok(Family::ResourceInterchange),
                b"RIFX" => Ok(Family::ResourceInterchangeX),
                b"RF64" => Ok(Family::ResourceInterchange64),
                b"BW64" => Ok(Family::ResourceInterchange64),
                b"riff" => Ok(Family::Wave64),
                _ => Err(Error::UnknownContainer),
            },
            _ => Err(Error::UnknownContainer),
        }
    }
}

// TODO: figure out how I want to display family
impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Family::Interchange => write!(f, "IFF"),
            Family::ResourceInterchange => write!(f, "RIFF"),
            Family::ResourceInterchangeX => write!(f, "RIFX"),
            Family::ResourceInterchange64 => write!(f, "RF64"),
            Family::Wave64 => write!(f, "Wave64"),
        }
    }
}
