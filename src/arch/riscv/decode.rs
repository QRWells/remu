use super::instruction::{Ordering, RiscvInst};

fn rd(bits: u32) -> u8 {
    ((bits >> 7) & 0b11111) as u8
}

fn rs1(bits: u32) -> u8 {
    ((bits >> 15) & 0b11111) as u8
}

fn rs2(bits: u32) -> u8 {
    ((bits >> 20) & 0b11111) as u8
}

fn rs3(bits: u32) -> u8 {
    ((bits >> 27) & 0b11111) as u8
}

fn funct3(bits: u32) -> u32 {
    (bits >> 12) & 0b111
}

fn funct7(bits: u32) -> u32 {
    (bits >> 25) & 0b1111111
}

fn csr(bits: u32) -> u16 {
    (bits >> 20) as u16
}

fn i_imm(bits: u32) -> i32 {
    (bits as i32) >> 20
}

fn s_imm(bits: u32) -> i32 {
    ((bits & 0b11111110_00000000_00000000_00000000) as i32) >> 20
        | ((bits & 0b00000000_00000000_00001111_10000000) as i32) >> 7
}

fn b_imm(bits: u32) -> i32 {
    ((bits & 0b10000000_00000000_00000000_00000000) as i32) >> 19
        | ((bits & 0b00000000_00000000_00000000_10000000) as i32) << 4
        | ((bits & 0b01111110_00000000_00000000_00000000) as i32) >> 20
        | ((bits & 0b00000000_00000000_00001111_00000000) as i32) >> 7
}

fn u_imm(bits: u32) -> i32 {
    (bits & 0xfffff000) as i32
}

fn j_imm(instr: u32) -> i32 {
    ((instr & 0b10000000_00000000_00000000_00000000) as i32) >> 11
        | ((instr & 0b00000000_00001111_11110000_00000000) as i32) >> 0
        | ((instr & 0b00000000_00010000_00000000_00000000) as i32) >> 9
        | ((instr & 0b01111111_11100000_00000000_00000000) as i32) >> 20
}

fn c_funct3(bits: u16) -> u32 {
    ((bits >> 13) & 0b111) as u32
}

fn c_rd(bits: u16) -> u8 {
    ((bits >> 7) & 0b11111) as u8
}

fn c_rs1(bits: u16) -> u8 {
    c_rd(bits)
}

fn c_rs2(bits: u16) -> u8 {
    ((bits >> 2) & 0b11111) as u8
}

fn c_rds(bits: u16) -> u8 {
    ((bits >> 2) & 0b111) as u8 + 8
}

fn c_rs1s(bits: u16) -> u8 {
    ((bits >> 7) & 0b111) as u8 + 8
}

fn c_rs2s(bits: u16) -> u8 {
    c_rds(bits)
}

fn ci_imm(bits: u16) -> i32 {
    ((bits & 0b00010000_00000000) as i32) << (31 - 12) >> (31 - 5)
        | ((bits & 0b00000000_01111100) as i32) >> 2
}

fn ci_lwsp_imm(bits: u16) -> i32 {
    ((bits & 0b00000000_00001100) as i32) << 4
        | ((bits & 0b00010000_00000000) as i32) >> 7
        | ((bits & 0b00000000_01110000) as i32) >> 2
}

fn ci_ldsp_imm(bits: u16) -> i32 {
    ((bits & 0b00000000_00011100) as i32) << 4
        | ((bits & 0b00010000_00000000) as i32) >> 7
        | ((bits & 0b00000000_01100000) as i32) >> 2
}

fn ci_addi16sp_imm(bits: u16) -> i32 {
    ((bits & 0b00010000_00000000) as i32) << (31 - 12) >> (31 - 9)
        | ((bits & 0b00000000_00011000) as i32) << 4
        | ((bits & 0b00000000_00100000) as i32) << 1
        | ((bits & 0b00000000_00000100) as i32) << 3
        | ((bits & 0b00000000_01000000) as i32) >> 2
}

fn css_swsp_imm(bits: u16) -> i32 {
    ((bits & 0b00000001_10000000) as i32) >> 1 | ((bits & 0b00011110_00000000) as i32) >> 7
}

fn css_sdsp_imm(bits: u16) -> i32 {
    ((bits & 0b00000011_10000000) as i32) >> 1 | ((bits & 0b00011100_00000000) as i32) >> 7
}

fn ciw_imm(bits: u16) -> i32 {
    ((bits & 0b00000111_10000000) as i32) >> 1
        | ((bits & 0b00011000_00000000) as i32) >> 7
        | ((bits & 0b00000000_00100000) as i32) >> 2
        | ((bits & 0b00000000_01000000) as i32) >> 4
}

fn cl_lw_imm(bits: u16) -> i32 {
    ((bits & 0b00000000_00100000) as i32) << 1
        | ((bits & 0b00011100_00000000) as i32) >> 7
        | ((bits & 0b00000000_01000000) as i32) >> 4
}

fn cl_ld_imm(bits: u16) -> i32 {
    ((bits & 0b00000000_01100000) as i32) << 1 | ((bits & 0b00011100_00000000) as i32) >> 7
}

fn cs_sw_imm(bits: u16) -> i32 {
    cl_lw_imm(bits)
}

fn cs_sd_imm(bits: u16) -> i32 {
    cl_ld_imm(bits)
}

fn cb_imm(bits: u16) -> i32 {
    ((bits & 0b00010000_00000000) as i32) << (31 - 12) >> (31 - 8)
        | ((bits & 0b00000000_01100000) as i32) << 1
        | ((bits & 0b00000000_00000100) as i32) << 3
        | ((bits & 0b00001100_00000000) as i32) >> 7
        | ((bits & 0b00000000_00011000) as i32) >> 2
}

fn cj_imm(bits: u16) -> i32 {
    ((bits & 0b00010000_00000000) as i32) << (31 - 12) >> (31 - 11)
        | ((bits & 0b00000001_00000000) as i32) << 2
        | ((bits & 0b00000110_00000000) as i32) >> 1
        | ((bits & 0b00000000_01000000) as i32) << 1
        | ((bits & 0b00000000_10000000) as i32) >> 1
        | ((bits & 0b00000000_00000100) as i32) << 3
        | ((bits & 0b00001000_00000000) as i32) >> 7
        | ((bits & 0b00000000_00111000) as i32) >> 2
}

pub fn decode_compressed(bits: u16) -> RiscvInst {
    let function = c_funct3(bits);

    match bits & 0b11 {
        0b00 => {
            match function {
                0b000 => {
                    let imm = ciw_imm(bits);
                    if imm == 0 {
                        // Illegal instruction
                        return RiscvInst::Illegal;
                    }
                    // C.ADDI4SPN
                    // translate to addi rd', x2, imm
                    RiscvInst::Addi {
                        rd: c_rds(bits),
                        rs1: 2,
                        imm,
                    }
                }
                0b001 => {
                    // C.FLD
                    // translate to fld rd', rs1', offset
                    RiscvInst::Fld {
                        frd: c_rds(bits),
                        rs1: c_rs1s(bits),
                        imm: cl_ld_imm(bits),
                    }
                }
                0b010 => {
                    // C.LW
                    // translate to lw rd', rs1', offset
                    RiscvInst::Lw {
                        rd: c_rds(bits),
                        rs1: c_rs1s(bits),
                        imm: cl_lw_imm(bits),
                    }
                }
                0b011 => {
                    // C.LD
                    // translate to ld rd', rs1', offset
                    RiscvInst::Ld {
                        rd: c_rds(bits),
                        rs1: c_rs1s(bits),
                        imm: cl_ld_imm(bits),
                    }
                }
                0b100 => {
                    // Reserved
                    RiscvInst::Illegal
                }
                0b101 => {
                    // C.FSD
                    // translate to fsd rs2', rs1', offset
                    RiscvInst::Fsd {
                        rs1: c_rs1s(bits),
                        frs2: c_rs2s(bits),
                        imm: cs_sd_imm(bits),
                    }
                }
                0b110 => {
                    // C.SW
                    // translate to sw rs2', rs1', offset
                    RiscvInst::Sw {
                        rs1: c_rs1s(bits),
                        rs2: c_rs2s(bits),
                        imm: cs_sw_imm(bits),
                    }
                }
                0b111 => {
                    // C.SD
                    // translate to sd rs2', rs1', offset
                    RiscvInst::Sd {
                        rs1: c_rs1s(bits),
                        rs2: c_rs2s(bits),
                        imm: cs_sd_imm(bits),
                    }
                }
                // full case
                _ => unreachable!(),
            }
        }
        0b01 => {
            match function {
                0b000 => {
                    // rd = x0 is HINT
                    // r0 = 0 is C.NOP
                    // C.ADDI
                    // translate to addi rd, rd, imm
                    let rd = c_rd(bits);
                    RiscvInst::Addi {
                        rd,
                        rs1: rd,
                        imm: ci_imm(bits),
                    }
                }
                0b001 => {
                    let rd = c_rd(bits);
                    if rd == 0 {
                        // Reserved
                        return RiscvInst::Illegal;
                    }
                    // C.ADDIW
                    // translate to addiw rd, rd, imm
                    RiscvInst::Addiw {
                        rd,
                        rs1: rd,
                        imm: ci_imm(bits),
                    }
                }
                0b010 => {
                    // rd = x0 is HINT
                    // C.LI
                    // translate to addi rd, x0, imm
                    RiscvInst::Addi {
                        rd: c_rd(bits),
                        rs1: 0,
                        imm: ci_imm(bits),
                    }
                }
                0b011 => {
                    let rd = c_rd(bits);
                    if rd == 2 {
                        let imm = ci_addi16sp_imm(bits);
                        if imm == 0 {
                            // Reserved
                            return RiscvInst::Illegal;
                        }
                        // C.ADDI16SP
                        // translate to addi x2, x2, imm
                        RiscvInst::Addi { rd: 2, rs1: 2, imm }
                    } else {
                        // rd = x0 is HINT
                        // C.LUI
                        // translate to lui rd, imm
                        RiscvInst::Lui {
                            rd,
                            imm: ci_imm(bits) << 12,
                        }
                    }
                }
                0b100 => {
                    let rs1 = c_rs1s(bits);
                    match (bits >> 10) & 0b11 {
                        0b00 => {
                            // imm = 0 is HINT
                            // C.SRLI
                            // translate to srli rs1', rs1', imm
                            RiscvInst::Srli {
                                rd: rs1,
                                rs1,
                                imm: ci_imm(bits) & 63,
                            }
                        }
                        0b01 => {
                            // imm = 0 is HINT
                            // C.SRAI
                            // translate to srai rs1', rs1', imm
                            RiscvInst::Srai {
                                rd: rs1,
                                rs1,
                                imm: ci_imm(bits) & 63,
                            }
                        }
                        0b10 => {
                            // C.ANDI
                            // translate to andi rs1', rs1', imm
                            RiscvInst::Andi {
                                rd: rs1,
                                rs1,
                                imm: ci_imm(bits),
                            }
                        }
                        0b11 => {
                            if (bits & 0x1000) == 0 {
                                // C.SUB
                                // C.XOR
                                // C.OR
                                // C.AND
                                // translates to [OP] rs1', rs1', rs2'
                                let rs2 = c_rs2s(bits);
                                match (bits >> 5) & 0b11 {
                                    0b00 => RiscvInst::Sub { rd: rs1, rs1, rs2 },
                                    0b01 => RiscvInst::Xor { rd: rs1, rs1, rs2 },
                                    0b10 => RiscvInst::Or { rd: rs1, rs1, rs2 },
                                    0b11 => RiscvInst::And { rd: rs1, rs1, rs2 },
                                    // full case
                                    _ => unreachable!(),
                                }
                            } else {
                                // C.SUBW
                                // C.ADDW
                                let rs2 = c_rs2s(bits);
                                match (bits >> 5) & 0b11 {
                                    0b00 => RiscvInst::Subw { rd: rs1, rs1, rs2 },
                                    0b01 => RiscvInst::Addw { rd: rs1, rs1, rs2 },
                                    _ => RiscvInst::Illegal,
                                }
                            }
                        }
                        // full case
                        _ => unreachable!(),
                    }
                }
                0b101 => {
                    // C.J
                    // translate to jal x0, imm
                    RiscvInst::Jal {
                        rd: 0,
                        imm: cj_imm(bits),
                    }
                }
                0b110 => {
                    // C.BEQZ
                    // translate to beq rs1', x0, imm
                    RiscvInst::Beq {
                        rs1: c_rs1s(bits),
                        rs2: 0,
                        imm: cb_imm(bits),
                    }
                }
                0b111 => {
                    // C.BNEZ
                    // translate to bne rs1', x0, imm
                    RiscvInst::Bne {
                        rs1: c_rs1s(bits),
                        rs2: 0,
                        imm: cb_imm(bits),
                    }
                }
                // full case
                _ => unreachable!(),
            }
        }
        0b10 => {
            match function {
                0b000 => {
                    // imm = 0 is HINT
                    // rd = 0 is HINT
                    // C.SLLI
                    // translates to slli rd, rd, imm
                    let rd = c_rd(bits);
                    RiscvInst::Slli {
                        rd,
                        rs1: rd,
                        imm: ci_imm(bits) & 63,
                    }
                }
                0b001 => {
                    // C.FLDSP
                    // translate to fld rd, x2, imm
                    RiscvInst::Fld {
                        frd: c_rd(bits),
                        rs1: 2,
                        imm: ci_ldsp_imm(bits),
                    }
                }
                0b010 => {
                    let rd = c_rd(bits);
                    if rd == 0 {
                        // Reserved
                        return RiscvInst::Illegal;
                    }
                    // C.LWSP
                    // translate to lw rd, x2, imm
                    RiscvInst::Lw {
                        rd,
                        rs1: 2,
                        imm: ci_lwsp_imm(bits),
                    }
                }
                0b011 => {
                    let rd = c_rd(bits);
                    if rd == 0 {
                        // Reserved
                        return RiscvInst::Illegal;
                    }
                    // C.LDSP
                    // translate to ld rd, x2, imm
                    RiscvInst::Ld {
                        rd,
                        rs1: 2,
                        imm: ci_ldsp_imm(bits),
                    }
                }
                0b100 => {
                    let rs2 = c_rs2(bits);
                    if (bits & 0x1000) == 0 {
                        if rs2 == 0 {
                            let rs1 = c_rs1(bits);
                            if rs1 == 0 {
                                // Reserved
                                return RiscvInst::Illegal;
                            }
                            // C.JR
                            // translate to jalr x0, rs1, 0
                            RiscvInst::Jalr { rd: 0, rs1, imm: 0 }
                        } else {
                            // rd = 0 is HINT
                            // C.MV
                            // translate to add rd, x0, rs2
                            RiscvInst::Add {
                                rd: c_rd(bits),
                                rs1: 0,
                                rs2,
                            }
                        }
                    } else {
                        let rs1 = c_rs1(bits);
                        if rs1 == 0 {
                            // C.EBREAK
                            RiscvInst::Ebreak
                        } else if rs2 == 0 {
                            // C.JALR
                            // translate to jalr x1, rs1, 0
                            RiscvInst::Jalr { rd: 1, rs1, imm: 0 }
                        } else {
                            // rd = 0 is HINT
                            // C.ADD
                            // translate to add rd, rd, rs2
                            let rd = c_rd(bits);
                            RiscvInst::Add { rd, rs1: rd, rs2 }
                        }
                    }
                }
                0b101 => {
                    // C.FSDSP
                    // translate to fsd rs2, x2, imm
                    RiscvInst::Fsd {
                        rs1: 2,
                        frs2: c_rs2(bits),
                        imm: css_sdsp_imm(bits),
                    }
                }
                0b110 => {
                    // C.SWSP
                    // translate to sw rs2, x2, imm
                    RiscvInst::Sw {
                        rs1: 2,
                        rs2: c_rs2(bits),
                        imm: css_swsp_imm(bits),
                    }
                }
                0b111 => {
                    // C.SDSP
                    // translate to sd rs2, x2, imm
                    RiscvInst::Sd {
                        rs1: 2,
                        rs2: c_rs2(bits),
                        imm: css_sdsp_imm(bits),
                    }
                }
                // full case
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

pub fn decode(bits: u32) -> RiscvInst {
    macro_rules! rm {
        ($rm: expr) => {{
            let rm = $rm as u8;
            if rm > 4 && rm != 0b111 {
                return RiscvInst::Illegal;
            }
            rm
        }};
    }

    // We shouldn't see compressed ops here
    assert!(bits & 3 == 3);

    // Longer ops, treat them as illegal ops
    if bits & 0x1f == 0x1f {
        return RiscvInst::Illegal;
    }

    let function = funct3(bits);
    let rd = rd(bits);
    let rs1 = rs1(bits);
    let rs2 = rs2(bits);

    match bits & 0b1111111 {
        /* LOAD */
        0b0000011 => {
            let imm = i_imm(bits);
            match function {
                0b000 => RiscvInst::Lb { rd, rs1, imm },
                0b001 => RiscvInst::Lh { rd, rs1, imm },
                0b010 => RiscvInst::Lw { rd, rs1, imm },
                0b011 => RiscvInst::Ld { rd, rs1, imm },
                0b100 => RiscvInst::Lbu { rd, rs1, imm },
                0b101 => RiscvInst::Lhu { rd, rs1, imm },
                0b110 => RiscvInst::Lwu { rd, rs1, imm },
                _ => RiscvInst::Illegal,
            }
        }

        /* LOAD-FP */
        0b0000111 => {
            let imm = i_imm(bits);
            match function {
                0b010 => RiscvInst::Flw { frd: rd, rs1, imm },
                0b011 => RiscvInst::Fld { frd: rd, rs1, imm },
                _ => RiscvInst::Illegal,
            }
        }

        /* OP-IMM */
        0b0010011 => {
            let imm = i_imm(bits);
            match function {
                0b000 => RiscvInst::Addi { rd, rs1, imm },
                0b001 => {
                    if imm >= 64 {
                        RiscvInst::Illegal
                    } else {
                        RiscvInst::Slli { rd, rs1, imm }
                    }
                }
                0b010 => RiscvInst::Slti { rd, rs1, imm },
                0b011 => RiscvInst::Sltiu { rd, rs1, imm },
                0b100 => RiscvInst::Xori { rd, rs1, imm },
                0b101 => {
                    if imm & !0x400 >= 64 {
                        RiscvInst::Illegal
                    } else if (imm & 0x400) != 0 {
                        RiscvInst::Srai {
                            rd,
                            rs1,
                            imm: imm & !0x400,
                        }
                    } else {
                        RiscvInst::Srli { rd, rs1, imm }
                    }
                }
                0b110 => RiscvInst::Ori { rd, rs1, imm },
                0b111 => RiscvInst::Andi { rd, rs1, imm },
                // full case
                _ => unreachable!(),
            }
        }

        /* MISC-MEM */
        0b0001111 => {
            match function {
                0b000 => {
                    // TODO Multiple types of fence
                    RiscvInst::Fence
                }
                0b001 => RiscvInst::FenceI,
                _ => RiscvInst::Illegal,
            }
        }

        /* OP-IMM-32 */
        0b0011011 => {
            let imm = i_imm(bits);
            match function {
                0b000 => RiscvInst::Addiw { rd, rs1, imm },
                0b001 => {
                    if imm >= 32 {
                        RiscvInst::Illegal
                    } else {
                        RiscvInst::Slliw { rd, rs1, imm }
                    }
                }
                0b101 => {
                    if imm & !0x400 >= 32 {
                        RiscvInst::Illegal
                    } else if (imm & 0x400) != 0 {
                        RiscvInst::Sraiw {
                            rd,
                            rs1,
                            imm: imm & !0x400,
                        }
                    } else {
                        RiscvInst::Srliw { rd, rs1, imm }
                    }
                }
                _ => RiscvInst::Illegal,
            }
        }

        /* STORE */
        0b0100011 => {
            let imm = s_imm(bits);
            match function {
                0b000 => RiscvInst::Sb { rs1, rs2, imm },
                0b001 => RiscvInst::Sh { rs1, rs2, imm },
                0b010 => RiscvInst::Sw { rs1, rs2, imm },
                0b011 => RiscvInst::Sd { rs1, rs2, imm },
                _ => RiscvInst::Illegal,
            }
        }

        /* STORE-FP */
        0b0100111 => {
            let imm = s_imm(bits);
            match function {
                0b010 => RiscvInst::Fsw {
                    rs1,
                    frs2: rs2,
                    imm,
                },
                0b011 => RiscvInst::Fsd {
                    rs1,
                    frs2: rs2,
                    imm,
                },
                _ => RiscvInst::Illegal,
            }
        }

        /* Base Opcode AMO */
        0b0101111 => {
            /* A-Extension */
            let func = funct7(bits) >> 2;
            let aqrl = match funct7(bits) & 3 {
                0 => Ordering::Relaxed,
                1 => Ordering::Release,
                2 => Ordering::Acquire,
                3 => Ordering::SeqCst,
                _ => unreachable!(),
            };
            if function == 0b010 {
                match func {
                    0b00010 => {
                        if rs2 != 0 {
                            RiscvInst::Illegal
                        } else {
                            RiscvInst::LrW { rd, rs1, aqrl }
                        }
                    }
                    0b00011 => RiscvInst::ScW { rd, rs1, rs2, aqrl },
                    0b00001 => RiscvInst::AmoswapW { rd, rs1, rs2, aqrl },
                    0b00000 => RiscvInst::AmoaddW { rd, rs1, rs2, aqrl },
                    0b00100 => RiscvInst::AmoxorW { rd, rs1, rs2, aqrl },
                    0b01100 => RiscvInst::AmoandW { rd, rs1, rs2, aqrl },
                    0b01000 => RiscvInst::AmoorW { rd, rs1, rs2, aqrl },
                    0b10000 => RiscvInst::AmominW { rd, rs1, rs2, aqrl },
                    0b10100 => RiscvInst::AmomaxW { rd, rs1, rs2, aqrl },
                    0b11000 => RiscvInst::AmominuW { rd, rs1, rs2, aqrl },
                    0b11100 => RiscvInst::AmomaxuW { rd, rs1, rs2, aqrl },
                    _ => RiscvInst::Illegal,
                }
            } else if function == 0b011 {
                match func {
                    0b00010 => {
                        if rs2 != 0 {
                            RiscvInst::Illegal
                        } else {
                            RiscvInst::LrD { rd, rs1, aqrl }
                        }
                    }
                    0b00011 => RiscvInst::ScD { rd, rs1, rs2, aqrl },
                    0b00001 => RiscvInst::AmoswapD { rd, rs1, rs2, aqrl },
                    0b00000 => RiscvInst::AmoaddD { rd, rs1, rs2, aqrl },
                    0b00100 => RiscvInst::AmoxorD { rd, rs1, rs2, aqrl },
                    0b01100 => RiscvInst::AmoandD { rd, rs1, rs2, aqrl },
                    0b01000 => RiscvInst::AmoorD { rd, rs1, rs2, aqrl },
                    0b10000 => RiscvInst::AmominD { rd, rs1, rs2, aqrl },
                    0b10100 => RiscvInst::AmomaxD { rd, rs1, rs2, aqrl },
                    0b11000 => RiscvInst::AmominuD { rd, rs1, rs2, aqrl },
                    0b11100 => RiscvInst::AmomaxuD { rd, rs1, rs2, aqrl },
                    _ => RiscvInst::Illegal,
                }
            } else {
                RiscvInst::Illegal
            }
        }

        /* OP */
        0b0110011 => {
            match funct7(bits) {
                // M-extension
                0b0000001 => match function {
                    0b000 => RiscvInst::Mul { rd, rs1, rs2 },
                    0b001 => RiscvInst::Mulh { rd, rs1, rs2 },
                    0b010 => RiscvInst::Mulhsu { rd, rs1, rs2 },
                    0b011 => RiscvInst::Mulhu { rd, rs1, rs2 },
                    0b100 => RiscvInst::Div { rd, rs1, rs2 },
                    0b101 => RiscvInst::Divu { rd, rs1, rs2 },
                    0b110 => RiscvInst::Rem { rd, rs1, rs2 },
                    0b111 => RiscvInst::Remu { rd, rs1, rs2 },
                    // full case
                    _ => unreachable!(),
                },
                0b0000000 => match function {
                    0b000 => RiscvInst::Add { rd, rs1, rs2 },
                    0b001 => RiscvInst::Sll { rd, rs1, rs2 },
                    0b010 => RiscvInst::Slt { rd, rs1, rs2 },
                    0b011 => RiscvInst::Sltu { rd, rs1, rs2 },
                    0b100 => RiscvInst::Xor { rd, rs1, rs2 },
                    0b101 => RiscvInst::Srl { rd, rs1, rs2 },
                    0b110 => RiscvInst::Or { rd, rs1, rs2 },
                    0b111 => RiscvInst::And { rd, rs1, rs2 },
                    // full case
                    _ => unreachable!(),
                },
                0b0100000 => match function {
                    0b000 => RiscvInst::Sub { rd, rs1, rs2 },
                    0b101 => RiscvInst::Sra { rd, rs1, rs2 },
                    _ => RiscvInst::Illegal,
                },
                _ => RiscvInst::Illegal,
            }
        }

        /* LUI */
        0b0110111 => RiscvInst::Lui {
            rd,
            imm: u_imm(bits),
        },

        /* OP-32 */
        0b0111011 => {
            match funct7(bits) {
                // M-extension
                0b0000001 => match function {
                    0b000 => RiscvInst::Mulw { rd, rs1, rs2 },
                    0b100 => RiscvInst::Divw { rd, rs1, rs2 },
                    0b101 => RiscvInst::Divuw { rd, rs1, rs2 },
                    0b110 => RiscvInst::Remw { rd, rs1, rs2 },
                    0b111 => RiscvInst::Remuw { rd, rs1, rs2 },
                    _ => RiscvInst::Illegal,
                },
                0b0000000 => match function {
                    0b000 => RiscvInst::Addw { rd, rs1, rs2 },
                    0b001 => RiscvInst::Sllw { rd, rs1, rs2 },
                    0b101 => RiscvInst::Srlw { rd, rs1, rs2 },
                    _ => RiscvInst::Illegal,
                },
                0b0100000 => match function {
                    0b000 => RiscvInst::Subw { rd, rs1, rs2 },
                    0b101 => RiscvInst::Sraw { rd, rs1, rs2 },
                    _ => RiscvInst::Illegal,
                },
                _ => RiscvInst::Illegal,
            }
        }

        /* MADD */
        0b1000011 => match funct7(bits) & 3 {
            0b00 => RiscvInst::FmaddS {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            0b01 => RiscvInst::FmaddD {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            _ => RiscvInst::Illegal,
        },

        /* MSUB */
        0b1000111 => match funct7(bits) & 3 {
            0b00 => RiscvInst::FmsubS {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            0b01 => RiscvInst::FmsubD {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            _ => RiscvInst::Illegal,
        },

        /* NMSUB */
        0b1001011 => match funct7(bits) & 3 {
            0b00 => RiscvInst::FnmsubS {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            0b01 => RiscvInst::FnmsubD {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            _ => RiscvInst::Illegal,
        },

        /* NMADD */
        0b1001111 => match funct7(bits) & 3 {
            0b00 => RiscvInst::FnmaddS {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            0b01 => RiscvInst::FnmaddD {
                frd: rd,
                frs1: rs1,
                frs2: rs2,
                frs3: rs3(bits),
                rm: rm!(function),
            },
            _ => RiscvInst::Illegal,
        },

        /* AUIPC */
        0b0010111 => RiscvInst::Auipc {
            rd,
            imm: u_imm(bits),
        },

        /* OP-FP */
        0b1010011 => {
            let function7 = funct7(bits);
            match function7 {
                /* F-extension and D-extension */
                0b0000000 => RiscvInst::FaddS {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0000001 => RiscvInst::FaddD {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0000100 => RiscvInst::FsubS {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0000101 => RiscvInst::FsubD {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0001000 => RiscvInst::FmulS {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0001001 => RiscvInst::FmulD {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0001100 => RiscvInst::FdivS {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0001101 => RiscvInst::FdivD {
                    frd: rd,
                    frs1: rs1,
                    frs2: rs2,
                    rm: rm!(function),
                },
                0b0101100 => match rs2 {
                    0b00000 => RiscvInst::FsqrtS {
                        frd: rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0101101 => match rs2 {
                    0b00000 => RiscvInst::FsqrtD {
                        frd: rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0010000 => match function {
                    0b000 => RiscvInst::FsgnjS {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b001 => RiscvInst::FsgnjnS {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b010 => RiscvInst::FsgnjxS {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0010001 => match function {
                    0b000 => RiscvInst::FsgnjD {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b001 => RiscvInst::FsgnjnD {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b010 => RiscvInst::FsgnjxD {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0010100 => match function {
                    0b000 => RiscvInst::FminS {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b001 => RiscvInst::FmaxS {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0010101 => match function {
                    0b000 => RiscvInst::FminD {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b001 => RiscvInst::FmaxD {
                        frd: rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0100000 => match rs2 {
                    0b00001 => RiscvInst::FcvtSD {
                        frd: rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b0100001 => match rs2 {
                    0b00000 => RiscvInst::FcvtDS {
                        frd: rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1100000 => match rs2 {
                    0b00000 => RiscvInst::FcvtWS {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    0b00001 => RiscvInst::FcvtWuS {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    0b00010 => RiscvInst::FcvtLS {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    0b00011 => RiscvInst::FcvtLuS {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1100001 => match rs2 {
                    0b00000 => RiscvInst::FcvtWD {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    0b00001 => RiscvInst::FcvtWuD {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    0b00010 => RiscvInst::FcvtLD {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    0b00011 => RiscvInst::FcvtLuD {
                        rd,
                        frs1: rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1110000 => match (rs2, function) {
                    (0b00000, 0b000) => RiscvInst::FmvXW { rd, frs1: rs1 },
                    (0b00000, 0b001) => RiscvInst::FclassS { rd, frs1: rs1 },
                    _ => RiscvInst::Illegal,
                },
                0b1110001 => match (rs2, function) {
                    (0b00000, 0b000) => RiscvInst::FmvXD { rd, frs1: rs1 },
                    (0b00000, 0b001) => RiscvInst::FclassD { rd, frs1: rs1 },
                    _ => RiscvInst::Illegal,
                },
                0b1010000 => match function {
                    0b000 => RiscvInst::FleS {
                        rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b001 => RiscvInst::FltS {
                        rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b010 => RiscvInst::FeqS {
                        rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1010001 => match function {
                    0b000 => RiscvInst::FleD {
                        rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b001 => RiscvInst::FltD {
                        rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    0b010 => RiscvInst::FeqD {
                        rd,
                        frs1: rs1,
                        frs2: rs2,
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1101000 => match rs2 {
                    0b00000 => RiscvInst::FcvtSW {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    0b00001 => RiscvInst::FcvtSWu {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    0b00010 => RiscvInst::FcvtSL {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    0b00011 => RiscvInst::FcvtSLu {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1101001 => match rs2 {
                    0b00000 => RiscvInst::FcvtDW {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    0b00001 => RiscvInst::FcvtDWu {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    0b00010 => RiscvInst::FcvtDL {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    0b00011 => RiscvInst::FcvtDLu {
                        frd: rd,
                        rs1,
                        rm: rm!(function),
                    },
                    _ => RiscvInst::Illegal,
                },
                0b1111000 => match (rs2, function) {
                    (0b00000, 0b000) => RiscvInst::FmvWX { frd: rd, rs1 },
                    _ => RiscvInst::Illegal,
                },
                0b1111001 => match (rs2, function) {
                    (0b00000, 0b000) => RiscvInst::FmvDX { frd: rd, rs1 },
                    _ => RiscvInst::Illegal,
                },
                _ => RiscvInst::Illegal,
            }
        }

        /* BRANCH */
        0b1100011 => {
            let imm = b_imm(bits);
            match function {
                0b000 => RiscvInst::Beq { rs1, rs2, imm },
                0b001 => RiscvInst::Bne { rs1, rs2, imm },
                0b100 => RiscvInst::Blt { rs1, rs2, imm },
                0b101 => RiscvInst::Bge { rs1, rs2, imm },
                0b110 => RiscvInst::Bltu { rs1, rs2, imm },
                0b111 => RiscvInst::Bgeu { rs1, rs2, imm },
                _ => RiscvInst::Illegal,
            }
        }

        /* JALR */
        0b1100111 => RiscvInst::Jalr {
            rd,
            rs1,
            imm: i_imm(bits),
        },

        /* JAL */
        0b1101111 => RiscvInst::Jal {
            rd,
            imm: j_imm(bits),
        },

        /* SYSTEM */
        0b1110011 => {
            match function {
                0b000 => match bits {
                    0x73 => RiscvInst::Ecall,
                    0x100073 => RiscvInst::Ebreak,
                    0x30200073 => RiscvInst::Mret,
                    0x10200073 => RiscvInst::Sret,
                    0x10500073 => RiscvInst::Wfi,
                    bits if rd == 0 && funct7(bits) == 0b0001001 => {
                        RiscvInst::SfenceVma { rs1, rs2 }
                    }
                    _ => RiscvInst::Illegal,
                },
                0b100 => RiscvInst::Illegal,
                _ => {
                    // Otherwise this is CSR instruction
                    let csr = super::csr::Csr(csr(bits));
                    // For CSRRS, CSRRC, CSRRSI, CSRRCI, rs1 = 0 means readonly.
                    // If the CSR is readonly while we try to write it, it is an exception.
                    let readonly = function & 0b010 != 0 && rs1 == 0;
                    if csr.readonly() && !readonly {
                        return RiscvInst::Illegal;
                    }
                    match function {
                        0b001 => RiscvInst::Csrrw { rd, rs1, csr },
                        0b010 => RiscvInst::Csrrs { rd, rs1, csr },
                        0b011 => RiscvInst::Csrrc { rd, rs1, csr },
                        0b101 => RiscvInst::Csrrwi { rd, imm: rs1, csr },
                        0b110 => RiscvInst::Csrrsi { rd, imm: rs1, csr },
                        0b111 => RiscvInst::Csrrci { rd, imm: rs1, csr },
                        _ => unreachable!(),
                    }
                }
            }
        }
        _ => RiscvInst::Illegal,
    }
}
