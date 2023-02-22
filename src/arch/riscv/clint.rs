use super::{bus::*, exception::Exception};

pub struct Clint {
    mtime: u64,
    mtimecmp: u64,
}

pub(crate) const CLINT_MTIMECMP: u64 = CLINT_BASE + 0x4000;
pub(crate) const CLINT_MTIME: u64 = CLINT_BASE + 0xbff8;

impl Clint {
    pub fn new() -> Self {
        Self {
            mtime: 0,
            mtimecmp: 0,
        }
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, Exception> {
        if size != 64 {
            return Err(Exception::LoadAccessFault(addr));
        }
        match addr {
            CLINT_MTIMECMP => Ok(self.mtimecmp),
            CLINT_MTIME => Ok(self.mtime),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        if size != 64 {
            return Err(Exception::LoadAccessFault(addr));
        }
        match addr {
            CLINT_MTIMECMP => Ok(self.mtimecmp = value),
            CLINT_MTIME => Ok(self.mtime = value),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }
}
