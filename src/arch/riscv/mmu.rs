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

type PTE = u64;

pub fn check_permission(pte: PTE, access: Accessibility, prv: u8, status: u64) -> Result<(), ()> {
    if pte & PTE_V == 0 {
        return Err(());
    }

    if prv == 0 {
        if pte & PTE_U == 0 {
            return Err(());
        }
    } else {
        if pte & PTE_U != 0 && status & (1 << 18) == 0 {
            return Err(());
        }
    }

    if pte & PTE_A == 0 {
        return Err(());
    }

    match access {
        Accessibility::Read => {
            if pte & PTE_R == 0 && (pte & PTE_X == 0 || status & (1 << 19) == 0) {
                return Err(());
            }
        }
        Accessibility::Write => {
            if pte & PTE_W == 0 || pte & PTE_D == 0 {
                return Err(());
            }
        }
        Accessibility::Execute => {
            if pte & PTE_X == 0 {
                return Err(());
            }
        }
    }

    Ok(())
}
