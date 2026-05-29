use std::fmt;

/// Block identifier markers.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Marker {
    /// Four-character code (FourCC).
    FourCC([u8; 4]),
    /// UUID or GUID.
    UUID([u8; 16]),
}

/// To initialize a unique marker, use the following format:
///
/// Marker::[TYPE](*b"...")
///
/// Example:
///
/// Marker::FourCC(*b"STYX")
impl Marker {}

/// Container marker constants.
impl Marker {
    /// Interchange File Format (IFF)
    pub const FORM: Self = Self::FourCC(*b"FORM");
    /// Resource Interchange File Format (RIFF)
    pub const RIFF: Self = Self::FourCC(*b"RIFF");
    /// RIFF big-endian variant.
    pub const RIFX: Self = Self::FourCC(*b"RIFX");
    /// Alternate RIFF big-endian variant.
    pub const FFIR: Self = Self::FourCC(*b"FFIR");
    /// Alternate RIFF big-endian variant.
    pub const XFIR: Self = Self::FourCC(*b"XFIR");
    /// RIFF 64-bit, superseded by BW64.
    pub const RF64: Self = Self::FourCC(*b"RF64");
    /// Broadcast Wave Format 64-bit.
    pub const BW64: Self = Self::FourCC(*b"BW64");
    /// Sony Wave64 uses UUIDs. The container UUID
    /// leads with an intentional FourCC. This allows
    /// for easier container detection.
    pub const SW64: Self = Self::FourCC(*b"riff");
}

/// Form-type marker constants.
impl Marker {
    /// TODO: Document these.
    pub const AIFF: Self = Self::FourCC(*b"AIFF");
    pub const AIFC: Self = Self::FourCC(*b"AIFC");
    pub const WAVE: Self = Self::FourCC(*b"WAVE");
}

/// Block marker constants.
impl Marker {
    /// 64-bit size table for RF64 and BW64 containers ('ds64').
    pub const DS64: Self = Self::FourCC(*b"ds64");
    /// Format parameters ('fmt ', note the trailing space).
    pub const FMT: Self = Self::FourCC(*b"fmt ");
    /// Audio sample data ('data').
    pub const DATA: Self = Self::FourCC(*b"data");
    /// Broadcast extension data ('bext').
    pub const BEXT: Self = Self::FourCC(*b"bext");
    // IMPORTANT: HANDLE NESTED LATER.
    pub const LIST: Self = Self::FourCC(*b"LIST");
}

impl From<[u8; 4]> for Marker {
    fn from(bytes: [u8; 4]) -> Self {
        Self::FourCC(bytes)
    }
}

impl From<[u8; 16]> for Marker {
    fn from(bytes: [u8; 16]) -> Self {
        Self::UUID(bytes)
    }
}

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
