use crate::{Error, Marker, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    WAVE,
}

impl TryFrom<Marker> for Kind {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::WAVE => Ok(Kind::WAVE),
            _ => Err(Error::UnknownKind),
        }
    }
}
