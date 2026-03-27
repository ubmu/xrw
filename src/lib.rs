/*!
Structural I/O for binary formats.

# Overview

...

A helper function is provided  for reading a block's raw payload
by seeking to its stored offset.

*/

pub mod error;
pub mod read;
pub mod structure;
pub mod types;

//pub use error::{Error, ErrorKind};
pub use read::Reader;
pub mod xrw;
use thiserror::Error;
//pub use types::{Byteorder, Container, Kind, Marker};

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unknown container")]
    UnknownContainer,

    #[error("unexpected end of stream")]
    UnexpectedEof,
}

pub type Result<T> = std::result::Result<T, Error>;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Byteorder {
    Big,
    Little,
}
