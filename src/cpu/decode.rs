use super::trap::Trap;

#[derive(Clone, Copy, Debug)]
pub enum Instr {
    // R-type (0b0110011)
    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    // I-type arithmetic (0b0010011)
    Addi { rd: u8, rs1: u8, imm: u64 },
    // I-type load (0b0000011)
    LB { rd: u8, rs1: u8, off: u64 },
    LBU { rd: u8, rs1: u8, off: u64 },
    LH { rd: u8, rs1: u8, off: u64 },
    LHU { rd: u8, rs1: u8, off: u64 },
    LD { rd: u8, rs1: u8, off: u64 },
    // S-type (0b0100011)
    SB { rs1: u8, rs2: u8, off: u64 },
    SH { rs1: u8, rs2: u8, off: u64 },
    SW { rs1: u8, rs2: u8, off: u64 },
    SD { rs1: u8, rs2: u8, off: u64 },
    // B-type (0b1100011)
    Beq { rs1: u8, rs2: u8, off: u64 },
    Bne { rs1: u8, rs2: u8, off: u64 },
    // TODO: later BLT, BGE
    // U-type
    Lui { rd: u8, imm: u64 },   // 0b0110111
    Auipc { rd: u8, imm: u64 }, // 0b0010111
    // J-type (0b1101111)
    Jal { rd: u8, off: u64 },
    // I-type jump (0b1100111)
    Jalr { rd: u8, rs1: u8, off: u64 },
    // note that most are u64 for type uniformity
    // and doesn't matter for instructions, but
    // technically is signed in theory

    // slt, slti, blt, bge require explicit signedness
}

fn sign_extend(value: u64, bits: u32) -> u64 {
    let shift = 64 - bits;
    ((value << shift) as i64 >> shift) as u64
}

pub fn decode(pc: u64, inst: u32) -> Result<Instr, Trap> {
    let opcode = inst & 0x7f;
    match opcode {
        // r type
        0b0110011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let rs2 = ((inst >> 20) & 0x1f) as u8;
            let funct7 = ((inst >> 25) & 0x7f) as u8;

            match (funct3, funct7) {
                (0x0, 0x00) => Ok(Instr::Add { rd, rs1, rs2 }),
                (0x0, 0x20) => Ok(Instr::Sub { rd, rs1, rs2 }),
                _ => Err(Trap::IllegalInstruction { pc, inst }),
            }
        }
        // i type
        0b0010011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let imm = sign_extend((inst >> 20) as u64, 12);

            match funct3 {
                0x0 => Ok(Instr::Addi { rd, rs1, imm }),
                _ => Err(Trap::IllegalInstruction { pc, inst }),
            }
        }
        0b0000011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let imm = sign_extend((inst >> 20) as u64, 12);

            match funct3 {
                0x0 => Ok(Instr::LB { rd, rs1, off: imm }),
                0x4 => Ok(Instr::LBU { rd, rs1, off: imm }),
                0x1 => Ok(Instr::LH { rd, rs1, off: imm }),
                0x5 => Ok(Instr::LHU { rd, rs1, off: imm }),
                0x3 => Ok(Instr::LD { rd, rs1, off: imm }),
                _ => Err(Trap::IllegalInstruction { pc, inst }),
            }
        }
        // s type
        0b0100011 => {
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let rs2 = ((inst >> 20) & 0x1f) as u8;
            let imm = {
                let imm4_0 = (inst >> 7) & 0x1f;
                let imm11_5 = (inst >> 25) & 0x7f;
                sign_extend(((imm11_5 << 5) | imm4_0) as u64, 12)
            };

            match funct3 {
                0x0 => Ok(Instr::SB { rs1, rs2, off: imm }),
                0x1 => Ok(Instr::SH { rs1, rs2, off: imm }),
                0x3 => Ok(Instr::SW { rs1, rs2, off: imm }),
                0x7 => Ok(Instr::SD { rs1, rs2, off: imm }),
                _ => Err(Trap::IllegalInstruction { pc, inst }),
            }
        }
        //  b type
        0b1100011 => {
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let rs2 = ((inst >> 20) & 0x1f) as u8;
            let imm = {
                let imm11 = (inst >> 7) & 0x1;
                let imm4_1 = (inst >> 8) & 0xf;
                let imm10_5 = (inst >> 25) & 0x3f;
                let imm12 = (inst >> 31) & 0x1;
                sign_extend(
                    ((imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1)) as u64,
                    13,
                )
            };

            match funct3 {
                0x0 => Ok(Instr::Beq { rs1, rs2, off: imm }),
                0x1 => Ok(Instr::Bne { rs1, rs2, off: imm }),
                _ => Err(Trap::IllegalInstruction { pc, inst }),
            }
        }
        // u type
        0b0110111 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let imm = sign_extend((inst & 0xfffff000) as u64, 32);
            Ok(Instr::Lui { rd, imm })
        }
        0b0010111 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let imm = sign_extend((inst & 0xfffff000) as u64, 32);
            Ok(Instr::Auipc { rd, imm })
        }
        // j type
        0b1101111 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let imm = {
                let imm19_12 = (inst >> 12) & 0xff;
                let imm11 = (inst >> 20) & 0x1;
                let imm10_1 = (inst >> 21) & 0x3ff;
                let imm20 = (inst >> 31) & 0x1;
                sign_extend(
                    ((imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1)) as u64,
                    21,
                )
            };
            Ok(Instr::Jal { rd, off: imm })
        }
        // i type jalr
        0b1100111 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let imm = sign_extend((inst >> 20) as u64, 12);
            match funct3 {
                0x0 => Ok(Instr::Jalr { rd, rs1, off: imm }),
                _ => Err(Trap::IllegalInstruction { pc, inst }),
            }
        }
        // TODO: more opcodes
        _ => Err(Trap::IllegalInstruction { pc, inst }),
    }
}
