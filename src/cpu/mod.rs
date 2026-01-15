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
    pub host_exit_addr: Option<u64>,
    pub max_insns: u64,
    pub executed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltReason {
    HostExit { gp: u64 },
    MaxInsns,
}

impl std::fmt::Display for HaltReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HaltReason::HostExit { gp } => {
                let status = if *gp == 1 { "PASS" } else { "FAIL" };
                write!(f, "host exit [{}] (gp={})", status, gp)
            }
            HaltReason::MaxInsns => write!(f, "maximum instructions executed"),
        }
    }
}

pub enum CpuStepResult {
    Continue,
    Trapped(trap::Trap),
    Halt(HaltReason),
}

impl std::fmt::Display for CpuStepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuStepResult::Continue => write!(f, "CPU continue"),
            CpuStepResult::Trapped(trap) => write!(f, "CPU trapped: {}", trap),
            CpuStepResult::Halt(reason) => write!(f, "CPU halted ({})", reason),
        }
    }
}

impl std::fmt::Debug for CpuStepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuStepResult::Continue => write!(f, "Continue"),
            CpuStepResult::Trapped(trap) => write!(f, "Trapped({:?})", trap),
            CpuStepResult::Halt(reason) => write!(f, "Halt({:?})", reason),
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
            max_insns: 0,
            executed: 0,
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
        // TODO: temp for riscv-tests
        match exec::execute(&mut self.cpu, &mut self.mem, decoded, self.host_exit_addr) {
            Ok(()) => {}
            Err(CpuStepResult::Halt(reason)) => {
                self.executed += 1;
                return Err(CpuStepResult::Halt(reason));
            }
            Err(CpuStepResult::Trapped(trap)) => {
                self.handle_trap(trap)?;
            }
            Err(e) => return Err(e),
        }

        // Increment instruction counter and check max_insns
        self.executed += 1;
        if self.max_insns != 0 && self.executed >= self.max_insns {
            return Err(CpuStepResult::Halt(HaltReason::MaxInsns));
        }

        Ok(())
    }

    fn handle_trap(&mut self, trap: trap::Trap) -> Result<(), CpuStepResult> {
        let fault_pc = trap.pc();

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
