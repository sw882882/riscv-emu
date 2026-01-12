use std::fmt;

#[derive(Debug, Clone)]
pub enum CsrError {
    UnsupportedRead(u16),
    UnsupportedWrite(u16),
}

impl fmt::Display for CsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsrError::UnsupportedRead(csr) => write!(f, "unsupported CSR read: 0x{:03x}", csr),
            CsrError::UnsupportedWrite(csr) => write!(f, "unsupported CSR write: 0x{:03x}", csr),
        }
    }
}

#[derive(Default)]
pub struct CsrFile {
    // Keep it simple at first. Add real CSRs as you need them.
    // You can switch to an array/map later.

    // core csrs for unit testing
    mtvec: u64,  // Machine Trap-Vector Base Address Register
    mepc: u64,   // Machine Exception Program Counter
    mcause: u64, // Machine Cause Register
    mtval: u64,  // Machine Trap Value Register

    mstatus: u64, // Machine Status Register
    // id
    mhartid: u64, // Machine Hardware Thread ID Register = 0

    // stub for now
    // TODO: print a single warning when these are accessed
    mie: u64,           // Machine Interrupt Enable Register
    mip: u64,           // Machine Interrupt Pending Register
    medeleg: u64,       // Machine Exception Delegation Register
    mideleg: u64,       // Machine Interrupt Delegation Register
    satp: u64,          // Supervisor Address Translation and Protection Register
    pmpaddr: [u64; 16], // Physical Memory Protection Address Registers
    // care about 0 for now
    pmpcfg: [u8; 16], // Physical Memory Protection Configuration Registers
    // care about 0 for now
    stvec: u64,    // Supervisor Trap-Vector Base Address Register
    sepc: u64,     // Supervisor Exception Program Counter
    mnstatus: u64, // Machine Nested Trap Status Register
}

impl CsrFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self, csr: u16) -> Result<u64, CsrError> {
        match csr {
            // supervisor trap setup
            0x105 => Ok(self.stvec),

            // supervisor address translation and protection
            0x180 => Ok(self.satp),

            // machine trap setup
            0x300 => Ok(self.mstatus),
            // 0x301 => Ok(self.misa),
            0x302 => Ok(self.medeleg),
            0x303 => Ok(self.mideleg),
            0x304 => Ok(self.mie),
            0x305 => Ok(self.mtvec),
            // 0x306 => Ok(self.mcounteren),
            // 0x310 => Ok(self.mstatush),

            // machine trap handling
            // 0x340 => Ok(self.mscratch),
            0x341 => Ok(self.mepc),
            0x342 => Ok(self.mcause),
            0x343 => Ok(self.mtval),
            0x344 => Ok(self.mip),
            // 0x34A => Ok(self.mtinst),
            // 0x34B => Ok(self.mtval2),

            // physical memory protection
            0x3A0 => Ok(self.pmpcfg[0] as u64),
            0x3B0 => Ok(self.pmpaddr[0]),

            // machine nested trap
            0x744 => Ok(self.mnstatus),

            // machine information
            0xF14 => Ok(0), // mhartid - always returns 0

            _ => Err(CsrError::UnsupportedRead(csr)),
        }
    }

    pub fn write(&mut self, csr: u16, value: u64) -> Result<(), CsrError> {
        match csr {
            // supervisor trap setup
            0x105 => {
                self.stvec = value;
                Ok(())
            }

            // supervisor address translation and protection
            0x180 => {
                self.satp = value;
                Ok(())
            }

            // machine trap setup
            0x300 => {
                self.mstatus = value;
                Ok(())
            }
            0x302 => {
                self.medeleg = value;
                Ok(())
            }
            0x303 => {
                self.mideleg = value;
                Ok(())
            }
            0x304 => {
                self.mie = value;
                Ok(())
            }
            0x305 => {
                self.mtvec = value;
                Ok(())
            }

            // machine trap handling
            0x341 => {
                self.mepc = value;
                Ok(())
            }
            0x342 => {
                self.mcause = value;
                Ok(())
            }
            0x343 => {
                self.mtval = value;
                Ok(())
            }

            // physical memory protection
            0x3A0 => {
                self.pmpcfg[0] = value as u8;
                Ok(())
            }
            0x3B0 => {
                self.pmpaddr[0] = value;
                Ok(())
            }

            // machine nested trap
            0x744 => {
                self.mnstatus = value;
                Ok(())
            }

            // mhartid is read-only, silently ignore writes
            0xF14 => Ok(()),

            _ => Err(CsrError::UnsupportedWrite(csr)),
        }
    }
    pub fn set_bits(&mut self, csr: u16, mask: u64) -> Result<(), CsrError> {
        let current = self.read(csr)?;
        self.write(csr, current | mask)
    }
    pub fn clear_bits(&mut self, csr: u16, mask: u64) -> Result<(), CsrError> {
        let current = self.read(csr)?;
        self.write(csr, current & !mask)
    }
}
