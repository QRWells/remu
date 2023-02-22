use super::{bus::*, exception::Exception};

pub struct Plic {
    pending: u64,
    senable: u64,
    spriority: u64,
    sclaim: u64,
}

pub(crate) const PLIC_PENDING: u64 = PLIC_BASE + 0x1000;
pub(crate) const PLIC_SENABLE: u64 = PLIC_BASE + 0x2000;
pub(crate) const PLIC_SPRIORITY: u64 = PLIC_BASE + 0x201000;
pub(crate) const PLIC_SCLAIM: u64 = PLIC_BASE + 0x201004;

impl Plic {
    pub fn new() -> Self {
        Self {
            pending: 0,
            senable: 0,
            spriority: 0,
            sclaim: 0,
        }
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, Exception> {
        if size != 32 {
            return Err(Exception::LoadAccessFault(addr));
        }
        match addr {
            PLIC_PENDING => Ok(self.pending),
            PLIC_SENABLE => Ok(self.senable),
            PLIC_SPRIORITY => Ok(self.spriority),
            PLIC_SCLAIM => Ok(self.sclaim),
            _ => Ok(0),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        if size != 32 {
            return Err(Exception::StoreAMOAccessFault(addr));
        }
        match addr {
            PLIC_PENDING => Ok(self.pending = value),
            PLIC_SENABLE => Ok(self.senable = value),
            PLIC_SPRIORITY => Ok(self.spriority = value),
            PLIC_SCLAIM => Ok(self.sclaim = value),
            _ => Ok(()),
        }
    }
}
