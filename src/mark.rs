pub enum Mark {
    /// Four-Character Code
    Four([u8; 4]),
    /// Universally Unique Identifier
    UUID([u8; 16]),
}
