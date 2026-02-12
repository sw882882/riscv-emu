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

    /// Translate a virtual address to physical address.
    /// With Sv39 enabled, performs page table walk through MMU.
    /// Otherwise returns identity mapping (bare mode).
    pub fn translate_addr(
        &mut self,
        vaddr: u64,
        satp: u64,
        is_fetch: bool,
        is_write: bool,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<u64, MemError> {
        mmu.translate(vaddr, satp, is_fetch, is_write, priv_mode, self)
            .map_err(|_| MemError::Oob(vaddr))
    }

    /// Legacy translate_addr for backward compatibility (identity mapping only)
    fn translate_addr_bare(&self, vaddr: u64, _is_write: bool) -> Result<u64, MemError> {
        // Identity mapping: virtual == physical
        Ok(vaddr)
    }

    fn check_oob(&self, addr: u64, size: u64) -> Result<usize, MemError> {
        let a = addr.checked_sub(self.base).ok_or(MemError::Oob(addr))?;
        let end = a.checked_add(size).ok_or(MemError::Oob(addr))?;
        if end as usize > self.data.len() {
            return Err(MemError::Oob(addr));
        }
        Ok(a as usize)
    }

    // ========== Physical Address Access (internal use) ==========
    // These methods bypass translation and access physical memory directly

    pub fn read_u32_phys(&self, paddr: u64) -> Result<u32, MemError> {
        let off = self.check_oob(paddr, 4)?;
        let b = &self.data[off..off + 4];
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_u64_phys(&self, paddr: u64) -> Result<u64, MemError> {
        let off = self.check_oob(paddr, 8)?;
        let b = &self.data[off..off + 8];
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn write_u32_phys(&mut self, paddr: u64, v: u32) -> Result<(), MemError> {
        let off = self.check_oob(paddr, 4)?;
        self.data[off..off + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn write_u64_phys(&mut self, paddr: u64, v: u64) -> Result<(), MemError> {
        let off = self.check_oob(paddr, 8)?;
        self.data[off..off + 8].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn read_u8_phys(&self, paddr: u64) -> Result<u8, MemError> {
        let off = self.check_oob(paddr, 1)?;
        Ok(self.data[off])
    }

    pub fn write_u8_phys(&mut self, paddr: u64, v: u8) -> Result<(), MemError> {
        let off = self.check_oob(paddr, 1)?;
        self.data[off] = v;
        Ok(())
    }

    pub fn write_u16_phys(&mut self, paddr: u64, v: u16) -> Result<(), MemError> {
        let off = self.check_oob(paddr, 2)?;
        self.data[off..off + 2].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn read_u16_phys(&self, paddr: u64) -> Result<u16, MemError> {
        let off = self.check_oob(paddr, 2)?;
        let b = &self.data[off..off + 2];
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    // ========== Virtual Address Access (public API) ==========
    // These methods translate virtual addresses and then access physical memory

    pub fn read_u32(
        &mut self,
        vaddr: u64,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<u32, MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, false, priv_mode, mmu)?;
        self.read_u32_phys(paddr)
    }

    pub fn read_u64(
        &mut self,
        vaddr: u64,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<u64, MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, false, priv_mode, mmu)?;
        self.read_u64_phys(paddr)
    }

    pub fn write_u32(
        &mut self,
        vaddr: u64,
        v: u32,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<(), MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, true, priv_mode, mmu)?;
        self.write_u32_phys(paddr, v)
    }

    pub fn write_u64(
        &mut self,
        vaddr: u64,
        v: u64,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<(), MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, true, priv_mode, mmu)?;
        self.write_u64_phys(paddr, v)
    }

    pub fn read_u8(
        &mut self,
        vaddr: u64,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<u8, MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, false, priv_mode, mmu)?;
        self.read_u8_phys(paddr)
    }

    pub fn write_u8(
        &mut self,
        vaddr: u64,
        v: u8,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<(), MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, true, priv_mode, mmu)?;
        self.write_u8_phys(paddr, v)
    }

    pub fn write_u16(
        &mut self,
        vaddr: u64,
        v: u16,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<(), MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, true, priv_mode, mmu)?;
        self.write_u16_phys(paddr, v)
    }

    pub fn read_u16(
        &mut self,
        vaddr: u64,
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<u16, MemError> {
        let paddr = self.translate_addr(vaddr, satp, false, false, priv_mode, mmu)?;
        self.read_u16_phys(paddr)
    }

    pub fn write_bytes(
        &mut self,
        vaddr: u64,
        bytes: &[u8],
        satp: u64,
        priv_mode: crate::csr::PrivMode,
        mmu: &mut crate::mmu::Mmu,
    ) -> Result<(), MemError> {
        // For multi-byte writes, translate the start address only
        let paddr = self.translate_addr(vaddr, satp, false, true, priv_mode, mmu)?;
        let off = self.check_oob(paddr, bytes.len() as u64)?;
        self.data[off..off + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn write_bytes_phys(
        &mut self,
        paddr: u64,
        bytes: &[u8],
    ) -> Result<(), MemError> {
        // Direct physical write (for ELF loading and boot)
        let off = self.check_oob(paddr, bytes.len() as u64)?;
        self.data[off..off + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn end_addr(&self) -> u64 {
        self.base + self.data.len() as u64
    }
}
