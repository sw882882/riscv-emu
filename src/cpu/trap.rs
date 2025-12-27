use crate::mem::MemError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Trap {
    #[error("illegal instruction at pc=0x{pc:x} inst=0x{inst:08x}")]
    IllegalInstruction { pc: u64, inst: u32 },

    #[error("memory error at pc=0x{pc:x}: {err}")]
    Mem { pc: u64, err: MemError },
}

impl Trap {
    pub fn from_mem(pc: u64, err: MemError) -> Self {
        Trap::Mem { pc, err }
    }
}
