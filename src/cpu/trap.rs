use crate::mem::MemError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Trap {
    #[error("illegal instruction at pc=0x{pc:x} inst=0x{inst:08x}")]
    IllegalInstruction { pc: u64, inst: u32 },

    #[error("memory error at pc=0x{pc:x}: {err}")]
    Mem { pc: u64, err: MemError },

    #[error("CSR error at pc=0x{pc:x}: {msg}")]
    CsrError { pc: u64, msg: String },
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

impl<T> WithPc<T> for Result<T, String> {
    fn with_pc(self, pc: u64) -> Result<T, Trap> {
        self.map_err(|msg| Trap::CsrError { pc, msg })
    }
}
