pub mod decode;
pub mod exec;
pub mod trap;

use crate::cpu::trap::WithPc;
use crate::csr::CsrFile;
use crate::mem::Memory;
use crate::mmu::Mmu;

#[derive(Default)]
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub csr: CsrFile,
}

pub struct Machine {
    pub cpu: Cpu,
    pub mem: Memory,
    pub mmu: Mmu,
    pub host_exit_addr: Option<u64>,
    pub max_insns: u64,
    pub executed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltReason {
    HostExit { code: u64, gp: u64 },
    MaxInsns,
}

impl std::fmt::Display for HaltReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HaltReason::HostExit { code, gp } => {
                let status = if *code == 1 { "PASS" } else { "FAIL" };
                write!(f, "host exit [{}] (code={}, gp={})", status, code, gp)
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
            mmu: Mmu::new(),
            host_exit_addr: None,
            max_insns: 0,
            executed: 0,
        }
    }

    pub fn step(&mut self) -> Result<(), CpuStepResult> {
        use crate::cpu::trap::Trap;

        // Check for pending interrupts before fetching
        if let Some(cause) = self.cpu.csr.check_pending_interrupt() {
            let pc = self.cpu.pc;
            let trap = match cause {
                1 => Trap::SupervisorSoftwareInterrupt { pc },
                3 => Trap::MachineSoftwareInterrupt { pc },
                5 => Trap::SupervisorTimerInterrupt { pc },
                7 => Trap::MachineTimerInterrupt { pc },
                9 => Trap::SupervisorExternalInterrupt { pc },
                11 => Trap::MachineExternalInterrupt { pc },
                _ => return self.finish_step(), // Unknown interrupt, ignore
            };
            self.handle_trap(trap)?;
            return self.finish_step();
        }

        // Fetch
        let inst = match self
            .mem
            .read_u32_exec(
                self.cpu.pc,
                self.cpu.csr.satp,
                self.cpu.csr.priv_mode,
                &mut self.mmu,
            )
            .with_pc(self.cpu.pc)
            .into_cpu_result()
        {
            Ok(i) => i,
            Err(CpuStepResult::Trapped(trap)) => {
                self.handle_trap(trap)?;
                return self.finish_step();
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
                return self.finish_step();
            }
            Err(e) => return Err(e),
        };

        // Execute
        // TODO: temp for riscv-tests
        match exec::execute(
            &mut self.cpu,
            &mut self.mem,
            &mut self.mmu,
            decoded,
            self.host_exit_addr,
        ) {
            Ok(()) => {}
            Err(CpuStepResult::Halt(reason)) => {
                self.executed += 1;
                return Err(CpuStepResult::Halt(reason));
            }
            Err(CpuStepResult::Trapped(trap)) => {
                self.handle_trap(trap)?;
                return self.finish_step();
            }
            Err(e) => return Err(e),
        }

        self.finish_step()
    }

    fn finish_step(&mut self) -> Result<(), CpuStepResult> {
        // Increment instruction counter and check max_insns
        self.executed += 1;
        if self.max_insns != 0 && self.executed >= self.max_insns {
            return Err(CpuStepResult::Halt(HaltReason::MaxInsns));
        }

        // Increment cycle counter
        self.cpu.csr.cycle = self.cpu.csr.cycle.wrapping_add(1);

        Ok(())
    }

    fn handle_trap(&mut self, trap: trap::Trap) -> Result<(), CpuStepResult> {
        use crate::csr::PrivMode;

        let fault_pc = trap.pc();
        let cause = trap.cause();
        let tval = trap.tval();
        let is_interrupt = trap.is_interrupt();

        // Determine if this trap should be delegated to S-mode
        let delegate_to_s = if is_interrupt {
            self.cpu.csr.should_delegate_interrupt(cause)
        } else {
            self.cpu.csr.should_delegate_exception(cause)
        };

        let target_mode = if delegate_to_s {
            PrivMode::Supervisor
        } else {
            PrivMode::Machine
        };

        // Set cause (with interrupt bit if applicable)
        let cause_val = if is_interrupt {
            cause | (1 << 63)
        } else {
            cause
        };

        // Trap entry writes are architectural side-effects and must bypass CSR
        // privilege checks based on the pre-trap mode.
        let tvec = if target_mode == PrivMode::Supervisor {
            self.cpu.csr.sepc = fault_pc & !0b1;
            self.cpu.csr.scause = cause_val;
            self.cpu.csr.stval = tval;
            self.cpu.csr.stvec
        } else {
            self.cpu.csr.mepc = fault_pc & !0b1;
            self.cpu.csr.mcause = cause_val;
            self.cpu.csr.mtval = tval;
            self.cpu.csr.mtvec
        };

        // Update mstatus/sstatus privilege stack
        let current_mode = self.cpu.csr.priv_mode;
        if target_mode == PrivMode::Machine {
            // Save current MIE to MPIE
            let mie = (self.cpu.csr.mstatus >> 3) & 1;
            self.cpu.csr.mstatus = (self.cpu.csr.mstatus & !(1 << 7)) | (mie << 7);
            // Clear MIE
            self.cpu.csr.mstatus &= !(1 << 3);
            // Save previous privilege mode to MPP
            self.cpu.csr.set_mpp(current_mode);
        } else {
            // Supervisor mode trap
            // Save current SIE to SPIE
            let sie = (self.cpu.csr.mstatus >> 1) & 1;
            self.cpu.csr.mstatus = (self.cpu.csr.mstatus & !(1 << 5)) | (sie << 5);
            // Clear SIE
            self.cpu.csr.mstatus &= !(1 << 1);
            // Save previous privilege mode to SPP
            self.cpu.csr.set_spp(current_mode);
        }

        // Update privilege mode
        self.cpu.csr.priv_mode = target_mode;

        // Jump to trap vector
        let mode = tvec & 0b11;
        let base = tvec & !0b11;

        if base == 0 {
            // No trap handler configured
            return Err(CpuStepResult::Trapped(trap));
        }

        self.cpu.pc = match mode {
            0 => base, // Direct mode
            1 => {
                // Vectored mode: base + 4 * cause for interrupts
                if is_interrupt {
                    base.wrapping_add(4 * cause)
                } else {
                    base
                }
            }
            _ => base, // Reserved modes default to direct
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CpuStepResult, HaltReason, Machine};

    #[test]
    fn test_step_trap_path_counts_toward_max_insns() {
        let mut m = Machine::new(0x10000);
        // Force an immediate fetch trap: PC is outside mapped RAM base (0x8000_0000+).
        m.cpu.pc = 0x0;
        // Ensure trap entry is considered configured so trap handling succeeds.
        m.cpu.csr.mtvec = 0x8000_0100;
        m.max_insns = 1;

        match m.step() {
            Err(CpuStepResult::Halt(HaltReason::MaxInsns)) => {}
            other => panic!("expected MaxInsns halt, got {:?}", other),
        }

        assert_eq!(m.executed, 1, "trap-handled step should increment executed");
        assert_eq!(m.cpu.pc, 0x8000_0100, "trap should vector to mtvec base");
    }
}
