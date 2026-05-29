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

    /// Descriptor for IFF and big-endian RIFF (RIFX) containers.
    pub const IFF: Self = Self::new(Byteorder::Big, 2, 0, 4, 4);
    /// Descriptor for RIFF, RF64, and BW64 containers.
    pub const RIFF: Self = Self::new(Byteorder::Little, 2, 0, 4, 4);
    /// Descriptor for Sony Wave64 containers.
    pub const SW64: Self = Self::new(Byteorder::Little, 8, 24, 16, 8);

    /// Returns the number of padding bytes required after a payload of the
    /// given size to reach the next alignment boundary.
    pub(crate) fn padding_after(&self, payload_size: u64) -> u64 {
        let alignment = self.alignment as u64;
        let remainder = payload_size % alignment;
        if remainder != 0 { alignment - remainder } else { 0 }
    }
}
