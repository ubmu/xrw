use crate::Byteorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerWidth {
    FourCC,
    UUID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeWidth {
    U32,
    U64,
}

/// Describes the structural layout of a container family.
///
/// The descriptor defines the properties required for reading any structured binary format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    /// The format byte-order.
    pub byteorder: Byteorder,
    /// The byte-boundary that the format aligns blocks to.
    /// For example, IFF-based formats align chunks to any even byte-boundary.
    pub block_alignment: u8,
    /// The number of block header bytes included in the size field value.
    /// This is subtracted when reading the payload size.
    pub header_overhead: u8,
    /// The size of the identifier field in bytes.
    pub width_marker: MarkerWidth,
    /// The size of the payload size field in bytes.
    pub width_payload_size: SizeWidth,
}

impl Descriptor {
    pub const INTERCHANGE: Self = Self {
        byteorder: Byteorder::Big,
        block_alignment: 2,
        header_overhead: 0,
        width_marker: MarkerWidth::FourCC,
        width_payload_size: SizeWidth::U32,
    };

    pub const RESOURCE_INTERCHANGE: Self = Self {
        byteorder: Byteorder::Little,
        ..Self::INTERCHANGE
    };

    pub const WAVE_64: Self = Self {
        byteorder: Byteorder::Little,
        block_alignment: 8,
        header_overhead: 24,
        width_marker: MarkerWidth::UUID,
        width_payload_size: SizeWidth::U64,
    };
}
