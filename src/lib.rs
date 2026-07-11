/*!
A general editor library for structured binary formats.

### Related
- [`umedia`](https://github.com/ubmu/umedia) — Extensive multimedia metadata parsing and editing.
*/
#![warn(clippy::pedantic)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod assembler;
mod block;
mod container;
mod descriptor;
mod extension;
mod io;
mod layout;
mod marker;
mod opts;
mod parser;

pub use block::{Block, Source};
pub use container::Container;
pub use descriptor::Descriptor;
pub use extension::{Ds64, Extension};
pub use io::{Reader, Writer};
pub use layout::Layout;
pub use marker::Marker;
pub use opts::{ReadOptions, WriteOptions};

pub mod prelude {
    pub use super::{Block, Layout, Marker, ReadOptions, Reader, WriteOptions, Writer};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Byteorder {
    Big,
    Little,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // parsing/probing errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("A container could not be detected. The provided source is malformed or unsupported.")]
    UnknownContainer,
    #[error("An unsupported marker width was provided: {marker_width}")]
    UnsupportedMarkerWidth { marker_width: u8 },
    #[error("An ununsupported size width was provided: {size_width}")]
    UnsupportedSizeWidth { size_width: u8 },
    #[error("A size field was negative when there is zero reason it should at: {offset}")]
    NegativeSize { offset: u64 },
    #[error("The [`ds64`] chunk is required immediately after the RF64/BW64 master header: {got}")]
    MissingDs64 { got: Marker },
    #[error("Invalid block size {size} at {offset}")]
    InvalidBlockSize { size: u64, offset: u64 },

    // CAF errors that can occur when auto_fix is not set during writing.
    #[error("Invalid file type for CAF: expected {expected} - got {got}")]
    InvalidFileType { expected: Marker, got: Marker },
    #[error("Invalid file version for CAF: expected {expected} - got {got}")]
    InvalidFileVersion { expected: u8, got: u16 },
    #[error("['desc'] must be the first block for CAF: expected {expected} - got {got}")]
    InvalidFirstBlock { expected: Marker, got: Marker },

    // General writing errors.
    #[error("A subtype is missing. This is required for writing certain containers.")]
    MissingSubtype,
    #[error("Container size exceeds `u32::MAX` and `auto_promote` is disabled")]
    SizeOverflow,
    #[error(
        "The `Layout` contains `Source::Original` blocks which require a `Reader`. Try `write_from` instead."
    )]
    WrongWriteFunction,
}

pub type Result<T> = std::result::Result<T, Error>;
