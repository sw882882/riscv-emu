use crate::cpu::Cpu;

pub fn trace(cpu: &Cpu, step: u64) {
    eprintln!(
        "[{:08}] pc=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3(gp)=0x{:016x} x5=0x{:016x}",
        step,
        cpu.pc,
        cpu.regs[1],
        cpu.regs[2],
        cpu.regs[3],
        cpu.regs[5]
    );
}
