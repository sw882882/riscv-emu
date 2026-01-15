use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemError {
    #[error("address out of range: 0x{0:x}")]
    Oob(u64),
}

pub struct Memory {
    data: Vec<u8>,
    pub base: u64,
}

impl Memory {
    pub fn new(bytes: usize) -> Self {
        Self {
            data: vec![0; bytes],
            base: 0x8000_0000, // around the typical RISC-V physical memory base
        }
    }

    fn check_oob(&self, addr: u64, size: u64) -> Result<usize, MemError> {
        let a = addr.checked_sub(self.base).ok_or(MemError::Oob(addr))?;
        let end = a.checked_add(size).ok_or(MemError::Oob(addr))?;
        if end as usize > self.data.len() {
            return Err(MemError::Oob(addr));
        }
        Ok(a as usize)
    }

    pub fn read_u32(&self, addr: u64) -> Result<u32, MemError> {
        // Allow misaligned accesses - RISC-V ma_data test expects this
        let off = self.check_oob(addr, 4)?;
        let b = &self.data[off..off + 4];
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64, MemError> {
        // Allow misaligned accesses - RISC-V ma_data test expects this
        let off = self.check_oob(addr, 8)?;
        let b = &self.data[off..off + 8];
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn write_u32(&mut self, addr: u64, v: u32) -> Result<(), MemError> {
        // Allow misaligned accesses - RISC-V ma_data test expects this
        let off = self.check_oob(addr, 4)?;
        self.data[off..off + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn write_u64(&mut self, addr: u64, v: u64) -> Result<(), MemError> {
        // Allow misaligned accesses - RISC-V ma_data test expects this
        let off = self.check_oob(addr, 8)?;
        self.data[off..off + 8].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn read_u8(&self, addr: u64) -> Result<u8, MemError> {
        let off = self.check_oob(addr, 1)?;
        Ok(self.data[off])
    }

    pub fn write_u8(&mut self, addr: u64, v: u8) -> Result<(), MemError> {
        let off = self.check_oob(addr, 1)?;
        self.data[off] = v;
        Ok(())
    }

    pub fn write_u16(&mut self, addr: u64, v: u16) -> Result<(), MemError> {
        // Allow misaligned accesses - RISC-V ma_data test expects this
        let off = self.check_oob(addr, 2)?;
        self.data[off..off + 2].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn read_u16(&self, addr: u64) -> Result<u16, MemError> {
        // Allow misaligned accesses - RISC-V ma_data test expects this
        let off = self.check_oob(addr, 2)?;
        let b = &self.data[off..off + 2];
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn write_bytes(&mut self, addr: u64, bytes: &[u8]) -> Result<(), MemError> {
        let off = self.check_oob(addr, bytes.len() as u64)?;
        self.data[off..off + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn end_addr(&self) -> u64 {
        self.base + self.data.len() as u64
    }
}
