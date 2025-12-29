use super::decode::Instr;
use crate::cpu::{Cpu, mem, trap::Trap};
use crate::mem::Memory;

pub fn execute(cpu: &mut Cpu, mem: &mut Memory, instr: Instr) -> Result<(), Trap> {
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
            let byte = mem!(pc, mem.read_u8(addr))?;
            let value = sign_extend(byte as i64, 8) as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LBU { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let byte = mem!(pc, mem.read_u8(addr))?;
            let value = byte as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LH { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let half = mem!(pc, mem.read_u16(addr))?;
            let value = sign_extend(half as i64, 16) as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LHU { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let half = mem!(pc, mem.read_u16(addr))?;
            let value = half as u64;
            w(cpu, rd, value);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::LD { rd, rs1, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let word = mem!(pc, mem.read_u64(addr))?;
            w(cpu, rd, word);
            cpu.pc = pc.wrapping_add(4);
        }
        Instr::SB { rs1, rs2, off } => {
            let addr = r(cpu, rs1).wrapping_add(off as u64);
            let byte = (r(cpu, rs2) & 0xff) as u8;
            mem!(pc, mem.write_u8(addr, byte))?;
            cpu.pc = pc.wrapping_add(4);
        }
        _ => todo!("execute: unimplemented instruction {:?}", instr),
    }

    // Keep x0 pinned (extra safety)
    cpu.regs[0] = 0;
    Ok(())
}
