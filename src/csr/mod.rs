use std::fmt;

#[derive(Debug, Clone)]
pub enum CsrError {
    UnsupportedRead(u16),
    UnsupportedWrite(u16),
    PrivilegeViolation(u16),
}

impl fmt::Display for CsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsrError::UnsupportedRead(csr) => write!(f, "unsupported CSR read: 0x{:03x}", csr),
            CsrError::UnsupportedWrite(csr) => write!(f, "unsupported CSR write: 0x{:03x}", csr),
            CsrError::PrivilegeViolation(csr) => {
                write!(f, "privilege violation accessing CSR: 0x{:03x}", csr)
            }
        }
    }
}

/// Privilege modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

impl Default for PrivMode {
    fn default() -> Self {
        PrivMode::Machine
    }
}

impl PrivMode {
    pub fn from_u64(val: u64) -> Option<Self> {
        match val {
            0 => Some(PrivMode::User),
            1 => Some(PrivMode::Supervisor),
            3 => Some(PrivMode::Machine),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct CsrFile {
    // Current privilege mode
    pub priv_mode: PrivMode,

    // Machine-mode CSRs
    pub mstatus: u64,
    pub mtvec: u64,
    pub mepc: u64,
    pub mcause: u64,
    pub mtval: u64,
    pub mie: u64,
    pub mip: u64,
    pub medeleg: u64,
    pub mideleg: u64,
    pub mscratch: u64,

    // Supervisor-mode CSRs
    pub stvec: u64,
    pub sepc: u64,
    pub scause: u64,
    pub stval: u64,
    pub sscratch: u64,
    pub satp: u64,

    // Counters
    pub cycle: u64,
    pub time: u64,

    // Physical Memory Protection (minimal support)
    pmpaddr: [u64; 16],
    pmpcfg: [u8; 16],

    // Hardware thread ID
    mhartid: u64,
}

impl CsrFile {
    pub fn new() -> Self {
        let csr = Self::default();
        // Initialize mstatus with sane defaults
        // Set initial privilege to Machine mode (already default)
        csr
    }

    /// mstatus bit positions
    const MSTATUS_MIE: u64 = 1 << 3;
    const MSTATUS_SIE: u64 = 1 << 1;
    #[allow(dead_code)]
    const MSTATUS_MPIE: u64 = 1 << 7;
    #[allow(dead_code)]
    const MSTATUS_SPIE: u64 = 1 << 5;
    const MSTATUS_MPP: u64 = 0b11 << 11;
    const MSTATUS_SPP: u64 = 1 << 8;
    #[allow(dead_code)]
    const MSTATUS_MPRV: u64 = 1 << 17;
    #[allow(dead_code)]
    const MSTATUS_SUM: u64 = 1 << 18;
    #[allow(dead_code)]
    const MSTATUS_MXR: u64 = 1 << 19;

    /// Extract MPP field from mstatus
    pub fn mpp(&self) -> PrivMode {
        let mpp = (self.mstatus >> 11) & 0b11;
        PrivMode::from_u64(mpp).unwrap_or(PrivMode::User)
    }

    /// Set MPP field in mstatus
    pub fn set_mpp(&mut self, mode: PrivMode) {
        self.mstatus = (self.mstatus & !Self::MSTATUS_MPP) | ((mode as u64) << 11);
    }

    /// Extract SPP field from mstatus
    pub fn spp(&self) -> PrivMode {
        if (self.mstatus & Self::MSTATUS_SPP) != 0 {
            PrivMode::Supervisor
        } else {
            PrivMode::User
        }
    }

    /// Set SPP field in mstatus
    pub fn set_spp(&mut self, mode: PrivMode) {
        if mode == PrivMode::Supervisor {
            self.mstatus |= Self::MSTATUS_SPP;
        } else {
            self.mstatus &= !Self::MSTATUS_SPP;
        }
    }

    /// Check if an exception should be delegated to S-mode
    pub fn should_delegate_exception(&self, cause: u64) -> bool {
        if self.priv_mode == PrivMode::Machine {
            return false; // No delegation from M-mode
        }
        (self.medeleg & (1 << cause)) != 0
    }

    /// Check if an interrupt should be delegated to S-mode
    pub fn should_delegate_interrupt(&self, cause: u64) -> bool {
        if self.priv_mode == PrivMode::Machine {
            return false;
        }
        (self.mideleg & (1 << cause)) != 0
    }

    /// Get sstatus (filtered view of mstatus)
    fn sstatus(&self) -> u64 {
        // sstatus is a restricted view of mstatus
        const SSTATUS_MASK: u64 = (1 << 1) |  // SIE
            (1 << 5) |  // SPIE
            (1 << 8) |  // SPP
            (1 << 18) | // SUM
            (1 << 19) | // MXR
            (0b11 << 13) | // FS
            (0b11 << 15) | // XS
            (1 << 63) | // SD
            (0b1111 << 32); // UXL
        self.mstatus & SSTATUS_MASK
    }

    /// Write sstatus (update only writable bits of mstatus)
    fn write_sstatus(&mut self, value: u64) {
        const SSTATUS_WRITABLE: u64 = (1 << 1) |  // SIE
            (1 << 5) |  // SPIE
            (1 << 8) |  // SPP
            (1 << 18) | // SUM
            (1 << 19); // MXR
        self.mstatus = (self.mstatus & !SSTATUS_WRITABLE) | (value & SSTATUS_WRITABLE);
    }

    /// Get sip (filtered view of mip)
    fn sip(&self) -> u64 {
        const SIP_MASK: u64 = (1 << 1) | (1 << 5) | (1 << 9); // SSIP, STIP, SEIP
        self.mip & SIP_MASK
    }

    /// Write sip (only SSIP is writable)
    fn write_sip(&mut self, value: u64) {
        const SSIP: u64 = 1 << 1;
        self.mip = (self.mip & !SSIP) | (value & SSIP);
    }

    /// Get sie (filtered view of mie)
    fn sie(&self) -> u64 {
        const SIE_MASK: u64 = (1 << 1) | (1 << 5) | (1 << 9); // SSIE, STIE, SEIE
        self.mie & SIE_MASK
    }

    /// Write sie
    fn write_sie(&mut self, value: u64) {
        const SIE_WRITABLE: u64 = (1 << 1) | (1 << 5) | (1 << 9);
        self.mie = (self.mie & !SIE_WRITABLE) | (value & SIE_WRITABLE);
    }

    /// Check privilege level for CSR access
    fn check_csr_privilege(&self, csr: u16) -> Result<(), CsrError> {
        let priv_level = (csr >> 8) & 0x3;
        let required = match priv_level {
            0 => PrivMode::User,
            1 => PrivMode::Supervisor,
            3 => PrivMode::Machine,
            _ => return Err(CsrError::UnsupportedRead(csr)),
        };

        if (self.priv_mode as u64) < (required as u64) {
            return Err(CsrError::PrivilegeViolation(csr));
        }
        Ok(())
    }

    pub fn read(&self, csr: u16) -> Result<u64, CsrError> {
        self.check_csr_privilege(csr)?;

        match csr {
            // Supervisor trap setup
            0x100 => Ok(self.sstatus()),
            0x104 => Ok(self.sie()),
            0x105 => Ok(self.stvec),

            // Supervisor trap handling
            0x140 => Ok(self.sscratch),
            0x141 => Ok(self.sepc),
            0x142 => Ok(self.scause),
            0x143 => Ok(self.stval),
            0x144 => Ok(self.sip()),

            // Supervisor address translation
            0x180 => Ok(self.satp),

            // Machine information registers
            0xF11 => Ok(0), // mvendorid
            0xF12 => Ok(0), // marchid
            0xF13 => Ok(0), // mimpid
            0xF14 => Ok(self.mhartid),

            // Machine trap setup
            0x300 => Ok(self.mstatus),
            0x301 => Ok(0x8000000000141101), // misa: RV64IMAC
            0x302 => Ok(self.medeleg),
            0x303 => Ok(self.mideleg),
            0x304 => Ok(self.mie),
            0x305 => Ok(self.mtvec),

            // Machine trap handling
            0x340 => Ok(self.mscratch),
            0x341 => Ok(self.mepc),
            0x342 => Ok(self.mcause),
            0x343 => Ok(self.mtval),
            0x344 => Ok(self.mip),

            // Machine counters/timers
            0xB00 => Ok(self.cycle), // mcycle
            0xB02 => Ok(self.time),  // minstret (use cycle for now)
            0xC00 => Ok(self.cycle), // cycle
            0xC01 => Ok(self.time),  // time
            0xC02 => Ok(self.cycle), // instret

            // Physical memory protection
            0x3A0 => Ok(self.pmpcfg[0] as u64),
            0x3B0 => Ok(self.pmpaddr[0]),

            _ => Err(CsrError::UnsupportedRead(csr)),
        }
    }

    pub fn write(&mut self, csr: u16, value: u64) -> Result<(), CsrError> {
        self.check_csr_privilege(csr)?;

        // Check if CSR is read-only (top 2 bits == 0b11)
        if (csr >> 10) == 0b11 {
            return Ok(()); // Silently ignore writes to read-only CSRs
        }

        match csr {
            // Supervisor trap setup
            0x100 => {
                self.write_sstatus(value);
                Ok(())
            }
            0x104 => {
                self.write_sie(value);
                Ok(())
            }
            0x105 => {
                self.stvec = value;
                Ok(())
            }

            // Supervisor trap handling
            0x140 => {
                self.sscratch = value;
                Ok(())
            }
            0x141 => {
                self.sepc = value & !0b1; // Clear bottom bit
                Ok(())
            }
            0x142 => {
                self.scause = value;
                Ok(())
            }
            0x143 => {
                self.stval = value;
                Ok(())
            }
            0x144 => {
                self.write_sip(value);
                Ok(())
            }

            // Supervisor address translation
            0x180 => {
                // For now, only accept bare mode (satp.mode = 0)
                let mode = value >> 60;
                if mode == 0 {
                    self.satp = value;
                }
                // Silently ignore writes with non-zero mode until Sv39 is implemented
                Ok(())
            }

            // Machine trap setup
            0x300 => {
                // Mask writable bits of mstatus
                const MSTATUS_WRITABLE: u64 = (1 << 1) |  // SIE
                    (1 << 3) |  // MIE
                    (1 << 5) |  // SPIE
                    (1 << 7) |  // MPIE
                    (1 << 8) |  // SPP
                    (0b11 << 11) | // MPP
                    (1 << 17) | // MPRV
                    (1 << 18) | // SUM
                    (1 << 19); // MXR
                self.mstatus = (self.mstatus & !MSTATUS_WRITABLE) | (value & MSTATUS_WRITABLE);
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

            // Machine trap handling
            0x340 => {
                self.mscratch = value;
                Ok(())
            }
            0x341 => {
                self.mepc = value & !0b1; // Clear bottom bit
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
            0x344 => {
                // mip - some bits writable by software
                const MIP_WRITABLE: u64 = (1 << 1) | (1 << 3); // SSIP, MSIP
                self.mip = (self.mip & !MIP_WRITABLE) | (value & MIP_WRITABLE);
                Ok(())
            }

            // Physical memory protection
            0x3A0 => {
                self.pmpcfg[0] = value as u8;
                Ok(())
            }
            0x3B0 => {
                self.pmpaddr[0] = value;
                Ok(())
            }

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

    /// Check for pending and enabled interrupts, return highest priority interrupt cause if any
    pub fn check_pending_interrupt(&self) -> Option<u64> {
        // Calculate which interrupts are pending and enabled
        let pending_enabled = self.mip & self.mie;

        if pending_enabled == 0 {
            return None;
        }

        // Check if interrupts are globally enabled for current privilege level
        let interrupts_enabled = match self.priv_mode {
            PrivMode::Machine => (self.mstatus & Self::MSTATUS_MIE) != 0,
            PrivMode::Supervisor => {
                // S-mode: enabled if SIE=1, or if in lower privilege
                (self.mstatus & Self::MSTATUS_SIE) != 0
            }
            PrivMode::User => true, // Interrupts always enabled in U-mode
        };

        if !interrupts_enabled {
            return None;
        }

        // Priority order: MEI, MSI, MTI, SEI, SSI, STI
        // Check M-mode interrupts first
        if (pending_enabled & (1 << 11)) != 0 {
            Some(11) // MEI
        } else if (pending_enabled & (1 << 3)) != 0 {
            Some(3) // MSI
        } else if (pending_enabled & (1 << 7)) != 0 {
            Some(7) // MTI
        } else if (pending_enabled & (1 << 9)) != 0 {
            Some(9) // SEI
        } else if (pending_enabled & (1 << 1)) != 0 {
            Some(1) // SSI
        } else if (pending_enabled & (1 << 5)) != 0 {
            Some(5) // STI
        } else {
            None
        }
    }

    /// Set a timer interrupt pending
    pub fn set_timer_interrupt(&mut self, is_machine: bool) {
        if is_machine {
            self.mip |= 1 << 7; // MTIP
        } else {
            self.mip |= 1 << 5; // STIP
        }
    }

    /// Clear a timer interrupt
    pub fn clear_timer_interrupt(&mut self, is_machine: bool) {
        if is_machine {
            self.mip &= !(1 << 7); // MTIP
        } else {
            self.mip &= !(1 << 5); // STIP
        }
    }
}
