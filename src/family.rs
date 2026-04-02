use crate::{Descriptor, Error, Marker, Result};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// Interchange file format (EA IFF 85)
    IFF,
    /// Resource interchange file format (RIFF)
    RIFF,
    /// Big-endian variant of RIFF (RIFX, FFIR, XFIR)
    RIFX,
    /// 64-bit variant of RIFF
    RF64,
    /// Sony Wave64, 64-bit variant of RIFF
    SW64,
}

impl TryFrom<&Family> for Descriptor {
    type Error = Error;

    fn try_from(family: &Family) -> Result<Self> {
        match family {
            Family::IFF | Family::RIFX => Ok(Descriptor::IFF),
            Family::RIFF | Family::RF64 => Ok(Descriptor::RIFF),
            Family::SW64 => Ok(Descriptor::SW64),
        }
    }
}

impl TryFrom<Marker> for Family {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::FourCC(b) => match &b {
                b"FORM" => Ok(Family::IFF),
                b"RIFF" => Ok(Family::RIFF),
                b"RIFX" | b"FFIR" | b"XFIR" => Ok(Family::RIFX),
                b"RF64" | b"BW64" => Ok(Family::RF64),
                b"riff" => Ok(Family::SW64),

                _ => Err(Error::UnknownFamily),
            },
            _ => Err(Error::UnknownFamily),
        }
    }
}

// TODO: figure out how I want to display family
impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Family::IFF => write!(f, "IFF"),
            Family::RIFF => write!(f, "RIFF"),
            Family::RIFX => write!(f, "RIFX"),
            Family::RF64 => write!(f, "RF64"),
            Family::SW64 => write!(f, "SW64"),
        }
    }
}
