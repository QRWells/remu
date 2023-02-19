use crate::{bus::Bus, cpu::Cpu};

use super::{
    bus::RiscvBus,
    csr::Csrs,
    decode::{decode, decode_compressed},
    exception::Exception,
    instruction::{RiscvInst, RiscvInstWrapper},
    mmu::MMU,
};

const MACHINE_MODE: u8 = 0;
const SUPERVISOR_MODE: u8 = 1;
const USER_MODE: u8 = 2;

pub struct RV64Cpu {
    pub(crate) clock: u64,
    pub(crate) pc: u64,
    pub(crate) x: [u64; 32],
    pub(crate) f: [f64; 32],
    pub(crate) bus: RiscvBus,
    pub(crate) mmu: MMU,
    pub(crate) csr: Csrs,
    pub(crate) mode: u8,
}

impl RV64Cpu {
    fn new() -> Self {
        Self {
            clock: 0,
            pc: 0,
            x: [0; 32],
            f: [0.0; 32],
            bus: RiscvBus::new(),
            mmu: MMU::new(),
            csr: Csrs::new(),
            mode: MACHINE_MODE,
        }
    }

    pub fn fetch(&mut self) -> Result<RiscvInstWrapper, Exception> {
        let addr = self.mmu.translate(self.pc).expect("Translation failed");
        match self.bus.load(addr, 1) {
            Ok(val) => match val & 0x3 {
                0x3 => {
                    let inst = u32::from_le(self.bus.load(addr, 4).unwrap() as u32);
                    Ok(RiscvInstWrapper::Full(decode(inst)))
                }
                _ => {
                    let inst = u16::from_le(self.bus.load(addr, 2).unwrap() as u16);
                    Ok(RiscvInstWrapper::Compact(decode_compressed(inst)))
                }
            },
            Err(e) => Err(e),
        }
    }
}

impl Cpu for RV64Cpu {
    fn init(&mut self) {
        self.bus.init();
    }

    fn load(&mut self, data: Vec<u8>) {
        self.bus.load_data(0x8000_0000, &data).expect("Load failed");
    }

    fn reset(&mut self) {
        self.pc = 0;
        self.x = [0; 32];
    }

    fn execute(&mut self) {
        loop {
            let inst = self.fetch();
            let inst_with_len = match inst {
                Ok(inst) => inst,
                Err(e) => {
                    println!("Exception: {:?}", e);
                    break;
                }
            };
            let inst = match inst_with_len {
                RiscvInstWrapper::Full(inst) => inst,
                RiscvInstWrapper::Compact(inst) => inst,
            };
            debug!("pc:{:#x}\t{}", self.pc, inst);
            match inst {
                RiscvInst::Illegal => break,
                RiscvInst::Lb { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] =
                        self.bus.load_byte(addr).expect("Load failed") as i8 as u64;
                }
                RiscvInst::Lh { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] =
                        self.bus.load_half(addr).expect("Load failed") as i16 as u64;
                }
                RiscvInst::Lw { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] =
                        self.bus.load_word(addr).expect("Load failed") as i32 as u64;
                }
                RiscvInst::Ld { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] = self.bus.load(addr, 8).expect("Load failed");
                }
                RiscvInst::Lbu { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] = self.bus.load_byte(addr).expect("Load failed") as u64;
                }
                RiscvInst::Lhu { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] = self.bus.load_half(addr).expect("Load failed") as u64;
                }
                RiscvInst::Lwu { rd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("Translation failed");
                    self.x[rd as usize] = self.bus.load_word(addr).expect("Load failed") as u64;
                }
                RiscvInst::Fence => {}
                RiscvInst::FenceI => {}
                RiscvInst::Addi { rd, rs1, imm } => {
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm as u64);
                }
                RiscvInst::Slli { rd, rs1, imm } => {
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_shl(imm as u32);
                }
                RiscvInst::Slti { rd, rs1, imm } => {
                    self.x[rd as usize] = if (self.x[rs1 as usize] as i64) < (imm as i64) {
                        1
                    } else {
                        0
                    };
                }
                RiscvInst::Sltiu { rd, rs1, imm } => {
                    self.x[rd as usize] = if self.x[rs1 as usize] < (imm as u64) {
                        1
                    } else {
                        0
                    };
                }
                RiscvInst::Xori { rd, rs1, imm } => {
                    self.x[rd as usize] = self.x[rs1 as usize] ^ (imm as u64);
                }
                RiscvInst::Srli { rd, rs1, imm } => {
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_shr(imm as u32);
                }
                RiscvInst::Srai { rd, rs1, imm } => {
                    self.x[rd as usize] =
                        (self.x[rs1 as usize] as i64).wrapping_shr(imm as u32) as u64;
                }
                RiscvInst::Ori { rd, rs1, imm } => {
                    self.x[rd as usize] = self.x[rs1 as usize] | (imm as u64);
                }
                RiscvInst::Andi { rd, rs1, imm } => {
                    self.x[rd as usize] = self.x[rs1 as usize] & (imm as u64);
                }
                RiscvInst::Auipc { rd, imm } => {
                    self.x[rd as usize] = self.pc.wrapping_add((imm as u64) << 12);
                }
                RiscvInst::Lui { rd, imm } => {
                    self.x[rd as usize] = (imm as u64) << 12;
                }
                RiscvInst::Addiw { rd, rs1, imm } => {
                    self.x[rd as usize] =
                        (self.x[rs1 as usize].wrapping_add(imm as u64)) as u32 as u64;
                }
                RiscvInst::Slliw { rd, rs1, imm } => {
                    let shamt = (imm & 0x1f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shl(shamt) as u64;
                }
                RiscvInst::Srliw { rd, rs1, imm } => {
                    let shamt = (imm & 0x1f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shr(shamt) as u64;
                }
                RiscvInst::Sraiw { rd, rs1, imm } => {
                    let shamt = (imm & 0x1f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as i32).wrapping_shr(shamt) as u64;
                }
                RiscvInst::Addw { rd, rs1, rs2 } => {
                    self.x[rd as usize] =
                        (self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize])) as u32 as u64;
                }
                RiscvInst::Subw { rd, rs1, rs2 } => {
                    self.x[rd as usize] =
                        (self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize])) as u32 as u64;
                }
                RiscvInst::Sllw { rd, rs1, rs2 } => {
                    let shamt = (self.x[rs2 as usize] & 0x1f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shl(shamt) as u64;
                }
                RiscvInst::Srlw { rd, rs1, rs2 } => {
                    let shamt = (self.x[rs2 as usize] & 0x1f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shr(shamt) as u64;
                }
                RiscvInst::Sraw { rd, rs1, rs2 } => {
                    let shamt = (self.x[rs2 as usize] & 0x1f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as i32).wrapping_shr(shamt) as u64;
                }
                RiscvInst::Sb { rs1, rs2, imm } => {
                    let addr = self
                        .mmu
                        .translate(self.x[rs1 as usize].wrapping_add(imm as u64))
                        .expect("memory access violation");
                    self.bus
                        .store_byte(addr, self.x[rs2 as usize] as u8)
                        .expect("memory access violation");
                }
                RiscvInst::Sh { rs1, rs2, imm } => {
                    let addr = self
                        .mmu
                        .translate(self.x[rs1 as usize].wrapping_add(imm as u64))
                        .expect("memory access violation");
                    let bytes = self.x[rs2 as usize].to_le_bytes();
                    self.bus
                        .store_half(addr, [bytes[0], bytes[1]])
                        .expect("memory access violation");
                }
                RiscvInst::Sw { rs1, rs2, imm } => {
                    let addr = self
                        .mmu
                        .translate(self.x[rs1 as usize].wrapping_add(imm as u64))
                        .expect("memory access violation");
                    let bytes = self.x[rs2 as usize].to_le_bytes();
                    self.bus
                        .store_word(addr, [bytes[0], bytes[1], bytes[2], bytes[3]])
                        .expect("memory access violation");
                }
                RiscvInst::Sd { rs1, rs2, imm } => {
                    let addr = self
                        .mmu
                        .translate(self.x[rs1 as usize].wrapping_add(imm as u64))
                        .expect("memory access violation");
                    let bytes = self.x[rs2 as usize].to_le_bytes();
                    self.bus
                        .store_double(
                            addr,
                            [
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                bytes[6], bytes[7],
                            ],
                        )
                        .expect("memory access violation");
                }
                RiscvInst::Add { rd, rs1, rs2 } => {
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]);
                }
                RiscvInst::Sub { rd, rs1, rs2 } => {
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]);
                }
                RiscvInst::Sll { rd, rs1, rs2 } => {
                    let shamt = (self.x[rs2 as usize] & 0x3f) as u32;
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_shl(shamt);
                }
                RiscvInst::Slt { rd, rs1, rs2 } => {
                    self.x[rd as usize] =
                        if (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64) {
                            1
                        } else {
                            0
                        };
                }
                RiscvInst::Sltu { rd, rs1, rs2 } => {
                    self.x[rd as usize] = if self.x[rs1 as usize] < self.x[rs2 as usize] {
                        1
                    } else {
                        0
                    };
                }
                RiscvInst::Xor { rd, rs1, rs2 } => {
                    self.x[rd as usize] = self.x[rs1 as usize] ^ self.x[rs2 as usize];
                }
                RiscvInst::Srl { rd, rs1, rs2 } => {
                    let shamt = (self.x[rs2 as usize] & 0x3f) as u32;
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_shr(shamt);
                }
                RiscvInst::Sra { rd, rs1, rs2 } => {
                    let shamt = (self.x[rs2 as usize] & 0x3f) as u32;
                    self.x[rd as usize] = (self.x[rs1 as usize] as i64).wrapping_shr(shamt) as u64;
                }
                RiscvInst::Or { rd, rs1, rs2 } => {
                    self.x[rd as usize] = self.x[rs1 as usize] | self.x[rs2 as usize];
                }
                RiscvInst::And { rd, rs1, rs2 } => {
                    self.x[rd as usize] = self.x[rs1 as usize] & self.x[rs2 as usize];
                }
                RiscvInst::Beq { rs1, rs2, imm } => {
                    if self.x[rs1 as usize] == self.x[rs2 as usize] {
                        self.pc = self.pc.wrapping_add(imm as u64);
                        continue;
                    }
                }
                RiscvInst::Bne { rs1, rs2, imm } => {
                    if self.x[rs1 as usize] != self.x[rs2 as usize] {
                        self.pc = self.pc.wrapping_add(imm as u64);
                        continue;
                    }
                }
                RiscvInst::Blt { rs1, rs2, imm } => {
                    if (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64) {
                        self.pc = self.pc.wrapping_add(imm as u64);
                        continue;
                    }
                }
                RiscvInst::Bge { rs1, rs2, imm } => {
                    if (self.x[rs1 as usize] as i64) >= (self.x[rs2 as usize] as i64) {
                        self.pc = self.pc.wrapping_add(imm as u64);
                        continue;
                    }
                }
                RiscvInst::Bltu { rs1, rs2, imm } => {
                    if self.x[rs1 as usize] < self.x[rs2 as usize] {
                        self.pc = self.pc.wrapping_add(imm as u64);
                        continue;
                    }
                }
                RiscvInst::Bgeu { rs1, rs2, imm } => {
                    if self.x[rs1 as usize] >= self.x[rs2 as usize] {
                        self.pc = self.pc.wrapping_add(imm as u64);
                        continue;
                    }
                }
                RiscvInst::Jalr { rd, rs1, imm } => {
                    let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
                    self.x[rd as usize] = self.pc + 4;
                    self.pc = addr & !1;
                    continue;
                }
                RiscvInst::Jal { rd, imm } => {
                    self.x[rd as usize] = self.pc + 4;
                    self.pc = self.pc.wrapping_add(imm as u64);
                    continue;
                }
                RiscvInst::Ecall => {
                    todo!("ecall")
                }
                RiscvInst::Ebreak => {
                    todo!("ebreak")
                }

                RiscvInst::Csrrw { rd, rs1, csr } => {
                    let t = self.csr.load(csr.into());
                    self.csr.store(csr.into(), self.x[rs1 as usize]);
                    self.x[rd as usize] = t.into();
                }
                RiscvInst::Csrrs { rd, rs1, csr } => {
                    let t = self.csr.load(csr.into());
                    self.csr
                        .store(csr.into(), (t | self.x[rs1 as usize]).into());
                    self.x[rd as usize] = t.into();
                }
                RiscvInst::Csrrc { rd, rs1, csr } => {
                    let t = self.csr.load(csr.into());
                    self.csr
                        .store(csr.into(), (t & !self.x[rs1 as usize]).into());
                    self.x[rd as usize] = t.into();
                }
                RiscvInst::Csrrwi { rd, imm, csr } => {
                    let t = self.csr.load(csr.into());
                    self.csr.store(csr.into(), imm as u64);
                    self.x[rd as usize] = t.into();
                }
                RiscvInst::Csrrsi { rd, imm, csr } => {
                    let t = self.csr.load(csr.into());
                    self.csr.store(csr.into(), (t | (imm as u64)).into());
                    self.x[rd as usize] = t.into();
                }
                RiscvInst::Csrrci { rd, imm, csr } => {
                    let t = self.csr.load(csr.into());
                    self.csr.store(csr.into(), (t & !(imm as u64)).into());
                    self.x[rd as usize] = t.into();
                }

                RiscvInst::Mul { rd, rs1, rs2 } => {
                    self.x[rd as usize] = self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]);
                }
                RiscvInst::Mulh { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i64;
                    let b = self.x[rs2 as usize] as i64;
                    self.x[rd as usize] = (a.wrapping_mul(b) >> 32) as u64;
                }
                RiscvInst::Mulhsu { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i64;
                    let b = self.x[rs2 as usize];
                    self.x[rd as usize] = (a.wrapping_mul(b as i64) >> 32) as u64;
                }
                RiscvInst::Mulhu { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize];
                    let b = self.x[rs2 as usize];
                    self.x[rd as usize] = (a.wrapping_mul(b) >> 32) as u64;
                }
                RiscvInst::Div { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i64;
                    let b = self.x[rs2 as usize] as i64;
                    self.x[rd as usize] = if b == 0 {
                        u64::MAX
                    } else {
                        a.wrapping_div(b) as u64
                    };
                }
                RiscvInst::Divu { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize];
                    let b = self.x[rs2 as usize];
                    self.x[rd as usize] = if b == 0 { u64::MAX } else { a.wrapping_div(b) };
                }
                RiscvInst::Rem { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i64;
                    let b = self.x[rs2 as usize] as i64;
                    self.x[rd as usize] = if b == 0 {
                        a as u64
                    } else {
                        a.wrapping_rem(b) as u64
                    };
                }
                RiscvInst::Remu { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize];
                    let b = self.x[rs2 as usize];
                    self.x[rd as usize] = if b == 0 { a } else { a.wrapping_rem(b) };
                }
                RiscvInst::Mulw { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i32;
                    let b = self.x[rs2 as usize] as i32;
                    self.x[rd as usize] = (a.wrapping_mul(b)) as u64;
                }
                RiscvInst::Divw { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i32;
                    let b = self.x[rs2 as usize] as i32;
                    self.x[rd as usize] = if b == 0 {
                        u64::MAX
                    } else {
                        (a.wrapping_div(b)) as u64
                    };
                }
                RiscvInst::Divuw { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as u32;
                    let b = self.x[rs2 as usize] as u32;
                    self.x[rd as usize] = if b == 0 {
                        u64::MAX
                    } else {
                        (a.wrapping_div(b)) as u64
                    };
                }
                RiscvInst::Remw { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as i32;
                    let b = self.x[rs2 as usize] as i32;
                    self.x[rd as usize] = if b == 0 {
                        a as u64
                    } else {
                        (a.wrapping_rem(b)) as u64
                    };
                }
                RiscvInst::Remuw { rd, rs1, rs2 } => {
                    let a = self.x[rs1 as usize] as u32;
                    let b = self.x[rs2 as usize] as u32;
                    self.x[rd as usize] = if b == 0 {
                        a as u64
                    } else {
                        (a.wrapping_rem(b)) as u64
                    };
                }
                RiscvInst::LrW { rd, rs1, aqrl } | RiscvInst::LrD { rd, rs1, aqrl } => {
                    todo!("atomic")
                }
                RiscvInst::ScW { rd, rs1, rs2, aqrl } | RiscvInst::ScD { rd, rs1, rs2, aqrl } => {
                    todo!("atomic")
                }
                RiscvInst::AmoswapW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoswapD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoaddW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoaddD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoxorW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoxorD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoandW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoandD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoorW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmoorD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmominW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmominD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmomaxW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmomaxD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmominuW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmominuD { rd, rs1, rs2, aqrl }
                | RiscvInst::AmomaxuW { rd, rs1, rs2, aqrl }
                | RiscvInst::AmomaxuD { rd, rs1, rs2, aqrl } => todo!("atomic"),

                RiscvInst::Flw { frd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(self.x[rs1 as usize].wrapping_add(imm as u64))
                        .expect("memory access invalid");
                    let val = self.bus.load_word(addr).expect("memory access invalid");
                    self.f[frd as usize] = f32::from_bits(val) as f64;
                }
                RiscvInst::Fsw { rs1, frs2, imm } => {
                    let addr = self
                        .mmu
                        .translate(self.x[rs1 as usize].wrapping_add(imm as u64))
                        .expect("memory access invalid");
                    let val = self.f[frs2 as usize] as f32;
                    self.bus
                        .store_word(addr, val.to_bits().to_le_bytes())
                        .expect("memory access invalid");
                }
                RiscvInst::FaddS {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.f[frd as usize] = (a + b) as f64;
                }
                RiscvInst::FsubS {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.f[frd as usize] = (a - b) as f64;
                }
                RiscvInst::FmulS {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.f[frd as usize] = (a * b) as f64;
                }
                RiscvInst::FdivS {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.f[frd as usize] = (a / b) as f64;
                    todo!("rm")
                }
                RiscvInst::FsqrtS { frd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.f[frd as usize] = a.sqrt() as f64;
                }
                RiscvInst::FsgnjS { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let sign = b.to_bits() & 0x8000_0000;
                    self.f[frd as usize] = f32::from_bits(a.to_bits() & !0x8000_0000 | sign) as f64;
                }
                RiscvInst::FsgnjnS { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let sign = b.to_bits() & 0x8000_0000;
                    self.f[frd as usize] = f32::from_bits(a.to_bits() | sign) as f64;
                }
                RiscvInst::FsgnjxS { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let sign = b.to_bits() & 0x8000_0000;
                    self.f[frd as usize] = f32::from_bits(a.to_bits() ^ sign) as f64;
                }
                RiscvInst::FminS { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.f[frd as usize] = a.min(b) as f64;
                }
                RiscvInst::FmaxS { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.f[frd as usize] = a.max(b) as f64;
                }
                RiscvInst::FcvtWS { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as i32 as u64;
                }
                RiscvInst::FcvtWuS { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as u64;
                }
                RiscvInst::FcvtLS { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as i64 as u64;
                }
                RiscvInst::FcvtLuS { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as u64;
                }
                RiscvInst::FmvXW { rd, frs1 } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as u64;
                }
                RiscvInst::FclassS { rd, frs1 } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = float_classify(a) as u64;
                }
                RiscvInst::FeqS { rd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.x[rd as usize] = (a == b) as u64;
                }
                RiscvInst::FltS { rd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.x[rd as usize] = (a < b) as u64;
                }
                RiscvInst::FleS { rd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    self.x[rd as usize] = (a <= b) as u64;
                }
                RiscvInst::FcvtSW { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as i32 as u32;
                    self.f[frd as usize] = a as f32 as f64;
                }
                RiscvInst::FcvtSWu { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as u32;
                    self.f[frd as usize] = a as f32 as f64;
                }
                RiscvInst::FcvtSL { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as i64 as u64;
                    self.f[frd as usize] = a as f32 as f64;
                }
                RiscvInst::FcvtSLu { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as u64;
                    self.f[frd as usize] = a as f32 as f64;
                }
                RiscvInst::FmvWX { frd, rs1 } => {
                    let a = self.x[rs1 as usize] as u32;
                    self.f[frd as usize] = a as f32 as f64;
                }
                RiscvInst::FmaddS {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let c = self.f[frs3 as usize] as f32;
                    self.f[frd as usize] = (a * b + c) as f64;
                }
                RiscvInst::FmsubS {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let c = self.f[frs3 as usize] as f32;
                    self.f[frd as usize] = (a * b - c) as f64;
                }
                RiscvInst::FnmsubS {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let c = self.f[frs3 as usize] as f32;
                    self.f[frd as usize] = (-a * b - c) as f64;
                }
                RiscvInst::FnmaddS {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize] as f32;
                    let b = self.f[frs2 as usize] as f32;
                    let c = self.f[frs3 as usize] as f32;
                    self.f[frd as usize] = (-a * b + c) as f64;
                }
                RiscvInst::Fld { frd, rs1, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("memory fault");
                    self.f[frd as usize] =
                        f64::from_bits(self.bus.load_double(addr).expect("memory fault"));
                }
                RiscvInst::Fsd { rs1, frs2, imm } => {
                    let addr = self
                        .mmu
                        .translate(addr_add(self.x[rs1 as usize], imm))
                        .expect("memory fault");
                    self.bus
                        .store_double(addr, self.f[frs2 as usize].to_bits().to_le_bytes())
                        .expect("memory fault");
                }
                RiscvInst::FaddD {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.f[frd as usize] = a + b;
                }
                RiscvInst::FsubD {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.f[frd as usize] = a - b;
                }
                RiscvInst::FmulD {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.f[frd as usize] = a * b;
                }
                RiscvInst::FdivD {
                    frd,
                    frs1,
                    frs2,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.f[frd as usize] = a / b;
                }
                RiscvInst::FsqrtD { frd, frs1, rm } => {
                    let a = self.f[frs1 as usize];
                    self.f[frd as usize] = a.sqrt();
                }
                RiscvInst::FsgnjD { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let sign = f64::to_bits(b) & (1 << 63);
                    self.f[frd as usize] = f64::from_bits(f64::to_bits(a) & !(1 << 63) | sign);
                }
                RiscvInst::FsgnjnD { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let sign = f64::to_bits(b) & (1 << 63);
                    self.f[frd as usize] = f64::from_bits(f64::to_bits(a) & !(1 << 63) | !sign);
                }
                RiscvInst::FsgnjxD { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let sign = f64::to_bits(b) & (1 << 63);
                    self.f[frd as usize] = f64::from_bits(f64::to_bits(a) ^ sign);
                }
                RiscvInst::FminD { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.f[frd as usize] = a.min(b);
                }
                RiscvInst::FmaxD { frd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.f[frd as usize] = a.max(b);
                }
                RiscvInst::FcvtSD { frd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.f[frd as usize] = a as f64;
                }
                RiscvInst::FcvtDS { frd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f64;
                    self.f[frd as usize] = a as f32 as f64;
                }
                RiscvInst::FcvtWD { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as i32 as u64;
                }
                RiscvInst::FcvtWuD { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as u32 as u64;
                }
                RiscvInst::FcvtLD { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as i64 as u64;
                }
                RiscvInst::FcvtLuD { rd, frs1, rm } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as u64;
                }
                RiscvInst::FmvXD { rd, frs1 } => {
                    let a = self.f[frs1 as usize] as f32;
                    self.x[rd as usize] = a as u64;
                }
                RiscvInst::FclassD { rd, frs1 } => {
                    let a = self.f[frs1 as usize];
                    self.x[rd as usize] = double_classify(a) as u64;
                }
                RiscvInst::FeqD { rd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.x[rd as usize] = if a == b { 1 } else { 0 };
                }
                RiscvInst::FltD { rd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.x[rd as usize] = if a < b { 1 } else { 0 };
                }
                RiscvInst::FleD { rd, frs1, frs2 } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    self.x[rd as usize] = if a <= b { 1 } else { 0 };
                }
                RiscvInst::FcvtDW { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as i32 as i64;
                    self.f[frd as usize] = a as f64;
                }
                RiscvInst::FcvtDWu { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as u32 as i64;
                    self.f[frd as usize] = a as f64;
                }
                RiscvInst::FcvtDL { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as i64;
                    self.f[frd as usize] = a as f64;
                }
                RiscvInst::FcvtDLu { frd, rs1, rm } => {
                    let a = self.x[rs1 as usize] as u64;
                    self.f[frd as usize] = a as f64;
                }
                RiscvInst::FmvDX { frd, rs1 } => {
                    let a = self.x[rs1 as usize] as u64;
                    self.f[frd as usize] = a as f64;
                }
                RiscvInst::FmaddD {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let c = self.f[frs3 as usize];
                    self.f[frd as usize] = a * b + c;
                }
                RiscvInst::FmsubD {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let c = self.f[frs3 as usize];
                    self.f[frd as usize] = a * b - c;
                }
                RiscvInst::FnmsubD {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let c = self.f[frs3 as usize];
                    self.f[frd as usize] = -(a * b - c);
                }
                RiscvInst::FnmaddD {
                    frd,
                    frs1,
                    frs2,
                    frs3,
                    rm,
                } => {
                    let a = self.f[frs1 as usize];
                    let b = self.f[frs2 as usize];
                    let c = self.f[frs3 as usize];
                    self.f[frd as usize] = -(a * b + c);
                }
                RiscvInst::Mret => todo!(),
                RiscvInst::Sret => todo!(),
                RiscvInst::Wfi => todo!(),
                RiscvInst::SfenceVma { rs1, rs2 } => todo!(),
            }
            self.pc += match inst_with_len {
                RiscvInstWrapper::Full(_) => 4,
                RiscvInstWrapper::Compact(_) => 2,
            }
        }
        self.x[0] = 0; // x0 is always 0
    }
}

fn float_classify(x: f32) -> u64 {
    let mut res = 0;
    if x == f32::NEG_INFINITY {
        res |= 1;
    }
    if x.is_normal() && x.is_sign_negative() {
        res |= 2;
    }
    if x.is_subnormal() && x.is_sign_negative() {
        res |= 4;
    }
    if x == 0.0 && x.is_sign_negative() {
        res |= 8;
    }
    if x == 0.0 && x.is_sign_positive() {
        res |= 16;
    }
    if x.is_subnormal() && x.is_sign_positive() {
        res |= 32;
    }
    if x.is_normal() && x.is_sign_positive() {
        res |= 64;
    }
    if x == f32::INFINITY {
        res |= 128;
    }
    if x.is_nan() && !quiet_nan(x) {
        res |= 256;
    }
    if x.is_nan() && quiet_nan(x) {
        res |= 512;
    }
    res
}

fn quiet_nan(value: f32) -> bool {
    let bits = value.to_bits();
    (bits & 0x7f800000) == 0x7f800000 && (bits & 0x007fffff) != 0
}

fn double_classify(x: f64) -> u64 {
    let mut res = 0;
    if x == f64::NEG_INFINITY {
        res |= 1;
    }
    if x.is_normal() && x.is_sign_negative() {
        res |= 2;
    }
    if x.is_subnormal() && x.is_sign_negative() {
        res |= 4;
    }
    if x == 0.0 && x.is_sign_negative() {
        res |= 8;
    }
    if x == 0.0 && x.is_sign_positive() {
        res |= 16;
    }
    if x.is_subnormal() && x.is_sign_positive() {
        res |= 32;
    }
    if x.is_normal() && x.is_sign_positive() {
        res |= 64;
    }
    if x == f64::INFINITY {
        res |= 128;
    }
    if x.is_nan() && !quiet_nan_double(x) {
        res |= 256;
    }
    if x.is_nan() && quiet_nan_double(x) {
        res |= 512;
    }
    res
}

fn quiet_nan_double(value: f64) -> bool {
    let bits = value.to_bits();
    (bits & 0x7ff0000000000000) == 0x7ff0000000000000 && (bits & 0x000fffffffffffff) != 0
}

fn addr_add(addr: u64, offset: i32) -> u64 {
    if offset.is_negative() {
        addr - offset.wrapping_abs() as u32 as u64
    } else {
        addr + offset as u64
    }
}

#[cfg(test)]
mod test {
    use crate::{
        arch::riscv::{
            cpu::addr_add,
            reg::{A0, RA, SP},
        },
        cpu::Cpu,
    };

    use super::RV64Cpu;

    #[test]
    fn test_exec() {
        assert_eq!(addr_add(64u64, -32i32), 32);
        // factorial
        let data: Vec<u32> = vec![
            // -O2
            // 0x00050793, 0x00100513, 0x00100693, 0x00f55a63, 0x00078713, 0xfff7879b, 0x02a7053b,
            // 0xfed79ae3, 0x0000001f,
            // -O0
            0xfe010113, 0x00113c23, 0x00813823, 0x02010413, 0x00050793, 0xfef42623, 0xfec42783,
            0x0007871b, 0x00100793, 0x00e7c663, 0x00100793, 0x0300006f, 0xfec42783, 0xfff7879b,
            0x0007879b, 0x00078513, 0x00000097, 0xfc0080e7, 0x00050793, 0x00078713, 0xfec42783,
            0x02e787bb, 0x0007879b, 0x00078513, 0x01813083, 0x01013403, 0x02010113, 0x00008067,
            0x0000001f,
        ];
        let data: Vec<u8> = data.iter().flat_map(|x| x.to_le_bytes().to_vec()).collect();
        let mut cpu = RV64Cpu::new();
        cpu.init();

        cpu.pc = 0x8000_0000;
        cpu.x[RA] = 0x8000_0000 + 0x70;
        cpu.x[SP] = 0x8000_0000 + 0x400;
        cpu.x[A0] = 5;

        cpu.load(data);
        cpu.execute();

        assert_eq!(cpu.x[10], 120u64);
    }
}
