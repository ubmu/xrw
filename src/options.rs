/// Options for controlling how a structure is read from a file.
#[derive(Debug, Clone, Copy)]
pub struct ReadOptions {
    /// When `true`, only the first occurrence of each block identifier is indexed.
    /// Subsequent duplicates are skipped. Defaults to `false`.
    pub skip_duplicates: bool,
    /// When `true`, strictly enforces block alignment boundaries without validation and assumes
    /// all chunks are correctly padded. When `false`, peeks at the expected padding byte to verify
    /// it is a null byte before skipping, adding minor overhead but ensuring block indexing remains
    /// successful even when chunks are not correctly padded. Defaults to `true`.
    pub strict_alignment: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            skip_duplicates: false,
            strict_alignment: true,
        }
    }
}

/// Options for controlling how a structure is written to a file.
#[derive(Debug, Clone, Copy)]
pub struct WriteOptions {}
