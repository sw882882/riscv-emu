use crate::mem::MemError;
use thiserror::Error;

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
}

impl Trap {
    pub fn mcause(&self) -> u64 {
        match self {
            Trap::IllegalInstruction { .. } => 2,
            Trap::Breakpoint { .. } => 3,
            Trap::LoadMisaligned { .. } => 4,
            Trap::StoreMisaligned { .. } => 6,
            Trap::Ecall { .. } => 11,
            Trap::Mem { .. } => 5, // Load access fault or store access fault
        }
    }

    pub fn mtval(&self) -> u64 {
        match self {
            Trap::IllegalInstruction { inst, .. } => *inst as u64,
            Trap::LoadMisaligned { addr, .. } => *addr,
            Trap::StoreMisaligned { addr, .. } => *addr,
            Trap::Mem { .. } => 0, // Could extract fault address if needed
            _ => 0,
        }
    }

    pub fn pc(&self) -> u64 {
        match self {
            Trap::IllegalInstruction { pc, .. } => *pc,
            Trap::Mem { pc, .. } => *pc,
            Trap::Breakpoint { pc } => *pc,
            Trap::LoadMisaligned { pc, .. } => *pc,
            Trap::StoreMisaligned { pc, .. } => *pc,
            Trap::Ecall { pc } => *pc,
        }
    }
}

/// Trait for adding PC context to errors that can become Traps
pub trait WithPc<T> {
    fn with_pc(self, pc: u64) -> Result<T, Trap>;
}

impl<T> WithPc<T> for Result<T, MemError> {
    fn with_pc(self, pc: u64) -> Result<T, Trap> {
        self.map_err(|err| Trap::Mem { pc, err })
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
