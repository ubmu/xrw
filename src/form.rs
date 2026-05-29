use crate::{Error, Result};

use super::marker::Marker;

/// The subtype of a container, identifying the specific format of the enclosed data.
/// Also called the file-type, form-type, or kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// Waveform audio (RIFF WAVE).
    Wave,
    /// Audio Interchange File Format (IFF AIFF).
    Aiff,
    /// Audio Interchange File Format Compressed (IFF AIFF-C).
    Aifc,
}

impl TryFrom<Marker> for Form {
    type Error = Error;
    fn try_from(marker: Marker) -> Result<Self> {
        match marker {
            Marker::WAVE => Ok(Form::Wave),
            Marker::AIFF => Ok(Form::Aiff),
            Marker::AIFC => Ok(Form::Aifc),
            _ => Err(Error::UnknownForm),
        }
    }
}
