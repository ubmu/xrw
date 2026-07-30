mod collect;

mod descriptor;
pub(crate) use descriptor::Descriptor;

mod detect;

mod family;
pub(crate) use family::Family;

mod format;
pub use format::Format;

mod preamble;

mod read;

mod write;

use crate::{Reader, Result};
use std::fmt;
use std::io::{Read, Seek};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Container {
    format: Format,
}

impl Container {
    pub fn new(format: Format) -> Self {
        Self { format }
    }

    pub(crate) fn detect<R: Read + Seek>(reader: &mut Reader<R>) -> Result<Self> {
        Ok(Self::new(Format::detect(reader)?))
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn family(&self) -> Family {
        self.format.family()
    }

    pub(crate) fn descriptor(&self) -> Descriptor {
        self.format.descriptor()
    }
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format)
    }
}
