//! A library for querying structured binary formats and performing modifications.
//!
//! This crate provides an abstraction over structured binary container formats,
//! allowing their contents to be queried and modified through a common interface
//! without requiring specific details on the file format.
#![warn(clippy::pedantic)]

mod block;
pub use block::{Block, Type};

mod container;
pub(crate) use container::Descriptor;
pub use container::{Container, Format};

mod extension;
pub use extension::{CoreAudioHeader, Ds64, Extension};

mod io;
pub(crate) use io::{DynReader, Reader, Writer};

mod layout;
pub use layout::Layout;
pub(crate) use layout::Source;

mod mark;
pub use mark::Mark;

mod options;
pub use options::{ReadOptions, WriteOptions};

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Byteorder {
    Big,
    Little,
}

// TODO: GO BACK TO thiserror
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Detect(String),
    Read(String),
    Preamble(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Detect(message) => write!(f, "{message}"),
            Self::Read(message) => write!(f, "{message}"),
            Self::Preamble(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
