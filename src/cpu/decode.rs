use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum DecodeError {
    InvalidOpcode { inst: u32 },
    InvalidFunct { inst: u32 },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidOpcode { inst } => write!(f, "invalid opcode: 0x{:08x}", inst),
            DecodeError::InvalidFunct { inst } => write!(f, "invalid function: 0x{:08x}", inst),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Instr {
    // ** RISC-V 32 & 64 Base Instructions **
    // R-type (0b0110011)
    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    Xor { rd: u8, rs1: u8, rs2: u8 },
    Or { rd: u8, rs1: u8, rs2: u8 },
    And { rd: u8, rs1: u8, rs2: u8 },
    Sll { rd: u8, rs1: u8, rs2: u8 },
    Srl { rd: u8, rs1: u8, rs2: u8 },
    Sra { rd: u8, rs1: u8, rs2: u8 },
    Slt { rd: u8, rs1: u8, rs2: u8 },
    Sltu { rd: u8, rs1: u8, rs2: u8 },
    // I-type arithmetic (0b0010011)
    Addi { rd: u8, rs1: u8, imm: i64 },
    Xori { rd: u8, rs1: u8, imm: i64 },
    Ori { rd: u8, rs1: u8, imm: i64 },
    Andi { rd: u8, rs1: u8, imm: i64 },
    Slli { rd: u8, rs1: u8, shamt: u8 },
    Srli { rd: u8, rs1: u8, shamt: u8 },
    Srai { rd: u8, rs1: u8, shamt: u8 },
    Slti { rd: u8, rs1: u8, imm: i64 },
    Sltiu { rd: u8, rs1: u8, imm: i64 },
    // I-type load (0b0000011)
    LB { rd: u8, rs1: u8, off: i64 },
    LBU { rd: u8, rs1: u8, off: i64 },
    LH { rd: u8, rs1: u8, off: i64 },
    LHU { rd: u8, rs1: u8, off: i64 },
    LW { rd: u8, rs1: u8, off: i64 },
    // S-type (0b0100011)
    SB { rs1: u8, rs2: u8, off: i64 },
    SH { rs1: u8, rs2: u8, off: i64 },
    SW { rs1: u8, rs2: u8, off: i64 },
    // B-type (0b1100011)
    Beq { rs1: u8, rs2: u8, off: i64 },
    Bne { rs1: u8, rs2: u8, off: i64 },
    Blt { rs1: u8, rs2: u8, off: i64 },
    Bge { rs1: u8, rs2: u8, off: i64 },
    Bltu { rs1: u8, rs2: u8, off: i64 },
    Bgeu { rs1: u8, rs2: u8, off: i64 },
    // J-type (0b1101111)
    Jal { rd: u8, off: i64 },
    // I-type jump (0b1100111)
    Jalr { rd: u8, rs1: u8, off: i64 },
    // U-type
    Lui { rd: u8, imm: i64 },   // 0b0110111
    Auipc { rd: u8, imm: i64 }, // 0b0010111
    // I-type environment
    Ecall,  // 0b1110011 with funct3=0 and imm=0
    Ebreak, // 0b1110011 with funct3=0 and imm=1
    // slt, slti, blt, bge require explicit signedness

    // ** RISC-V 64 Base Instructions **
    Addiw { rd: u8, rs1: u8, imm: i64 },
    Slliw { rd: u8, rs1: u8, shamt: u8 },
    Srliw { rd: u8, rs1: u8, shamt: u8 },
    Sraiw { rd: u8, rs1: u8, shamt: u8 },
    Addw { rd: u8, rs1: u8, rs2: u8 },
    Subw { rd: u8, rs1: u8, rs2: u8 },
    Sllw { rd: u8, rs1: u8, rs2: u8 },
    Srlw { rd: u8, rs1: u8, rs2: u8 },
    Sraw { rd: u8, rs1: u8, rs2: u8 },
    LWU { rd: u8, rs1: u8, off: i64 },
    LD { rd: u8, rs1: u8, off: i64 },
    SD { rs1: u8, rs2: u8, off: i64 },

    // CSR instructions
    Csrrw { rd: u8, csr: u16, rs1: u8 },
    Csrrs { rd: u8, csr: u16, rs1: u8 },
    Csrrc { rd: u8, csr: u16, rs1: u8 },
    Csrrwi { rd: u8, csr: u16, uimm: u8 },
    Csrrsi { rd: u8, csr: u16, uimm: u8 },
    Csrrci { rd: u8, csr: u16, uimm: u8 },
    Mret,
    // Atomic/Memory instructions
    Fence, // 0b0001111 - No-op for now
}

fn sign_extend(value: i64, bits: u32) -> i64 {
    let shift = 64 - bits;
    (value << shift) >> shift
}

pub fn decode(_pc: u64, inst: u32) -> Result<Instr, DecodeError> {
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
                (0x4, 0x00) => Ok(Instr::Xor { rd, rs1, rs2 }),
                (0x6, 0x00) => Ok(Instr::Or { rd, rs1, rs2 }),
                (0x7, 0x00) => Ok(Instr::And { rd, rs1, rs2 }),
                (0x1, 0x00) => Ok(Instr::Sll { rd, rs1, rs2 }),
                (0x5, 0x00) => Ok(Instr::Srl { rd, rs1, rs2 }),
                (0x5, 0x20) => Ok(Instr::Sra { rd, rs1, rs2 }),
                (0x2, 0x00) => Ok(Instr::Slt { rd, rs1, rs2 }),
                (0x3, 0x00) => Ok(Instr::Sltu { rd, rs1, rs2 }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        // i type
        0b0010011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let imm = sign_extend((inst >> 20) as i64, 12);

            match funct3 {
                0x0 => Ok(Instr::Addi { rd, rs1, imm }),
                0x4 => Ok(Instr::Xori { rd, rs1, imm }),
                0x6 => Ok(Instr::Ori { rd, rs1, imm }),
                0x7 => Ok(Instr::Andi { rd, rs1, imm }),
                0x1 => Ok(Instr::Slli {
                    rd,
                    rs1,
                    shamt: (imm & 0x3f) as u8,
                }),
                0x5 => {
                    let shamt = (imm & 0x3f) as u8;
                    let funct7 = ((imm >> 6) & 0x7f) as u8;
                    match funct7 {
                        0x00 => Ok(Instr::Srli { rd, rs1, shamt }),
                        0x20 => Ok(Instr::Srai { rd, rs1, shamt }),
                        _ => Err(DecodeError::InvalidOpcode { inst }),
                    }
                }
                0x2 => Ok(Instr::Slti { rd, rs1, imm }),
                0x3 => Ok(Instr::Sltiu { rd, rs1, imm }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        0b0000011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let imm = sign_extend((inst >> 20) as i64, 12);

            match funct3 {
                0x0 => Ok(Instr::LB { rd, rs1, off: imm }),
                0x4 => Ok(Instr::LBU { rd, rs1, off: imm }),
                0x1 => Ok(Instr::LH { rd, rs1, off: imm }),
                0x5 => Ok(Instr::LHU { rd, rs1, off: imm }),
                0x2 => Ok(Instr::LW { rd, rs1, off: imm }),
                // rv64 extensions
                0x6 => Ok(Instr::LWU { rd, rs1, off: imm }),
                0x3 => Ok(Instr::LD { rd, rs1, off: imm }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
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
                sign_extend(((imm11_5 << 5) | imm4_0) as i64, 12)
            };

            match funct3 {
                0x0 => Ok(Instr::SB { rs1, rs2, off: imm }),
                0x1 => Ok(Instr::SH { rs1, rs2, off: imm }),
                0x2 => Ok(Instr::SW { rs1, rs2, off: imm }),
                // rv64 extension
                0x3 => Ok(Instr::SD { rs1, rs2, off: imm }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
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
                    ((imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1)) as i64,
                    13,
                )
            };

            match funct3 {
                0x0 => Ok(Instr::Beq { rs1, rs2, off: imm }),
                0x1 => Ok(Instr::Bne { rs1, rs2, off: imm }),
                0x4 => Ok(Instr::Blt { rs1, rs2, off: imm }),
                0x5 => Ok(Instr::Bge { rs1, rs2, off: imm }),
                0x6 => Ok(Instr::Bltu { rs1, rs2, off: imm }),
                0x7 => Ok(Instr::Bgeu { rs1, rs2, off: imm }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        // u type
        0b0110111 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let imm = sign_extend((inst & 0xfffff000) as i64, 32);
            Ok(Instr::Lui { rd, imm })
        }
        0b0010111 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let imm = sign_extend((inst & 0xfffff000) as i64, 32);
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
                    ((imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1)) as i64,
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
            let imm = sign_extend((inst >> 20) as i64, 12);
            match funct3 {
                0x0 => Ok(Instr::Jalr { rd, rs1, off: imm }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        // i type environment
        0b1110011 => {
            let funct3 = ((inst >> 12) & 0x7) as u8;
            match funct3 {
                0x0 => {
                    let imm = inst >> 20;
                    match imm {
                        0x000 => Ok(Instr::Ecall),
                        0x001 => Ok(Instr::Ebreak),
                        0x302 => Ok(Instr::Mret),
                        _ => Err(DecodeError::InvalidOpcode { inst }),
                    }
                }
                0x1 => {
                    let rd = ((inst >> 7) & 0x1f) as u8;
                    let csr = (inst >> 20) as u16;
                    let rs1 = ((inst >> 15) & 0x1f) as u8;
                    Ok(Instr::Csrrw { rd, csr, rs1 })
                }
                0x2 => {
                    let rd = ((inst >> 7) & 0x1f) as u8;
                    let csr = (inst >> 20) as u16;
                    let rs1 = ((inst >> 15) & 0x1f) as u8;
                    Ok(Instr::Csrrs { rd, csr, rs1 })
                }
                0x3 => {
                    let rd = ((inst >> 7) & 0x1f) as u8;
                    let csr = (inst >> 20) as u16;
                    let rs1 = ((inst >> 15) & 0x1f) as u8;
                    Ok(Instr::Csrrc { rd, csr, rs1 })
                }
                0x5 => {
                    let rd = ((inst >> 7) & 0x1f) as u8;
                    let csr = (inst >> 20) as u16;
                    let uimm = ((inst >> 15) & 0x1f) as u8;
                    Ok(Instr::Csrrwi { rd, csr, uimm })
                }
                0x6 => {
                    let rd = ((inst >> 7) & 0x1f) as u8;
                    let csr = (inst >> 20) as u16;
                    let uimm = ((inst >> 15) & 0x1f) as u8;
                    Ok(Instr::Csrrsi { rd, csr, uimm })
                }
                0x7 => {
                    let rd = ((inst >> 7) & 0x1f) as u8;
                    let csr = (inst >> 20) as u16;
                    let uimm = ((inst >> 15) & 0x1f) as u8;
                    Ok(Instr::Csrrci { rd, csr, uimm })
                }
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        0b0011011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let imm = sign_extend((inst >> 20) as i64, 12);

            match funct3 {
                0x0 => Ok(Instr::Addiw { rd, rs1, imm }),
                0x1 => {
                    let shamt = (imm & 0x1f) as u8;
                    let funct7 = ((imm >> 5) & 0x7f) as u8;
                    match funct7 {
                        0x00 => Ok(Instr::Slliw { rd, rs1, shamt }),
                        _ => Err(DecodeError::InvalidOpcode { inst }),
                    }
                }
                0x5 => {
                    let shamt = (imm & 0x1f) as u8;
                    let funct7 = ((imm >> 5) & 0x7f) as u8;
                    match funct7 {
                        0x00 => Ok(Instr::Srliw { rd, rs1, shamt }),
                        0x20 => Ok(Instr::Sraiw { rd, rs1, shamt }),
                        _ => Err(DecodeError::InvalidOpcode { inst }),
                    }
                }
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        0b0111011 => {
            let rd = ((inst >> 7) & 0x1f) as u8;
            let funct3 = ((inst >> 12) & 0x7) as u8;
            let rs1 = ((inst >> 15) & 0x1f) as u8;
            let rs2 = ((inst >> 20) & 0x1f) as u8;
            let funct7 = ((inst >> 25) & 0x7f) as u8;
            match (funct3, funct7) {
                (0x0, 0x00) => Ok(Instr::Addw { rd, rs1, rs2 }),
                (0x0, 0x20) => Ok(Instr::Subw { rd, rs1, rs2 }),
                (0x1, 0x00) => Ok(Instr::Sllw { rd, rs1, rs2 }),
                (0x5, 0x00) => Ok(Instr::Srlw { rd, rs1, rs2 }),
                (0x5, 0x20) => Ok(Instr::Sraw { rd, rs1, rs2 }),
                _ => Err(DecodeError::InvalidOpcode { inst }),
            }
        }
        // Fence instruction (0b0001111)
        0b0001111 => Ok(Instr::Fence),
        _ => Err(DecodeError::InvalidOpcode { inst }),
    }
}
