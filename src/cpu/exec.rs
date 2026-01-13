use super::IntoCpuResult;
use super::decode::Instr;
use super::trap::{Trap, WithPc};
use crate::cpu::{Cpu, CpuStepResult};
use crate::mem::Memory;

pub fn execute(
    cpu: &mut Cpu,
    mem: &mut Memory,
    instr: Instr,
    host_exit_addr: Option<u64>,
) -> Result<(), CpuStepResult> {
    let pc = cpu.pc;

    let r = |cpu: &Cpu, idx: u8| -> u64 { cpu.regs[idx as usize] };
    let w = |cpu: &mut Cpu, idx: u8, val: u64| {
        if idx != 0 {
            cpu.regs[idx as usize] = val;
        } // x0 hardwired
    };

    let sign_extend = |val: i64, bits: u32| -> i64 {
        let shift = 64 - bits;
        (val << shift) >> shift
    };

    match instr {
        Instr::Addi { rd, rs1, imm } => {
            w(cpu, rd, r(cpu, rs1).wrapping_add(imm as u64));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Add { rd, rs1, rs2 } => {
            w(cpu, rd, r(cpu, rs1).wrapping_add(r(cpu, rs2)));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sub { rd, rs1, rs2 } => {
            w(cpu, rd, r(cpu, rs1).wrapping_sub(r(cpu, rs2)));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Beq { rs1, rs2, off } => {
            cpu.pc = if r(cpu, rs1) == r(cpu, rs2) {
                pc.wrapping_add(off as u64)
            } else {
                pc.wrapping_add(4)
            };
        }
        Instr::Bne { rs1, rs2, off } => {
            cpu.pc = if r(cpu, rs1) != r(cpu, rs2) {
                pc.wrapping_add(off as u64)
            } else {
                pc.wrapping_add(4)
            };
        }
        Instr::Lui { rd, imm } => {
            w(cpu, rd, imm as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Jal { rd, off } => {
            w(cpu, rd, pc.wrapping_add(4));
            cpu.pc = pc.wrapping_add(off as u64);
        }
        Instr::LB { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let byte = mem.read_u8(addr).with_pc(pc).into_cpu_result()?;
            let value = sign_extend(byte as i64, 8) as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LBU { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let byte = mem.read_u8(addr).with_pc(pc).into_cpu_result()?;
            let value = byte as u64; // Zero-extend from 8 to 64 bits
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LH { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let half = mem.read_u16(addr).with_pc(pc).into_cpu_result()?;
            let value = sign_extend(half as i64, 16) as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LHU { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let half = mem.read_u16(addr).with_pc(pc).into_cpu_result()?;
            let value = half as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LD { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let word = mem.read_u64(addr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, word);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::SB { rs1, rs2, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let byte = (r(cpu, rs2) & 0xff) as u8;
            mem.write_u8(addr, byte).with_pc(pc).into_cpu_result()?;
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Xor { rd, rs1, rs2 } => {
            w(cpu, rd, r(cpu, rs1) ^ r(cpu, rs2));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Or { rd, rs1, rs2 } => {
            w(cpu, rd, r(cpu, rs1) | r(cpu, rs2));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::And { rd, rs1, rs2 } => {
            w(cpu, rd, r(cpu, rs1) & r(cpu, rs2));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sll { rd, rs1, rs2 } => {
            w(
                cpu,
                rd,
                r(cpu, rs1).wrapping_shl((r(cpu, rs2) & 0x3f) as u32),
            );
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Srl { rd, rs1, rs2 } => {
            w(
                cpu,
                rd,
                r(cpu, rs1).wrapping_shr((r(cpu, rs2) & 0x3f) as u32),
            );
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sra { rd, rs1, rs2 } => {
            w(
                cpu,
                rd,
                ((r(cpu, rs1) as i64).wrapping_shr((r(cpu, rs2) & 0x3f) as u32)) as u64,
            );
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Slt { rd, rs1, rs2 } => {
            w(
                cpu,
                rd,
                if (r(cpu, rs1) as i64) < (r(cpu, rs2) as i64) {
                    1
                } else {
                    0
                },
            );
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sltu { rd, rs1, rs2 } => {
            w(cpu, rd, if r(cpu, rs1) < r(cpu, rs2) { 1 } else { 0 });
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Xori { rd, rs1, imm } => {
            w(cpu, rd, r(cpu, rs1) ^ (imm as u64));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Ori { rd, rs1, imm } => {
            w(cpu, rd, r(cpu, rs1) | (imm as u64));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Andi { rd, rs1, imm } => {
            w(cpu, rd, r(cpu, rs1) & (imm as u64));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Slli { rd, rs1, shamt } => {
            w(cpu, rd, r(cpu, rs1).wrapping_shl((shamt & 0x3f) as u32));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Srli { rd, rs1, shamt } => {
            w(cpu, rd, r(cpu, rs1).wrapping_shr((shamt & 0x3f) as u32));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Srai { rd, rs1, shamt } => {
            w(
                cpu,
                rd,
                ((r(cpu, rs1) as i64).wrapping_shr((shamt & 0x3f) as u32)) as u64,
            );
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Slti { rd, rs1, imm } => {
            w(
                cpu,
                rd,
                if (r(cpu, rs1) as i64) < (imm as i64) {
                    1
                } else {
                    0
                },
            );
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sltiu { rd, rs1, imm } => {
            w(cpu, rd, if r(cpu, rs1) < (imm as u64) { 1 } else { 0 });
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LW { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let word = mem.read_u32(addr).with_pc(pc).into_cpu_result()?;
            let value = sign_extend(word as i64, 32) as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::SH { rs1, rs2, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let half = (r(cpu, rs2) & 0xffff) as u16;
            mem.write_u16(addr, half).with_pc(pc).into_cpu_result()?;
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::SW { rs1, rs2, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let word = (r(cpu, rs2) & 0xffff_ffff) as u32;
            // host exit address check
            // TODO: temp for riscv-tests
            if let Some(exit_addr) = host_exit_addr {
                if addr == exit_addr {
                    let gp = r(cpu, 3); // x3 is gp (global pointer)
                    return Err(CpuStepResult::Halt(super::HaltReason::HostExit { gp }));
                }
            }
            mem.write_u32(addr, word).with_pc(pc).into_cpu_result()?;
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Blt { rs1, rs2, off } => {
            cpu.pc = if (r(cpu, rs1) as i64) < (r(cpu, rs2) as i64) {
                pc.wrapping_add(off as u64)
            } else {
                pc.wrapping_add(4)
            };
        }
        Instr::Bge { rs1, rs2, off } => {
            cpu.pc = if (r(cpu, rs1) as i64) >= (r(cpu, rs2) as i64) {
                pc.wrapping_add(off as u64)
            } else {
                pc.wrapping_add(4)
            };
        }
        Instr::Bltu { rs1, rs2, off } => {
            cpu.pc = if r(cpu, rs1) < r(cpu, rs2) {
                pc.wrapping_add(off as u64)
            } else {
                pc.wrapping_add(4)
            };
        }
        Instr::Bgeu { rs1, rs2, off } => {
            cpu.pc = if r(cpu, rs1) >= r(cpu, rs2) {
                pc.wrapping_add(off as u64)
            } else {
                pc.wrapping_add(4)
            };
        }
        Instr::Jalr { rd, rs1, off } => {
            let target = r(cpu, rs1).wrapping_add(off as u64) & !1;
            w(cpu, rd, pc.wrapping_add(4));
            cpu.pc = target;
        }
        Instr::Auipc { rd, imm } => {
            w(cpu, rd, pc.wrapping_add(imm as u64));
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Ecall => {
            return Err(CpuStepResult::Trapped(Trap::Ecall { pc }));
        }
        Instr::Ebreak => {
            return Err(CpuStepResult::Trapped(Trap::Breakpoint { pc }));
        }
        Instr::Addiw { rd, rs1, imm } => {
            let result = (r(cpu, rs1) as i64).wrapping_add(imm as i64);
            w(cpu, rd, sign_extend(result, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Slliw { rd, rs1, shamt } => {
            let result = (r(cpu, rs1) & 0xffff_ffff).wrapping_shl((shamt & 0x1f) as u32);
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Srliw { rd, rs1, shamt } => {
            let result = (r(cpu, rs1) & 0xffff_ffff).wrapping_shr((shamt & 0x1f) as u32);
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sraiw { rd, rs1, shamt } => {
            let result =
                ((r(cpu, rs1) & 0xffff_ffff) as i32).wrapping_shr((shamt & 0x1f) as u32) as u32;
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Addw { rd, rs1, rs2 } => {
            let result = (r(cpu, rs1) as i32).wrapping_add(r(cpu, rs2) as i32);
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Subw { rd, rs1, rs2 } => {
            let result = (r(cpu, rs1) as i32).wrapping_sub(r(cpu, rs2) as i32);
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sllw { rd, rs1, rs2 } => {
            let result = (r(cpu, rs1) as u32).wrapping_shl((r(cpu, rs2) & 0x1f) as u32);
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Srlw { rd, rs1, rs2 } => {
            let result = (r(cpu, rs1) as u32).wrapping_shr((r(cpu, rs2) & 0x1f) as u32);
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Sraw { rd, rs1, rs2 } => {
            let result = ((r(cpu, rs1) as i32).wrapping_shr((r(cpu, rs2) & 0x1f) as u32)) as i32;
            w(cpu, rd, sign_extend(result as i64, 32) as u64);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LWU { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let word = mem.read_u32(addr).with_pc(pc).into_cpu_result()?;
            let value = word as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::SD { rs1, rs2, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            // host exit address check
            if let Some(exit_addr) = host_exit_addr {
                if addr == exit_addr {
                    let gp = r(cpu, 3); // x3 is gp (global pointer)
                    return Err(CpuStepResult::Halt(super::HaltReason::HostExit { gp }));
                }
            }
            mem.write_u64(addr, r(cpu, rs2))
                .with_pc(pc)
                .into_cpu_result()?;
            cpu.pc = pc.wrapping_add(4);
        }
        // TODO: atomicity later
        Instr::Csrrw { rd, csr, rs1 } => {
            let csr_value = cpu.csr.read(csr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, csr_value);
            let rs1_value = r(cpu, rs1);
            cpu.csr
                .write(csr, rs1_value)
                .with_pc(pc)
                .into_cpu_result()?;
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Csrrs { rd, csr, rs1 } => {
            let csr_value = cpu.csr.read(csr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, csr_value);
            let rs1_value = r(cpu, rs1);
            if rs1 != 0 {
                cpu.csr
                    .set_bits(csr, rs1_value)
                    .with_pc(pc)
                    .into_cpu_result()?;
            }
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Csrrc { rd, csr, rs1 } => {
            let csr_value = cpu.csr.read(csr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, csr_value);
            let rs1_value = r(cpu, rs1);
            if rs1 != 0 {
                cpu.csr
                    .clear_bits(csr, rs1_value)
                    .with_pc(pc)
                    .into_cpu_result()?;
            }
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Csrrwi { rd, csr, uimm } => {
            let csr_value = cpu.csr.read(csr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, csr_value);
            cpu.csr
                .write(csr, uimm as u64)
                .with_pc(pc)
                .into_cpu_result()?;
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Csrrsi { rd, csr, uimm } => {
            let csr_value = cpu.csr.read(csr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, csr_value);
            if uimm != 0 {
                cpu.csr
                    .set_bits(csr, uimm as u64)
                    .with_pc(pc)
                    .into_cpu_result()?;
            }
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Csrrci { rd, csr, uimm } => {
            let csr_value = cpu.csr.read(csr).with_pc(pc).into_cpu_result()?;
            w(cpu, rd, csr_value);
            if uimm != 0 {
                cpu.csr
                    .clear_bits(csr, uimm as u64)
                    .with_pc(pc)
                    .into_cpu_result()?;
            }
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::Mret => {
            let mepc = cpu.csr.read(0x341).with_pc(pc).into_cpu_result()?; // mepc
            cpu.pc = mepc;
        }
        Instr::Fence => {
            // Memory fence - for in-order execution, this is a no-op
            cpu.pc = pc.wrapping_add(4);
            // TODO: once multiple harts, implement proper fencing
        }
    }

    // Keep x0 pinned (extra safety)
    cpu.regs[0] = 0;
    Ok(())
}
