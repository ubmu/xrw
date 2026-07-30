use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::{Byteorder, Descriptor, Error, Mark, Result};

pub struct Reader<R: Read + Seek> {
    inner: BufReader<R>,
    size: u64,
}

pub struct Writer<W: Write + Seek> {
    inner: BufWriter<W>,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let size = inner.seek(SeekFrom::End(0))?;
        inner.seek(SeekFrom::Start(0))?;
        Ok(Self { inner: BufReader::new(inner), size })
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn tell(&mut self) -> Result<u64> {
        Ok(self.inner.seek(SeekFrom::Current(0))?)
    }

    pub fn seek(&mut self, offset: u64) -> Result<u64> {
        Ok(self.inner.seek(SeekFrom::Start(offset))?)
    }

    pub fn skip(&mut self, bytes: u64) -> Result<u64> {
        Ok(self.inner.seek(SeekFrom::Current(bytes as i64))?)
    }

    pub fn rewind(&mut self, bytes: u64) -> Result<u64> {
        Ok(self.inner.seek(SeekFrom::Current(-(bytes as i64)))?)
    }

    pub fn seek_end(&mut self, offset: u64) -> Result<u64> {
        Ok(self.inner.seek(SeekFrom::End(-(offset as i64)))?)
    }

    fn read_exact_buf(&mut self, buf: &mut [u8]) -> Result<()> {
        Ok(self.inner.read_exact(buf)?)
    }

    pub fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; size];
        self.read_exact_buf(&mut buf)?;
        Ok(buf)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0u8; 1];
        self.read_exact_buf(&mut buf)?;
        Ok(buf[0] as i8)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact_buf(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_i16(&mut self, byte_order: Byteorder) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact_buf(&mut buf)?;
        Ok(match byte_order {
            Byteorder::Big => i16::from_be_bytes(buf),
            Byteorder::Little => i16::from_le_bytes(buf),
        })
    }

    pub fn read_u16(&mut self, byte_order: Byteorder) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact_buf(&mut buf)?;
        Ok(match byte_order {
            Byteorder::Big => u16::from_be_bytes(buf),
            Byteorder::Little => u16::from_le_bytes(buf),
        })
    }

    pub fn read_i32(&mut self, byte_order: Byteorder) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact_buf(&mut buf)?;
        Ok(match byte_order {
            Byteorder::Big => i32::from_be_bytes(buf),
            Byteorder::Little => i32::from_le_bytes(buf),
        })
    }

    pub fn read_u32(&mut self, byte_order: Byteorder) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact_buf(&mut buf)?;
        Ok(match byte_order {
            Byteorder::Big => u32::from_be_bytes(buf),
            Byteorder::Little => u32::from_le_bytes(buf),
        })
    }

    pub fn read_i64(&mut self, byte_order: Byteorder) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.read_exact_buf(&mut buf)?;
        Ok(match byte_order {
            Byteorder::Big => i64::from_be_bytes(buf),
            Byteorder::Little => i64::from_le_bytes(buf),
        })
    }

    pub fn read_u64(&mut self, byte_order: Byteorder) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact_buf(&mut buf)?;
        Ok(match byte_order {
            Byteorder::Big => u64::from_be_bytes(buf),
            Byteorder::Little => u64::from_le_bytes(buf),
        })
    }

    /// Reads a FourCC block identifier.
    pub fn read_code(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact_buf(&mut buf)?;
        Ok(buf)
    }

    /// Reads a UUID block identifier.
    pub fn read_uuid(&mut self) -> Result<[u8; 16]> {
        let mut buf = [0u8; 16];
        self.read_exact_buf(&mut buf)?;
        Ok(buf)
    }

    /// Skip padding bytes.
    pub fn skip_padding(&mut self, pad: u64) -> Result<()> {
        if pad > 0 {
            self.skip(pad)?;
        }
        Ok(())
    }

    /// Reads the expected padding bytes and returns whether all are null (`0x00`).
    /// Used by `assume_strict_alignment: false` to detect chunks written without padding.
    pub fn padding_valid(&mut self, pad: u64) -> Result<bool> {
        if pad == 0 {
            return Ok(true);
        }

        let pos = self.tell()?;

        // Not enough bytes left in the file to contain padding.
        if self.size() - pos < pad {
            return Ok(false);
        }

        let bytes = self.read_bytes(pad as usize)?;
        let is_padding = bytes.iter().all(|&b| b == 0x00);
        Ok(is_padding)
    }

    /// Reads a block mark of the width defined by the descriptor.
    pub fn read_mark(&mut self, descriptor: &Descriptor) -> Result<Mark> {
        match descriptor.mark_width {
            4 => Ok(Mark::Four(self.read_code()?)),
            16 => Ok(Mark::UUID(self.read_uuid()?)),
            width => Err(Error::Read(format!("unsupported mark width: {width}"))),
        }
    }

    /// Reads a size field of the width defined by the descriptor.
    pub fn read_size(&mut self, descriptor: &Descriptor) -> Result<u64> {
        match descriptor.size_width {
            4 => Ok(self.read_u32(descriptor.byteorder)? as u64),
            8 => Ok(self.read_u64(descriptor.byteorder)?),
            width => Err(Error::Read(format!("unsupported size width: {width}"))),
        }
    }

    /// Reads a size field and subtracts any header overhead to return the actual payload size.
    pub fn read_payload_size(&mut self, descriptor: &Descriptor) -> Result<u64> {
        let offset = self.tell()?;
        let size = self.read_size(descriptor)?;
        size.checked_sub(descriptor.header_overhead as u64).ok_or_else(|| {
            Error::Read(format!(
                "chunk size {size} at offset {offset} is smaller than the format's header overhead"
            ))
        })
    }
}

impl Reader<File> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::new(File::open(path)?)
    }
}

impl<W: Write + Seek> Writer<W> {
    pub fn new(inner: W) -> Self {
        Self { inner: BufWriter::new(inner) }
    }

    fn write_exact(&mut self, bytes: &[u8]) -> Result<()> {
        Ok(self.inner.write_all(bytes)?)
    }

    pub fn write_u32(&mut self, value: u32, byteorder: Byteorder) -> Result<()> {
        let bytes = match byteorder {
            Byteorder::Big => value.to_be_bytes(),
            Byteorder::Little => value.to_le_bytes(),
        };
        self.write_exact(&bytes)
    }

    pub fn write_u64(&mut self, value: u64, byteorder: Byteorder) -> Result<()> {
        let bytes = match byteorder {
            Byteorder::Big => value.to_be_bytes(),
            Byteorder::Little => value.to_le_bytes(),
        };
        self.write_exact(&bytes)
    }

    pub fn write_mark(&mut self, mark: Mark) -> Result<()> {
        match mark {
            Mark::Four(bytes) => self.write_exact(&bytes),
            Mark::UUID(bytes) => self.write_exact(&bytes),
        }
    }

    pub fn write_size(&mut self, size: u64, descriptor: &Descriptor) -> Result<()> {
        match descriptor.size_width {
            4 => self.write_u32(size as u32, descriptor.byteorder),
            8 => self.write_u64(size, descriptor.byteorder),
            _ => unreachable!(),
        }
    }

    pub fn write_padding(&mut self, count: u64) -> Result<()> {
        for _ in 0..count {
            // TODO: Other formats from IFF-RIFF may use different padding bytes.
            self.write_exact(&[0u8])?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        Ok(self.inner.flush()?)
    }
}

impl Writer<File> {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::new(File::create(path)?))
    }
}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub type DynReader = Reader<Box<dyn ReadSeek>>;

impl Reader<Box<dyn ReadSeek>> {
    pub fn boxed<R: Read + Seek + 'static>(inner: R) -> Result<Self> {
        Reader::new(Box::new(inner))
    }
}
