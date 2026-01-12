pub mod decode;
pub mod exec;
pub mod trap;

use crate::cpu::trap::WithPc;
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
    // TODO: temp for riscv-tests
    pub host_exit_addr: Option<u64>,
}

pub enum CpuStepResult {
    Continue,
    Trapped(trap::Trap),
    Halt,
    // TODO: in the future
    // Halt(reason: HaltReason),
}

impl std::fmt::Display for CpuStepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuStepResult::Continue => write!(f, "CPU continue"),
            CpuStepResult::Trapped(trap) => write!(f, "CPU trapped: {}", trap),
            CpuStepResult::Halt => write!(f, "CPU halted"),
        }
    }
}

impl std::fmt::Debug for CpuStepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuStepResult::Continue => write!(f, "Continue"),
            CpuStepResult::Trapped(trap) => write!(f, "Trapped({:?})", trap),
            CpuStepResult::Halt => write!(f, "Halt"),
        }
    }
}

impl std::error::Error for CpuStepResult {}

/// Helper trait to convert Result<T, Trap> into Result<T, CpuStepResult>
pub(crate) trait IntoCpuResult<T> {
    fn into_cpu_result(self) -> Result<T, CpuStepResult>;
}

impl<T> IntoCpuResult<T> for Result<T, trap::Trap> {
    fn into_cpu_result(self) -> Result<T, CpuStepResult> {
        self.map_err(CpuStepResult::Trapped)
    }
}

impl Machine {
    pub fn new(ram_bytes: usize) -> Self {
        Self {
            cpu: Cpu::default(),
            mem: Memory::new(ram_bytes),
            host_exit_addr: None,
        }
    }

    pub fn step(&mut self) -> Result<(), CpuStepResult> {
        // Fetch
        let inst = match self
            .mem
            .read_u32(self.cpu.pc)
            .with_pc(self.cpu.pc)
            .into_cpu_result()
        {
            Ok(i) => i,
            Err(CpuStepResult::Trapped(trap)) => {
                self.handle_trap(trap)?;
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // Decode
        let decoded = match decode::decode(self.cpu.pc, inst)
            .with_pc(self.cpu.pc)
            .into_cpu_result()
        {
            Ok(d) => d,
            Err(CpuStepResult::Trapped(trap)) => {
                self.handle_trap(trap)?;
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // Execute
        match exec::execute(&mut self.cpu, &mut self.mem, decoded) {
            Ok(()) => Ok(()),
            Err(CpuStepResult::Trapped(trap)) => {
                self.handle_trap(trap)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn handle_trap(&mut self, trap: trap::Trap) -> Result<(), CpuStepResult> {
        let fault_pc = self.cpu.pc;

        self.cpu
            .csr
            .write(0x341, fault_pc)
            .with_pc(fault_pc)
            .into_cpu_result()?;
        self.cpu
            .csr
            .write(0x342, trap.mcause())
            .with_pc(fault_pc)
            .into_cpu_result()?;
        self.cpu
            .csr
            .write(0x343, trap.mtval())
            .with_pc(fault_pc)
            .into_cpu_result()?;

        let mtvec = self.cpu.csr.read(0x305).unwrap_or(0);
        let base = mtvec & !0b11;
        // TODO: mode handling
        if base == 0 {
            return Err(CpuStepResult::Trapped(trap));
        }
        self.cpu.pc = base;
        Ok(())
    }
}
