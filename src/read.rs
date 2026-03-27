use crate::{Byteorder, Result};

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub struct Reader<R: Read + Seek> {
    inner: R,
    size: u64,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let size = inner.seek(SeekFrom::End(0))?;
        inner.seek(SeekFrom::Start(0))?;
        Ok(Self { inner, size })
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

    pub fn read_property_code(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact_buf(&mut buf)?;
        Ok(buf)
    }

    pub fn read_property_uuid(&mut self) -> Result<[u8; 16]> {
        let mut buf = [0u8; 16];
        self.read_exact_buf(&mut buf)?;
        Ok(buf)
    }
}

impl Reader<File> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::new(File::open(path)?)
    }
}
