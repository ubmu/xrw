/// The `ds64` chunk, required by RF64 and BW64 files.
///
/// Stores the true 64-bit sizes of chunks whose size fields are set to [`u32::MAX`],
/// which is used as a sentinel value to indicate that the real size exceeds 32 bits.
///
/// EBU Tech 3306-2007
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSize64 {
    pub _offset: u64,
    /// Size of the `ds64` chunk payload in bytes.
    pub _size: u32,
    /// True size of the RIFF container, replacing the outer header size field.
    pub riff_size: u64,
    /// True size of the `data` chunk payload.
    pub data_size: u64,
    /// True sample count, replacing the value in the `fact` chunk.
    pub sample_count: u64,
}
