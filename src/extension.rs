/// Format specific data that extends the base container structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extension {
    /// 64-bit size extension for RF64 and BW64 containers.
    Ds64(Ds64),
}

/// The `ds64` chunk, required in RF64 and BW64 files.
///
/// Stores the true 64-bit sizes of chunks whose size fields are set to [`u32::MAX`],
/// which is used as a sentinel value to indicate that the real size exceeds 32 bits.
///
/// EBU Tech 3306-2007
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ds64 {
    pub offset: u64,
    pub size: u32,
    /// True size of the RIFF container, replacing the outer header size field.
    pub riff_size: u64,
    /// True size of the `data` chunk payload.
    pub data_size: u64,
    /// True sample count, replacing the value in the `fact` chunk.
    pub sample_count: u64,
}
