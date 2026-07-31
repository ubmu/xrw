use std::fmt;

/// An identifier mark.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Mark {
    /// Four-character code
    Four([u8; 4]),
    /// Universally unique identifier
    UUID([u8; 16]),
}

// Marks for container formats.
impl Mark {
    pub const FORM: Self = Self::Four(*b"FORM");
    pub const RIFF: Self = Self::Four(*b"RIFF");
    pub const RIFX: Self = Self::Four(*b"RIFX");
    pub const FFIR: Self = Self::Four(*b"FFIR");
    pub const XFIR: Self = Self::Four(*b"XFIR");
    pub const RF64: Self = Self::Four(*b"RF64");
    pub const BW64: Self = Self::Four(*b"BW64");
    // This format uses UUIDs, but it is easier to detect the format using this FourCC.
    // TODO: Sony Wave64 actually embeds the chunk FourCC into every UUID. Perhaps it
    // would be simpler to just use and store the FourCC's and skip the rest?
    // Would require storing the UUID equivalent as a const for writing, though.
    pub const SW64: Self = Self::Four(*b"riff");
    pub const CAFF: Self = Self::Four(*b"caff");
    pub const FTYP: Self = Self::Four(*b"ftyp");
}

// Marks for subtypes.
impl Mark {
    // :http://justsolve.archiveteam.org/wiki/IFF
    pub const _8SVX: Self = Self::Four(*b"8SVX");
    pub const ACBM: Self = Self::Four(*b"ACBM");
    pub const AIFC: Self = Self::Four(*b"AIFC");
    pub const AIFF: Self = Self::Four(*b"AIFF");
    pub const AMFF: Self = Self::Four(*b"AMFF");
    pub const ANBM: Self = Self::Four(*b"ANBM");
    pub const ANIM: Self = Self::Four(*b"ANIM");
    pub const CMUS: Self = Self::Four(*b"CMUS");
    pub const CTLG: Self = Self::Four(*b"CTLG");
    pub const D3TV: Self = Self::Four(*b"D3TV");
    pub const DEEP: Self = Self::Four(*b"DEEP");
    pub const DR2D: Self = Self::Four(*b"DR2D");
    pub const FANT: Self = Self::Four(*b"FANT");
    pub const FAX3: Self = Self::Four(*b"FAX3");
    pub const FAXX: Self = Self::Four(*b"FAXX");
    pub const FNTR: Self = Self::Four(*b"FNTR");
    pub const FNTV: Self = Self::Four(*b"FNTV");
    pub const FTXT: Self = Self::Four(*b"FTXT");
    pub const GSCR: Self = Self::Four(*b"GSCR");
    pub const ICON: Self = Self::Four(*b"ICON");
    pub const IFRS: Self = Self::Four(*b"IFRS");
    pub const IFZS: Self = Self::Four(*b"IFZS");
    pub const ILBM: Self = Self::Four(*b"ILBM");
    pub const IMAG: Self = Self::Four(*b"IMAG");
    pub const LWLO: Self = Self::Four(*b"LWLO");
    pub const LWOB: Self = Self::Four(*b"LWOB");
    pub const LWO2: Self = Self::Four(*b"LWO2");
    pub const MAUD: Self = Self::Four(*b"MAUD");
    pub const MLDF: Self = Self::Four(*b"MLDF");
    pub const PBM: Self = Self::Four(*b"PBM ");
    pub const PDEF: Self = Self::Four(*b"PDEF");
    pub const PICS: Self = Self::Four(*b"PICS");
    pub const PLBM: Self = Self::Four(*b"PLBM");
    pub const RGFX: Self = Self::Four(*b"RGFX");
    pub const SCDH: Self = Self::Four(*b"SCDH");
    pub const SMUS: Self = Self::Four(*b"SMUS");
    pub const SSA: Self = Self::Four(*b"SSA ");
    pub const SWRT: Self = Self::Four(*b"SWRT");
    pub const TDDD: Self = Self::Four(*b"TDDD");
    pub const USCR: Self = Self::Four(*b"USCR");
    pub const UVOX: Self = Self::Four(*b"UVOX");
    pub const VAXL: Self = Self::Four(*b"VAXL");
    pub const VDEO: Self = Self::Four(*b"VDEO");
    pub const WORD: Self = Self::Four(*b"WORD");

    // :http://justsolve.archiveteam.org/wiki/RIFF
    pub const ACID: Self = Self::Four(*b"ACID");
    pub const ACON: Self = Self::Four(*b"ACON");
    pub const AMV: Self = Self::Four(*b"AMV ");
    pub const AVI: Self = Self::Four(*b"AVI ");
    pub const BND: Self = Self::Four(*b"BND ");
    pub const CARA: Self = Self::Four(*b"CARA");
    pub const CARB: Self = Self::Four(*b"CARB");
    pub const CARC: Self = Self::Four(*b"CARC");
    pub const CDXA: Self = Self::Four(*b"CDXA");
    pub const CDR: Self = Self::Four(*b"CDR?");
    pub const CMX1: Self = Self::Four(*b"CMX1");
    pub const DES: Self = Self::Four(*b"DES?");
    pub const EGG: Self = Self::Four(*b"Egg!");
    pub const FGDM: Self = Self::Four(*b"FGDM");
    pub const IDF: Self = Self::Four(*b"IDF ");
    pub const LBIT: Self = Self::Four(*b"LBit");
    pub const MDH1: Self = Self::Four(*b"MDH1");
    pub const MDH2: Self = Self::Four(*b"MDH2");
    pub const MGX: Self = Self::Four(*b"MGX ");
    pub const MIDS: Self = Self::Four(*b"MIDS");
    pub const MSFX: Self = Self::Four(*b"MSFX");
    pub const NIFF: Self = Self::Four(*b"NIFF");
    pub const NUND: Self = Self::Four(*b"NUND");
    pub const OAKT: Self = Self::Four(*b"oakt");
    pub const OFM8: Self = Self::Four(*b"OFM8");
    pub const PAL: Self = Self::Four(*b"PAL ");
    pub const PAN: Self = Self::Four(*b"PAN ");
    pub const PRTP: Self = Self::Four(*b"PRTP");
    pub const RDIB: Self = Self::Four(*b"RDIB");
    pub const RMID: Self = Self::Four(*b"RMID");
    pub const RMMP: Self = Self::Four(*b"RMMP");
    pub const SEKD: Self = Self::Four(*b"SEKD");
    pub const SFBK: Self = Self::Four(*b"sfbk");
    pub const SFPJ: Self = Self::Four(*b"SFPJ");
    pub const SHW4: Self = Self::Four(*b"shw4");
    pub const SPCR: Self = Self::Four(*b"SPCR");
    pub const STYL: Self = Self::Four(*b"STYL");
    pub const TRID: Self = Self::Four(*b"TRID");
    pub const VDRM: Self = Self::Four(*b"VDRM");
    pub const WAVE: Self = Self::Four(*b"WAVE");
    pub const WEBP: Self = Self::Four(*b"WEBP");
}

// Marks for block identifiers.
impl Mark {
    pub const DS64: Self = Self::Four(*b"ds64");
    pub const DATA: Self = Self::Four(*b"data");
    pub const DESC: Self = Self::Four(*b"desc");
    pub const FMT_: Self = Self::Four(*b"fmt ");

    pub const CAT: Self = Self::Four(*b"CAT ");
    pub const LIST: Self = Self::Four(*b"LIST");
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Four(bytes) => match std::str::from_utf8(bytes) {
                Ok(four) => f.write_str(four),
                Err(_) => write!(f, "{:02X?}", bytes),
            },
            Self::UUID(bytes) => {
                write!(
                    f,
                    "{:02x}{:02x}{:02x}{:02x}-\
                     {:02x}{:02x}-\
                     {:02x}{:02x}-\
                     {:02x}{:02x}-\
                     {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                    bytes[0],
                    bytes[1],
                    bytes[2],
                    bytes[3],
                    bytes[4],
                    bytes[5],
                    bytes[6],
                    bytes[7],
                    bytes[8],
                    bytes[9],
                    bytes[10],
                    bytes[11],
                    bytes[12],
                    bytes[13],
                    bytes[14],
                    bytes[15],
                )
            }
        }
    }
}
