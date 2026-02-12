use crate::mem::MemError;
use thiserror::Error;

/// Exception cause codes (RISC-V Privileged Spec)
pub mod causes {
    pub const INSTRUCTION_ADDRESS_MISALIGNED: u64 = 0;
    pub const INSTRUCTION_ACCESS_FAULT: u64 = 1;
    pub const ILLEGAL_INSTRUCTION: u64 = 2;
    pub const BREAKPOINT: u64 = 3;
    pub const LOAD_ADDRESS_MISALIGNED: u64 = 4;
    pub const LOAD_ACCESS_FAULT: u64 = 5;
    pub const STORE_ADDRESS_MISALIGNED: u64 = 6;
    pub const STORE_ACCESS_FAULT: u64 = 7;
    pub const ECALL_U: u64 = 8;
    pub const ECALL_S: u64 = 9;
    pub const ECALL_M: u64 = 11;
    pub const INSTRUCTION_PAGE_FAULT: u64 = 12;
    pub const LOAD_PAGE_FAULT: u64 = 13;
    pub const STORE_PAGE_FAULT: u64 = 15;

    // Interrupt causes (high bit set when interrupt)
    pub const SSI: u64 = 1;
    pub const MSI: u64 = 3;
    pub const STI: u64 = 5;
    pub const MTI: u64 = 7;
    pub const SEI: u64 = 9;
    pub const MEI: u64 = 11;
}

#[derive(Error, Debug)]
pub enum Trap {
    #[error("illegal instruction at pc=0x{pc:x} inst=0x{inst:08x}")]
    IllegalInstruction { pc: u64, inst: u32 },

    #[error("memory error at pc=0x{pc:x}: {err}")]
    Mem { pc: u64, err: MemError },

    #[error("breakpoint at pc=0x{pc:x}")]
    Breakpoint { pc: u64 },

    #[error("load address misaligned at pc=0x{pc:x}, addr=0x{addr:x}")]
    LoadMisaligned { pc: u64, addr: u64 },

    #[error("store address misaligned at pc=0x{pc:x}, addr=0x{addr:x}")]
    StoreMisaligned { pc: u64, addr: u64 },

    #[error("environment call at pc=0x{pc:x}")]
    Ecall { pc: u64 },

    #[error("supervisor environment call at pc=0x{pc:x}")]
    EcallFromS { pc: u64 },

    #[error("machine environment call at pc=0x{pc:x}")]
    EcallFromM { pc: u64 },

    #[error("instruction page fault at pc=0x{pc:x}, addr=0x{addr:x}")]
    InstructionPageFault { pc: u64, addr: u64 },

    #[error("load page fault at pc=0x{pc:x}, addr=0x{addr:x}")]
    LoadPageFault { pc: u64, addr: u64 },

    #[error("store page fault at pc=0x{pc:x}, addr=0x{addr:x}")]
    StorePageFault { pc: u64, addr: u64 },

    // Interrupts
    #[error("machine software interrupt")]
    MachineSoftwareInterrupt { pc: u64 },

    #[error("machine timer interrupt")]
    MachineTimerInterrupt { pc: u64 },

    #[error("machine external interrupt")]
    MachineExternalInterrupt { pc: u64 },

    #[error("supervisor software interrupt")]
    SupervisorSoftwareInterrupt { pc: u64 },

    #[error("supervisor timer interrupt")]
    SupervisorTimerInterrupt { pc: u64 },

    #[error("supervisor external interrupt")]
    SupervisorExternalInterrupt { pc: u64 },
}

impl Trap {
    /// Returns the exception/interrupt cause code
    pub fn cause(&self) -> u64 {
        match self {
            // Exceptions (no interrupt bit)
            Trap::IllegalInstruction { .. } => causes::ILLEGAL_INSTRUCTION,
            Trap::Breakpoint { .. } => causes::BREAKPOINT,
            Trap::LoadMisaligned { .. } => causes::LOAD_ADDRESS_MISALIGNED,
            Trap::StoreMisaligned { .. } => causes::STORE_ADDRESS_MISALIGNED,
            Trap::EcallFromS { .. } => causes::ECALL_S,
            Trap::EcallFromM { .. } => causes::ECALL_M,
            Trap::Ecall { .. } => causes::ECALL_U,
            Trap::InstructionPageFault { .. } => causes::INSTRUCTION_PAGE_FAULT,
            Trap::LoadPageFault { .. } => causes::LOAD_PAGE_FAULT,
            Trap::StorePageFault { .. } => causes::STORE_PAGE_FAULT,
            Trap::Mem { .. } => causes::LOAD_ACCESS_FAULT, // Load/store access fault

            // Interrupts (cause without interrupt bit)
            Trap::SupervisorSoftwareInterrupt { .. } => causes::SSI,
            Trap::MachineSoftwareInterrupt { .. } => causes::MSI,
            Trap::SupervisorTimerInterrupt { .. } => causes::STI,
            Trap::MachineTimerInterrupt { .. } => causes::MTI,
            Trap::SupervisorExternalInterrupt { .. } => causes::SEI,
            Trap::MachineExternalInterrupt { .. } => causes::MEI,
        }
    }

    /// Returns true if this is an interrupt (vs synchronous exception)
    pub fn is_interrupt(&self) -> bool {
        matches!(
            self,
            Trap::MachineSoftwareInterrupt { .. }
                | Trap::MachineTimerInterrupt { .. }
                | Trap::MachineExternalInterrupt { .. }
                | Trap::SupervisorSoftwareInterrupt { .. }
                | Trap::SupervisorTimerInterrupt { .. }
                | Trap::SupervisorExternalInterrupt { .. }
        )
    }

    /// Returns the trap value (mtval/stval)
    pub fn tval(&self) -> u64 {
        match self {
            Trap::IllegalInstruction { inst, .. } => *inst as u64,
            Trap::LoadMisaligned { addr, .. } => *addr,
            Trap::StoreMisaligned { addr, .. } => *addr,
            Trap::InstructionPageFault { addr, .. } => *addr,
            Trap::LoadPageFault { addr, .. } => *addr,
            Trap::StorePageFault { addr, .. } => *addr,
            Trap::Mem { .. } => 0, // Could extract fault address if needed
            _ => 0,
        }
    }

    /// Returns the PC where the trap occurred
    pub fn pc(&self) -> u64 {
        match self {
            Trap::IllegalInstruction { pc, .. } => *pc,
            Trap::Mem { pc, .. } => *pc,
            Trap::Breakpoint { pc } => *pc,
            Trap::LoadMisaligned { pc, .. } => *pc,
            Trap::StoreMisaligned { pc, .. } => *pc,
            Trap::Ecall { pc } => *pc,
            Trap::EcallFromS { pc } => *pc,
            Trap::EcallFromM { pc } => *pc,
            Trap::InstructionPageFault { pc, .. } => *pc,
            Trap::LoadPageFault { pc, .. } => *pc,
            Trap::StorePageFault { pc, .. } => *pc,
            Trap::MachineSoftwareInterrupt { pc } => *pc,
            Trap::MachineTimerInterrupt { pc } => *pc,
            Trap::MachineExternalInterrupt { pc } => *pc,
            Trap::SupervisorSoftwareInterrupt { pc } => *pc,
            Trap::SupervisorTimerInterrupt { pc } => *pc,
            Trap::SupervisorExternalInterrupt { pc } => *pc,
        }
    }

    // Legacy compatibility methods - can be removed once callers updated
    #[deprecated(note = "use cause() instead")]
    pub fn mcause(&self) -> u64 {
        self.cause()
    }

    #[deprecated(note = "use tval() instead")]
    pub fn mtval(&self) -> u64 {
        self.tval()
    }
}

/// Trait for adding PC context to errors that can become Traps
pub trait WithPc<T> {
    fn with_pc(self, pc: u64) -> Result<T, Trap>;
}

impl<T> WithPc<T> for Result<T, MemError> {
    fn with_pc(self, pc: u64) -> Result<T, Trap> {
        self.map_err(|err| match err {
            MemError::InstructionPageFault(addr) => Trap::InstructionPageFault { pc, addr },
            MemError::LoadPageFault(addr) => Trap::LoadPageFault { pc, addr },
            MemError::StorePageFault(addr) => Trap::StorePageFault { pc, addr },
            _ => Trap::Mem { pc, err },
        })
    }
}

impl<T> WithPc<T> for Result<T, crate::csr::CsrError> {
    fn with_pc(self, pc: u64) -> Result<T, Trap> {
        self.map_err(|_err| Trap::IllegalInstruction {
            pc,
            inst: 0, // CSR errors treated as illegal instruction
        })
    }
}

impl<T> WithPc<T> for Result<T, crate::cpu::decode::DecodeError> {
    fn with_pc(self, pc: u64) -> Result<T, Trap> {
        self.map_err(|err| match err {
            crate::cpu::decode::DecodeError::InvalidOpcode { inst } => {
                Trap::IllegalInstruction { pc, inst }
            }
            crate::cpu::decode::DecodeError::InvalidFunct { inst } => {
                Trap::IllegalInstruction { pc, inst }
            }
        })
    }
}
