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
    stvec: u64,     // Supervisor Trap-Vector Base Address Register
    sepc: u64,      // Supervisor Exception Program Counter
    mntstatus: u64, // Machine Nested Trap Status Register
}

impl CsrFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self, csr: u16) -> Result<u64, String> {
        match csr {
            0x300 => Ok(self.mstatus),
            0x301 => Ok(self.mie),
            0x302 => Ok(self.mtvec),
            0x305 => Ok(self.mtval),
            0x341 => Ok(self.mepc),
            0x342 => Ok(self.mcause),
            0xF14 => Ok(self.mhartid),
            // Add more CSRs as needed
            _ => Err(format!("Unsupported CSR read: 0x{:03x}", csr)),
        }
    }

    pub fn write(&mut self, csr: u16, value: u64) -> Result<(), String> {
        match csr {
            0x300 => {
                self.mstatus = value;
                Ok(())
            }
            0x301 => {
                self.mie = value;
                Ok(())
            }
            0x302 => {
                self.mtvec = value;
                Ok(())
            }
            0x305 => {
                self.mtval = value;
                Ok(())
            }
            0x341 => {
                self.mepc = value;
                Ok(())
            }
            0x342 => {
                self.mcause = value;
                Ok(())
            }
            // Add more CSRs as needed
            _ => Err(format!("Unsupported CSR write: 0x{:03x}", csr)),
        }
    }
    pub fn set_bits(&mut self, csr: u16, mask: u64) -> Result<(), String> {
        let current = self.read(csr)?;
        self.write(csr, current | mask)
    }
    pub fn clear_bits(&mut self, csr: u16, mask: u64) -> Result<(), String> {
        let current = self.read(csr)?;
        self.write(csr, current & !mask)
    }
}
