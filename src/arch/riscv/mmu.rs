use crate::bus::Bus;

use super::{bus::RiscvBus, exception::Exception};

pub const PAGE_SIZE: u64 = 4096;

pub const PTE_V: u64 = 0x1 << 0;
pub const PTE_R: u64 = 0x1 << 1;
pub const PTE_W: u64 = 0x1 << 2;
pub const PTE_X: u64 = 0x1 << 3;
pub const PTE_U: u64 = 0x1 << 4;
pub const PTE_G: u64 = 0x1 << 5;
pub const PTE_A: u64 = 0x1 << 6;
pub const PTE_D: u64 = 0x1 << 7;

/// Type of access. This excludes STATUS, PRV and other states that may influence permission check.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Accessibility {
    Read,
    Write,
    Execute,
}

pub struct PageTableEntry64(u64);

impl PageTableEntry64 {
    pub fn check_permission(&self, access: Accessibility, prv: u8, status: u64) -> Result<(), ()> {
        if self.0 & PTE_V == 0 {
            return Err(());
        }

        if prv == 0 {
            if self.0 & PTE_U == 0 {
                return Err(());
            }
        } else {
            if self.0 & PTE_U != 0 && status & (1 << 18) == 0 {
                return Err(());
            }
        }

        if self.0 & PTE_A == 0 {
            return Err(());
        }

        match access {
            Accessibility::Read => {
                if self.0 & PTE_R == 0 && (self.0 & PTE_X == 0 || status & (1 << 19) == 0) {
                    return Err(());
                }
            }
            Accessibility::Write => {
                if self.0 & PTE_W == 0 || self.0 & PTE_D == 0 {
                    return Err(());
                }
            }
            Accessibility::Execute => {
                if self.0 & PTE_X == 0 {
                    return Err(());
                }
            }
        }

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.0 & PTE_V != 0
    }

    pub fn is_readable(&self) -> bool {
        self.0 & PTE_R != 0
    }

    pub fn is_writable(&self) -> bool {
        self.0 & PTE_W != 0
    }

    pub fn is_executable(&self) -> bool {
        self.0 & PTE_X != 0
    }

    pub fn get_ppns(&self, mode: AddressingMode) -> [u64; 5] {
        match mode {
            AddressingMode::Bare => [0; 5],
            AddressingMode::Sv32 => [(self.0 >> 10) & 0x3ff, (self.0 >> 20) & 0xfff, 0, 0, 0],
            AddressingMode::Sv39 => [
                (self.0 >> 10) & 0x1ff,
                (self.0 >> 19) & 0x1ff,
                (self.0 >> 28) & 0x3ff_ffff,
                0,
                0,
            ],
            AddressingMode::Sv48 => [
                (self.0 >> 10) & 0x1ff,
                (self.0 >> 19) & 0x1ff,
                (self.0 >> 28) & 0x1ff,
                (self.0 >> 37) & 0x1_ffff,
                0,
            ],
            AddressingMode::Sv57 => [
                (self.0 >> 10) & 0x1ff,
                (self.0 >> 19) & 0x1ff,
                (self.0 >> 28) & 0x1ff,
                (self.0 >> 37) & 0x1ff,
                (self.0 >> 46) & 0x1ff,
            ],
        }
    }

    pub fn get_ppn(&self, mode: AddressingMode) -> u64 {
        match mode {
            AddressingMode::Bare => 0,
            AddressingMode::Sv32 => self.0 >> 10 & 0x3f_ffff,
            AddressingMode::Sv39 | AddressingMode::Sv48 | AddressingMode::Sv57 => {
                self.0 >> 10 & 0x3ff_ffff_ffff
            }
        }
    }
}

impl From<u64> for PageTableEntry64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

pub enum AddressingMode {
    Bare,
    Sv32,
    Sv39,
    Sv48,
    Sv57,
}

pub enum AccessType {
    Load,
    Store,
    Instruction,
}

pub struct MMU {
    addressing_mode: AddressingMode,
    physical_page_number: u64,
}

impl MMU {
    pub fn new() -> Self {
        Self {
            addressing_mode: AddressingMode::Bare,
            physical_page_number: 0,
        }
    }

    pub fn set_ppn(&mut self, satp: u64) {
        self.physical_page_number = satp & 0xfff_ffff_ffff;
    }

    pub fn translate(
        &self,
        access_type: AccessType,
        bus: &mut RiscvBus,
        addr: u64,
    ) -> Result<u64, Exception> {
        match self.addressing_mode {
            AddressingMode::Bare => self.translate_bare(addr),
            AddressingMode::Sv39 => self.translate_sv39(access_type, bus, addr),
            AddressingMode::Sv32 | AddressingMode::Sv48 | AddressingMode::Sv57 => {
                todo!("translate sv32, sv48, sv57")
            }
        }
    }

    fn translate_bare(&self, addr: u64) -> Result<u64, Exception> {
        Ok(addr)
    }

    fn translate_sv39(
        &self,
        access_type: AccessType,
        bus: &mut RiscvBus,
        addr: u64,
    ) -> Result<u64, Exception> {
        let levels = 3;

        let vpn = [
            (addr >> 12) & 0x1ff,
            (addr >> 21) & 0x1ff,
            (addr >> 30) & 0x1ff,
        ];

        let mut root = self.physical_page_number << 12;
        let mut i = levels - 1;
        let mut pte: PageTableEntry64;

        let err: Result<u64, Exception> = match access_type {
            AccessType::Instruction => Err(Exception::InstructionPageFault(addr)),
            AccessType::Load => Err(Exception::LoadPageFault(addr)),
            AccessType::Store => Err(Exception::StoreAMOPageFault(addr)),
        };

        loop {
            pte = bus.load(root + vpn[i as usize] * 8, 8)?.into();

            if !pte.is_valid() || (!pte.is_readable() && pte.is_writable()) {
                return err;
            }

            if pte.is_readable() || pte.is_executable() {
                break;
            }

            root = pte.get_ppn(AddressingMode::Sv39) << 12;

            i -= 1;
            if i < 0 {
                return err;
            }
        }

        let ppn = pte.get_ppns(AddressingMode::Sv39);

        let offset = addr & 0xfff;
        match i {
            0 => Ok((pte.get_ppn(AddressingMode::Sv39) << 12) | offset),
            1 => Ok((ppn[2] << 30) | (ppn[1] << 21) | (vpn[0] << 12) | offset),
            2 => Ok((ppn[2] << 30) | (vpn[1] << 21) | (vpn[0] << 12) | offset),
            _ => err,
        }
    }
}
