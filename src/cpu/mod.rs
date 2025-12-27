pub mod decode;
pub mod exec;
pub mod trap;

use crate::csr::CsrFile;
use crate::mem::Memory;

// Memory operation error handling macro
// Converts MemError into Trap::Mem with PC context
macro_rules! mem {
    ($pc:expr, $expr:expr) => {
        $expr.map_err(|e| $crate::cpu::trap::Trap::from_mem($pc, e))
    };
}

pub(crate) use mem;

#[derive(Default)]
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub csrs: CsrFile,
}

pub struct Machine {
    pub cpu: Cpu,
    pub mem: Memory,
}

impl Machine {
    pub fn new(ram_bytes: usize) -> Self {
        Self {
            cpu: Cpu::default(),
            mem: Memory::new(ram_bytes),
        }
    }

    pub fn step(&mut self) -> Result<(), trap::Trap> {
        // Fetch
        let inst = mem!(self.cpu.pc, self.mem.read_u32(self.cpu.pc))?;

        // Decode
        let decoded = decode::decode(self.cpu.pc, inst)?;

        // Execute
        exec::execute(&mut self.cpu, &mut self.mem, decoded)
    }
}
