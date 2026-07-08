use crate::Byteorder;

/// Describes the structural layout of a format family.
///
/// The descriptor defines the properties required for reading any structured binary format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    /// The format byte-order.
    pub byteorder: Byteorder,
    /// The boundary that the format aligns blocks to.
    /// For example, IFF-based formats align chunks to any even boundary.
    pub alignment: u8,
    /// The number of block header bytes included in the size field value.
    /// This is subtracted when reading the payload size.
    pub header_overhead: u8,
    /// The size of the header. This is just `marker_width` + `size_width`.
    pub header_width: u8,
    /// The size of the identifier field in bytes.
    pub marker_width: u8,
    /// The size of the payload size field in bytes.
    pub size_width: u8,
}

/// The byte-boundary to align blocks to. For example, RIFF files must align to even byte-boundaries.
/// For the case of a block payload ending at offset 139, a padding byte is needed to align to said boundary.
mod align {
    /// No padding required.
    pub const NONE: u8 = 1;
    /// Pad to 2-byte boundaries.
    pub const EVEN: u8 = 2;
    /// Pad to 8-byte boundary.
    pub const EIGHT: u8 = 8;
}

/// The size of the marker field.
mod marker {
    pub const FOURCC: u8 = 4;
    pub const UUID: u8 = 16;
}

/// The amount of bytes needed to read the size field.
mod size {
    pub const U32: u8 = 4;
    pub const U64: u8 = 8;
}

/// The amount of bytes included in the declared size that is unrelated to the payload.
/// Several formats store the block size rather than the payload size.
mod overhead {
    /// Zero overhead: the size field accurately describes the size of the block payload.
    pub const NONE: u8 = 0;
    /// 8-byte overhead: the size field contains 8 bytes of overhead.
    /// For ISOBMFF, this is due to the 4-byte type and 4-byte size field being included in the declared size.
    pub const EIGHT: u8 = 8;
    /// 24-byte overhead: the size field contains 24 bytes of overhead.
    /// For Sony Wave64, this is due to the 16-byte UUID and 8-byte size field being included in the declared size.
    pub const TWENTY_FOUR: u8 = 24;
}

impl Descriptor {
    pub const fn new(
        byteorder: Byteorder,
        alignment: u8,
        header_overhead: u8,
        marker_width: u8,
        size_width: u8,
    ) -> Self {
        Self {
            byteorder,
            alignment,
            header_overhead,
            header_width: marker_width + size_width,
            marker_width,
            size_width,
        }
    }

    /// Descriptor for IFF and big-endian RIFF variants.
    pub const INTERCHANGE: Self = Self::new(
        Byteorder::Big,
        align::EVEN,
        overhead::NONE,
        marker::FOURCC,
        size::U32,
    );
    /// Descriptor for RIFF, RF64, and BW64.
    pub const RESOURCE_INTERCHANGE: Self = Self::new(
        Byteorder::Little,
        align::EVEN,
        overhead::NONE,
        marker::FOURCC,
        size::U32,
    );
    /// Descriptor for Sony Wave64.
    pub const SONY_WAVE64: Self = Self::new(
        Byteorder::Little,
        align::EIGHT,
        overhead::TWENTY_FOUR,
        marker::UUID,
        size::U64,
    );

    // Descriptor for Core Audio Format.
    pub const CORE_AUDIO: Self = Self::new(
        Byteorder::Big,
        align::NONE,
        overhead::NONE,
        marker::FOURCC,
        size::U32,
    );

    /// Returns the number of padding bytes required after a payload of the
    /// given size to reach the next alignment boundary.
    pub(crate) fn padding_after(&self, payload_size: u64) -> u64 {
        let alignment = self.alignment as u64;
        let remainder = payload_size % alignment;
        if remainder != 0 { alignment - remainder } else { 0 }
    }
}
