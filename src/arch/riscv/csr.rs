use std::{
    default,
    ops::{BitAnd, BitOr, Index, Not},
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Csrs {
    csrs: [Csr; 4096],
}

impl Csrs {
    pub fn new() -> Csrs {
        Self {
            csrs: [Csr::from(0); 4096],
        }
    }

    pub fn load(&self, addr: usize) -> Csr {
        match addr {
            SIE => self.csrs[MIE] & self.csrs[MIDELEG],
            SIP => self.csrs[MIP] & self.csrs[MIDELEG],
            SSTATUS => self.csrs[MSTATUS] & MASK_SSTATUS,
            _ => self.csrs[addr],
        }
    }

    pub fn store(&mut self, addr: usize, value: u64) {
        match addr {
            SIE => {
                self.csrs[MIE] =
                    (self.csrs[MIE] & !self.csrs[MIDELEG]) | (self.csrs[MIDELEG] & value)
            }
            SIP => {
                self.csrs[MIP] =
                    (self.csrs[MIE] & !self.csrs[MIDELEG]) | (self.csrs[MIDELEG] & value)
            }
            SSTATUS => {
                self.csrs[MSTATUS] = (self.csrs[MSTATUS] & !MASK_SSTATUS) | (value & MASK_SSTATUS)
            }
            _ => self.csrs[addr] = value.into(),
        }
    }

    pub fn is_medelegated(&self, cause: u64) -> bool {
        (self.csrs[MEDELEG].data.wrapping_shr(cause as u32) & 1) == 1
    }
}

impl Index<u16> for Csrs {
    type Output = Csr;

    fn index(&self, index: u16) -> &Self::Output {
        &self.csrs[index as usize]
    }
}

impl Index<usize> for Csrs {
    type Output = Csr;

    fn index(&self, index: usize) -> &Self::Output {
        &self.csrs[index]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Csr {
    pub(crate) data: u64,
}

impl Csr {
    pub fn mpp(&self) -> u64 {
        (self.data & MASK_MPP) >> 11
    }

    pub fn mpie(&self) -> u64 {
        (self.data & MASK_MPIE) >> 7
    }

    pub fn mie(&self) -> u64 {
        (self.data & MASK_MIE) >> 3
    }

    pub fn spp(&self) -> u64 {
        (self.data & MASK_SPP) >> 8
    }

    pub fn spie(&self) -> u64 {
        (self.data & MASK_SPIE) >> 5
    }

    pub fn sie(&self) -> u64 {
        (self.data & MASK_SIE) >> 1
    }

    pub fn set(&mut self, value: u64) {
        self.data = value;
    }

    pub fn clear(&mut self, value: u64) {
        self.data &= !value;
    }
}

impl BitAnd for Csr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data & rhs.data,
        }
    }
}

impl BitAnd<u64> for Csr {
    type Output = Self;

    fn bitand(self, rhs: u64) -> Self::Output {
        Self {
            data: self.data & rhs,
        }
    }
}

impl BitOr for Csr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data | rhs.data,
        }
    }
}

impl BitOr<u64> for Csr {
    type Output = Self;

    fn bitor(self, rhs: u64) -> Self::Output {
        Self {
            data: self.data | rhs,
        }
    }
}

impl Not for Csr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { data: !self.data }
    }
}

impl From<u64> for Csr {
    fn from(data: u64) -> Self {
        Self { data }
    }
}

impl From<Csr> for u64 {
    fn from(val: Csr) -> Self {
        val.data
    }
}

/// Get the minimal privilege level required to access the CSR
pub fn csr_min_prv_level(addr: u16) -> u8 {
    ((addr >> 8) & 0b11) as u8
}

pub fn csr_readonly(addr: u16) -> bool {
    (addr >> 10) & 0b11 == 0b11
}

pub const FFLAGS: usize = 0x001;
pub const FRM: usize = 0x002;
pub const FCSR: usize = 0x003;

pub const CYCLE: usize = 0xC00;
pub const TIME: usize = 0xC01;
pub const INSTRET: usize = 0xC02;

// These CSRs are Rv32I only, and they are considered invalid in RV64I
pub const CYCLEH: usize = 0xC80;
pub const TIMEH: usize = 0xC81;
pub const INSTRETH: usize = 0xC82;

pub const SSTATUS: usize = 0x100;
pub const SIE: usize = 0x104;
pub const STVEC: usize = 0x105;
pub const SCOUNTEREN: usize = 0x106;
pub const SSCRATCH: usize = 0x140;
pub const SEPC: usize = 0x141;
pub const SCAUSE: usize = 0x142;
pub const STVAL: usize = 0x143;
pub const SIP: usize = 0x144;
pub const SATP: usize = 0x180;

pub const MVENDORID: usize = 0xF11;
pub const MARCHID: usize = 0xF12;
pub const MIMPID: usize = 0xF13;
pub const MHARTID: usize = 0xF14;
pub const MSTATUS: usize = 0x300;
pub const MISA: usize = 0x301;
pub const MEDELEG: usize = 0x302;
pub const MIDELEG: usize = 0x303;
pub const MIE: usize = 0x304;
pub const MTVEC: usize = 0x305;
pub const MCOUNTEREN: usize = 0x306;
pub const MSCRATCH: usize = 0x340;
pub const MEPC: usize = 0x341;
pub const MCAUSE: usize = 0x342;
pub const MTVAL: usize = 0x343;
pub const MIP: usize = 0x344;

pub const MCYCLE: usize = 0xB00;
pub const MTIME: usize = 0xB01;
pub const MINSTRET: usize = 0xB02;

// mstatus and sstatus field mask
pub const MASK_SIE: u64 = 1 << 1;
pub const MASK_MIE: u64 = 1 << 3;
pub const MASK_SPIE: u64 = 1 << 5;
pub const MASK_UBE: u64 = 1 << 6;
pub const MASK_MPIE: u64 = 1 << 7;
pub const MASK_SPP: u64 = 1 << 8;
pub const MASK_VS: u64 = 0b11 << 9;
pub const MASK_MPP: u64 = 0b11 << 11;
pub const MASK_FS: u64 = 0b11 << 13;
pub const MASK_XS: u64 = 0b11 << 15;
pub const MASK_MPRV: u64 = 1 << 17;
pub const MASK_SUM: u64 = 1 << 18;
pub const MASK_MXR: u64 = 1 << 19;
pub const MASK_TVM: u64 = 1 << 20;
pub const MASK_TW: u64 = 1 << 21;
pub const MASK_TSR: u64 = 1 << 22;
pub const MASK_UXL: u64 = 0b11 << 32;
pub const MASK_SXL: u64 = 0b11 << 34;
pub const MASK_SBE: u64 = 1 << 36;
pub const MASK_MBE: u64 = 1 << 37;
pub const MASK_SD: u64 = 1 << 63;
pub const MASK_SSTATUS: u64 = MASK_SIE
    | MASK_SPIE
    | MASK_UBE
    | MASK_SPP
    | MASK_FS
    | MASK_XS
    | MASK_SUM
    | MASK_MXR
    | MASK_UXL
    | MASK_SD;

// MIP / SIP field mask
pub const MASK_SSIP: u64 = 1 << 1;
pub const MASK_MSIP: u64 = 1 << 3;
pub const MASK_STIP: u64 = 1 << 5;
pub const MASK_MTIP: u64 = 1 << 7;
pub const MASK_SEIP: u64 = 1 << 9;
pub const MASK_MEIP: u64 = 1 << 11;
