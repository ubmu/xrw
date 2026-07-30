use super::{Descriptor, Family};
use crate::Mark;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Inter,
    ResourceInter,
    ResourceInterBig,
    ResourceInter64,
    SonyWave64,
    CoreAudio,
    BaseMedia,
}

struct FormatInfo {
    family: Family,
    descriptor: Descriptor,
    short_name: &'static str,
    long_name: &'static str,
}

impl Format {
    const fn info(&self) -> FormatInfo {
        match self {
            Self::Inter => FormatInfo {
                family: Family::Inter,
                descriptor: Descriptor::INTER,
                short_name: "IFF",
                long_name: "Interchange File Format",
            },
            Self::ResourceInter => FormatInfo {
                family: Family::Inter,
                descriptor: Descriptor::R_INTER,
                short_name: "RIFF",
                long_name: "Resource Interchange File Format",
            },
            Self::ResourceInterBig => FormatInfo {
                family: Family::Inter,
                descriptor: Descriptor::INTER,
                short_name: "RIFX",
                long_name: "Resource Interchange File Format (big-endian)",
            },
            Self::ResourceInter64 => FormatInfo {
                family: Family::Inter,
                descriptor: Descriptor::R_INTER,
                short_name: "RF64/BW64",
                long_name: "64-bit Resource Interchange File Format",
            },
            Self::SonyWave64 => FormatInfo {
                family: Family::Inter,
                descriptor: Descriptor::SONY_WAVE64,
                short_name: "Wave64",
                long_name: "Sony Wave64",
            },
            Self::CoreAudio => FormatInfo {
                family: Family::CoreAudio,
                descriptor: Descriptor::CORE_AUDIO,
                short_name: "CAF",
                long_name: "Core Audio Format",
            },
            Self::BaseMedia => FormatInfo {
                family: Family::BaseMedia,
                descriptor: Descriptor::BASE_MEDIA,
                short_name: "ISOBMFF",
                long_name: "ISO Base Media File Format",
            },
        }
    }

    pub(crate) fn family(&self) -> Family {
        self.info().family
    }

    pub(crate) fn descriptor(&self) -> Descriptor {
        self.info().descriptor
    }

    pub(crate) const fn short_name(&self) -> &'static str {
        self.info().short_name
    }

    pub(crate) const fn long_name(&self) -> &'static str {
        self.info().long_name
    }

    pub(crate) const fn contains_nesting_inter(&self, mark: Mark) -> bool {
        todo!();
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[format: {} - {}]", self.short_name(), self.long_name(),)
    }
}
