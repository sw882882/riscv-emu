pub mod decode;
pub mod exec;
pub mod trap;

use crate::csr::CsrFile;
use crate::mem::Memory;

#[derive(Default)]
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub csr: CsrFile,
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
        use trap::WithPc;

        // Fetch
        let inst = self.mem.read_u32(self.cpu.pc).with_pc(self.cpu.pc)?;

        // Decode
        let decoded = decode::decode(self.cpu.pc, inst)?;

        // Execute
        exec::execute(&mut self.cpu, &mut self.mem, decoded)
    }
}
