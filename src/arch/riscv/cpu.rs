use crate::{
    arch::riscv::csr::*,
    bus::Bus,
    cpu::Cpu,
    util::{addr_add, double_classify, float_classify},
};

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

    pub fn execute(&mut self, inst: RiscvInstWrapper) -> Result<u64, Exception> {
        let raw_inst = match inst {
            RiscvInstWrapper::Full(inst) | RiscvInstWrapper::Compact(inst) => inst,
        };

        match raw_inst {
            RiscvInst::Illegal => return Err(Exception::IllegalInstruction(self.pc)),
            RiscvInst::Lb { rd, rs1, imm } => {
                let addr = self
                    .mmu
                    .translate(addr_add(self.x[rs1 as usize], imm))
                    .expect("Translation failed");
                self.x[rd as usize] = self.bus.load_byte(addr).expect("Load failed") as i8 as u64;
            }
            RiscvInst::Lh { rd, rs1, imm } => {
                let addr = self
                    .mmu
                    .translate(addr_add(self.x[rs1 as usize], imm))
                    .expect("Translation failed");
                self.x[rd as usize] = self.bus.load_half(addr).expect("Load failed") as i16 as u64;
            }
            RiscvInst::Lw { rd, rs1, imm } => {
                let addr = self
                    .mmu
                    .translate(addr_add(self.x[rs1 as usize], imm))
                    .expect("Translation failed");
                self.x[rd as usize] = self.bus.load_word(addr).expect("Load failed") as i32 as u64;
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
                self.x[rd as usize] = (self.x[rs1 as usize] as i64).wrapping_shr(imm as u32) as u64;
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
                self.x[rd as usize] = (self.x[rs1 as usize].wrapping_add(imm as u64)) as u32 as u64;
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
                            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                            bytes[7],
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
                    return Ok(self.pc.wrapping_add(imm as u64));
                }
            }
            RiscvInst::Bne { rs1, rs2, imm } => {
                if self.x[rs1 as usize] != self.x[rs2 as usize] {
                    return Ok(self.pc.wrapping_add(imm as u64));
                }
            }
            RiscvInst::Blt { rs1, rs2, imm } => {
                if (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64) {
                    return Ok(self.pc.wrapping_add(imm as u64));
                }
            }
            RiscvInst::Bge { rs1, rs2, imm } => {
                if (self.x[rs1 as usize] as i64) >= (self.x[rs2 as usize] as i64) {
                    return Ok(self.pc.wrapping_add(imm as u64));
                }
            }
            RiscvInst::Bltu { rs1, rs2, imm } => {
                if self.x[rs1 as usize] < self.x[rs2 as usize] {
                    return Ok(self.pc.wrapping_add(imm as u64));
                }
            }
            RiscvInst::Bgeu { rs1, rs2, imm } => {
                if self.x[rs1 as usize] >= self.x[rs2 as usize] {
                    return Ok(self.pc.wrapping_add(imm as u64));
                }
            }
            RiscvInst::Jalr { rd, rs1, imm } => {
                let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
                self.x[rd as usize] = self.pc + 4;
                return Ok(addr & !1);
            }
            RiscvInst::Jal { rd, imm } => {
                self.x[rd as usize] = self.pc + 4;
                return Ok(self.pc.wrapping_add(imm as u64));
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
            RiscvInst::Mret => {
                let mut mstatus = self.csr.load(MSTATUS);
                // MPP is two bits wide at MSTATUS[12:11]
                self.mode = mstatus.mpp() as u8;
                // The MPIE bit is MSTATUS[7] and the MIE bit is the MSTATUS[3].
                let mpie = mstatus.mpie();
                // set MIE = MPIE
                mstatus = (mstatus & !MASK_MIE) | (mpie << 3);
                // set MPIE = 1
                mstatus.set(MASK_MPIE);
                // set MPP the least privilege mode (u-mode)
                mstatus.clear(MASK_MPP);
                // If MPP != M, sets MPRV=0
                mstatus.clear(MASK_MPRV);
                self.csr.store(MSTATUS, mstatus.into());
                // set the pc to CSRs[mepc].
                return Ok((self.csr.load(MEPC) & !0b11).into());
            }
            RiscvInst::Sret => {
                // When the SRET instruction is executed to return from the trap
                // handler, the privilege level is set to user mode if the SPP
                // bit is 0, or supervisor mode if the SPP bit is 1. The SPP bit
                // is SSTATUS[8].
                let mut sstatus = self.csr.load(SSTATUS);
                self.mode = sstatus.spp() as u8;
                // The SPIE bit is SSTATUS[5] and the SIE bit is the SSTATUS[1]
                let spie = sstatus.spie();
                // set SIE = SPIE
                sstatus = (sstatus & !MASK_SIE) | (spie << 1);
                // set SPIE = 1
                sstatus.set(MASK_SPIE);
                // set SPP the least privilege mode (u-mode)
                sstatus.clear(MASK_SPP);
                self.csr.store(SSTATUS, sstatus.into());
                // set the pc to CSRs[sepc].
                // whenever IALIGN=32, bit sepc[1] is masked on reads so that it appears to be 0. This
                // masking occurs also for the implicit read by the SRET instruction.
                return Ok((self.csr.load(SEPC) & !0b11).into());
            }
            RiscvInst::Wfi => todo!(),
            RiscvInst::SfenceVma { rs1, rs2 } => todo!(),
        };

        Ok(if inst.is_compact() {
            self.pc + 2
        } else {
            self.pc + 4
        })
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

    fn handle_exception(&mut self, e: Exception) {
        let pc = self.pc;
        let mode = self.mode;
        let cause = e.code();
        // if an exception happen in U-mode or S-mode, and the exception is delegated to S-mode.
        // then this exception should be handled in S-mode.
        let trap_in_s_mode = mode <= SUPERVISOR_MODE && self.csr.is_medelegated(cause);
        let (STATUS, TVEC, CAUSE, TVAL, EPC, MASK_PIE, pie_i, MASK_IE, ie_i, MASK_PP, pp_i) =
            if trap_in_s_mode {
                self.mode = SUPERVISOR_MODE;
                (
                    SSTATUS, STVEC, SCAUSE, STVAL, SEPC, MASK_SPIE, 5, MASK_SIE, 1, MASK_SPP, 8,
                )
            } else {
                self.mode = MACHINE_MODE;
                (
                    MSTATUS, MTVEC, MCAUSE, MTVAL, MEPC, MASK_MPIE, 7, MASK_MIE, 3, MASK_MPP, 11,
                )
            };
        // 3.1.7 & 4.1.2
        // The BASE field in tvec is a WARL field that can hold any valid virtual or physical address,
        // subject to the following alignment constraints: the address must be 4-byte aligned
        self.pc = (self.csr.load(TVEC) & !0b11).into();
        // 3.1.14 & 4.1.7
        // When a trap is taken into S-mode (or M-mode), sepc (or mepc) is written with the virtual address
        // of the instruction that was interrupted or that encountered the exception.
        self.csr.store(EPC, pc);
        // 3.1.15 & 4.1.8
        // When a trap is taken into S-mode (or M-mode), scause (or mcause) is written with a code indicating
        // the event that caused the trap.
        self.csr.store(CAUSE, cause);
        // 3.1.16 & 4.1.9
        // If stval is written with a nonzero value when a breakpoint, address-misaligned, access-fault, or
        // page-fault exception occurs on an instruction fetch, load, or store, then stval will contain the
        // faulting virtual address.
        // If stval is written with a nonzero value when a misaligned load or store causes an access-fault or
        // page-fault exception, then stval will contain the virtual address of the portion of the access that
        // caused the fault
        self.csr.store(TVAL, e.value());
        // 3.1.6 covers both sstatus and mstatus.
        let mut status = self.csr.load(STATUS);
        // get SIE or MIE
        let ie = Into::<u64>::into(status & MASK_IE) >> ie_i;
        // set SPIE = SIE / MPIE = MIE
        status = (status & !MASK_PIE) | (ie << pie_i);
        // set SIE = 0 / MIE = 0
        status.clear(MASK_IE);
        // set SPP / MPP = previous mode
        status = Csr {
            data: Into::<u64>::into(status & !MASK_PP) | (mode << pp_i) as u64,
        };
        self.csr.store(STATUS, status.into());
    }

    fn run(&mut self) {
        loop {
            self.x[0] = 0; // x0 is always 0
            let inst = self.fetch();
            let inst_with_len = match inst {
                Ok(inst) => inst,
                Err(_) => {
                    break;
                }
            };
            match self.execute(inst_with_len) {
                Ok(new_pc) => self.pc = new_pc,
                Err(e) => match e {
                    Exception::IllegalInstruction(_) => {
                        break;
                    }
                    _ => {
                        self.handle_exception(e);
                        break;
                    }
                },
            }
        }
    }

    type Exception = Exception;
}

#[cfg(test)]
mod test {

    use crate::{
        arch::riscv::reg::{A0, RA, SP},
        cpu::Cpu,
        util::addr_add,
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
        cpu.run();

        assert_eq!(cpu.x[10], 120u64);
    }
}
