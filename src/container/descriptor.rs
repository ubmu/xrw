use crate::Byteorder;

/// Describes the structural layout of a container family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    /// The byteorder or endianness.
    pub byteorder: Byteorder,
    /// The alignment boundary for blocks in bytes.
    /// `1` means no padding is required between blocks.
    pub alignment: u8,
    /// The number of header bytes included in the declared size field value.
    /// Subtracted from the size field when computing the actual payload size.
    pub header_overhead: u8,
    /// The width of the block identifier field in bytes.
    pub mark_width: u8,
    /// The width of the payload size field in bytes.
    pub size_width: u8,
}

impl Descriptor {
    /// The combined size of the identifier and size fields in bytes.
    pub const fn header_width(&self) -> u8 {
        self.mark_width + self.size_width
    }

    /// Returns the number of padding bytes required after a payload of the
    /// given size to reach the next alignment boundary.
    /// Returns `0` when `alignment == 1` or the payload is already aligned.
    pub(crate) fn padding_after(&self, payload_size: u64) -> u64 {
        let alignment = self.alignment as u64;
        let remainder = payload_size % alignment;
        if remainder != 0 { alignment - remainder } else { 0 }
    }
}

impl Descriptor {
    pub const INTER: Self = Self {
        byteorder: Byteorder::Big,
        alignment: 2,
        header_overhead: 0,
        mark_width: 4,
        size_width: 4,
    };
    pub const R_INTER: Self = Self {
        byteorder: Byteorder::Little,
        alignment: 2,
        header_overhead: 0,
        mark_width: 4,
        size_width: 4,
    };
    pub const SONY_WAVE64: Self = Self {
        byteorder: Byteorder::Little,
        alignment: 8,
        header_overhead: 24,
        mark_width: 16,
        size_width: 8,
    };
    pub const CORE_AUDIO: Self = Self {
        byteorder: Byteorder::Big,
        alignment: 1,
        header_overhead: 0,
        mark_width: 4,
        size_width: 8,
    };
    pub const BASE_MEDIA: Self = Self {
        byteorder: Byteorder::Big,
        alignment: 1,
        header_overhead: 8,
        mark_width: 4,
        size_width: 4,
    };
}
