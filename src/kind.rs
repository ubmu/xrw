use crate::{Error, Marker, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    WAVE,
    AIFF,
    AIFC,
}

impl TryFrom<Marker> for Kind {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::WAVE => Ok(Kind::WAVE),
            Marker::AIFF => Ok(Kind::AIFF),
            Marker::AIFC => Ok(Kind::AIFC),
            _ => Err(Error::UnknownKind),
        }
    }
}
