/// Options controlling how a `Layout` is parsed from a file.
#[derive(Debug, Clone, Copy)]
pub struct ReadOptions {
    /// Skips duplicate block identifiers when indexing the file.
    ///
    /// When enabled, only the first occurrence of each block identifier is
    /// indexed. Subsequent duplicates are ignored.
    ///
    /// Defaults to `false`.
    pub skip_duplicates: bool,

    /// Assumes the file strictly follows the format's alignment requirements.
    ///
    /// When enabled, the parser skips the expected alignment padding without verifying
    /// that the padding bytes are present and contain the format's required padding bytes.
    ///
    /// When disabled, the parser verifies the expected padding bytes before
    /// skipping them. If the expected padding is not present, parsing continues
    /// as though the chunk were written without padding.
    ///
    /// Disable this option when attempting to probe files that are not padded correctly.
    ///
    /// Defaults to `true`.
    pub assume_strict_alignment: bool,

    /// Validates the minimum payload size for known block identifiers.
    ///
    /// When enabled, payloads for known block identifiers must meet their
    /// minimum expected size.
    ///
    /// When disabled, only basic bounds checking is performed: the payload
    /// must be non-zero in size and fit within the file.
    ///
    /// Defaults to `true`.
    pub validate_minimum_payload_size: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            skip_duplicates: false,
            assume_strict_alignment: false,
            validate_minimum_payload_size: true,
        }
    }
}

/// Options controlling how a `Layout` is written to a file.
#[derive(Debug, Clone, Copy)]
pub struct WriteOptions {
    /// Automatically repairs specification violations before writing.
    ///
    /// For example, if a probed CAF file has an invalid `file_version` (see `Extension::CoreAudioHeader`),
    /// enabling this option updates it to the required value before the file is written.
    ///
    /// When disabled, the writer will not repair any specification violations. Since the writer normally only
    /// produces specification-compliant files, attempting to write an invalid `Layout` will result in an error
    /// unless `allow_specification_violations` is also enabled.
    ///
    /// Defaults to `true`.
    pub auto_fix: bool,

    /// Allows writing files that violate the format specification.
    ///
    /// By default, the writer validates the `Layout` and refuses to output a file that is not specification compliant.
    ///
    /// This option only has an effect when `auto_fix` is disabled. When enabled, the writer skips validation and writes
    /// the `Layout` in its provided state, regardless of if this produces a non-compliant file.
    ///
    /// This is primarily intended for generating test cases or reproducing malformed files.
    ///
    /// Defaults to `false`.
    pub allow_specification_violations: bool,
}
