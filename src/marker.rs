use crate::{Error, Result};
use std::fmt;

/// A block identifier marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Four-character code (FourCC).
    FourCC([u8; 4]),
    /// UUID or GUID.
    UUID([u8; 16]),
}

impl Marker {
    //
    pub const RIFF: Self = Self::FourCC(*b"RIFF");
    pub const RIFX: Self = Self::FourCC(*b"RIFX");
    pub const FFIR: Self = Self::FourCC(*b"FFIR");
    pub const RF64: Self = Self::FourCC(*b"RF64");
    pub const BW64: Self = Self::FourCC(*b"BW64");
    /// Sony Wave64 stores the FourCC in the first four bytes of its UUID identifier,
    /// allowing detection without reading the full UUID.
    pub const SW64: Self = Self::FourCC(*b"riff");

    // TODO: Figure out a general doc line for these.

    pub const WAVE: Self = Self::FourCC(*b"WAVE");
    // Markers for RIFF chunks.
    pub const DS64: Self = Self::FourCC(*b"ds64");
    pub const FMT: Self = Self::FourCC(*b"fmt ");
    pub const FMT_: Self = Self::FourCC(*b"fmt ");
    pub const DATA: Self = Self::FourCC(*b"data");
    pub const BEXT: Self = Self::FourCC(*b"bext");
    pub const LIST: Self = Self::FourCC(*b"LIST");

    pub fn minimum_payload_size(&self) -> u64 {
        match self {
            &Self::FMT => 16,
            // TODO: See if there is a better default value.
            // Set to 1 as we check if payload size is less than minimum size.
            _ => 1,
        }
    }
}

impl TryFrom<[u8; 4]> for Marker {
    type Error = Error;
    fn try_from(bytes: [u8; 4]) -> Result<Self> {
        Ok(Marker::FourCC(bytes))
    }
}

// impl TryFrom<[u8; 16]> for Marker {}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Marker::FourCC(bytes) => match std::str::from_utf8(bytes) {
                Ok(s) => write!(f, "{}", s),
                Err(_) => write!(f, "{:08X}", u32::from_le_bytes(*bytes)),
            },
            Marker::UUID(bytes) => {
                write!(
                    f,
                    "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
                    u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                    u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
                    u16::from_le_bytes(bytes[6..8].try_into().unwrap()),
                    u16::from_be_bytes(bytes[8..10].try_into().unwrap()),
                    {
                        let mut buf = [0u8; 8];
                        buf[2..].copy_from_slice(&bytes[10..16]);
                        u64::from_be_bytes(buf)
                    }
                )
            }
        }
    }
}
