#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Mark {
    /// Four-character code
    Four([u8; 4]),
    /// Universally unique identifier
    UUID([u8; 16]),
}
