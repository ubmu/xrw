use crate::Byteorder;

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
    /// The size of the header. This is just `marker_width` + `size_width`.
    pub header_width: u8,
    /// The size of the identifier field in bytes.
    pub marker_width: u8,
    /// The size of the payload size field in bytes.
    pub size_width: u8,
}

impl Descriptor {
    pub const fn new(
        byteorder: Byteorder,
        block_alignment: u8,
        header_overhead: u8,
        marker_width: u8,
        size_width: u8,
    ) -> Self {
        Self {
            byteorder,
            block_alignment,
            header_overhead,
            header_width: marker_width + size_width,
            marker_width,
            size_width,
        }
    }

    // Self::new(Byteorder, block_alignment, header_overhead, marker_width, size_width)
    pub const IFF: Self = Self::new(Byteorder::Big, 2, 0, 4, 4);
    pub const RIFF: Self = Self::new(Byteorder::Little, 2, 0, 4, 4);
    pub const SW64: Self = Self::new(Byteorder::Little, 8, 24, 16, 8);
}
