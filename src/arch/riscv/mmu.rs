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
}

pub enum AddressingMode {
    Bare,
    Sv32,
    Sv39,
    Sv48,
    Sv57,
}

pub struct MMU {
    addressing_mode: AddressingMode,
}

impl MMU {
    pub fn new() -> Self {
        Self {
            addressing_mode: AddressingMode::Bare,
        }
    }

    pub fn translate(&self, addr: u64) -> Result<u64, ()> {
        match self.addressing_mode {
            AddressingMode::Bare => self.translate_bare(addr),
            AddressingMode::Sv32
            | AddressingMode::Sv39
            | AddressingMode::Sv48
            | AddressingMode::Sv57 => todo!("translate sv32, sv39, sv48, sv57"),
        }
    }

    fn translate_bare(&self, addr: u64) -> Result<u64, ()> {
        Ok(addr)
    }
}
