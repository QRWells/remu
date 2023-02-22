use crate::{bus::Bus, mem::Memory};

use super::{clint, exception::Exception, plic};

pub struct RiscvBus {
    mem: Memory,
    plic: plic::Plic,
    clint: clint::Clint,
}

const DRAM_BASE: u64 = 0x8000_0000;
const DRAM_SIZE: u64 = 1024 * 1024 * 128;
const DRAM_END: u64 = DRAM_SIZE + DRAM_BASE - 1;

pub(crate) const PLIC_BASE: u64 = 0xc00_0000;
pub(crate) const PLIC_SIZE: u64 = 0x4000000;
pub(crate) const PLIC_END: u64 = PLIC_BASE + PLIC_SIZE - 1;

pub(crate) const CLINT_BASE: u64 = 0x200_0000;
pub(crate) const CLINT_SIZE: u64 = 0x10000;
pub(crate) const CLINT_END: u64 = CLINT_BASE + CLINT_SIZE - 1;

impl RiscvBus {
    pub fn new() -> Self {
        Self {
            mem: Memory::new(crate::mem::Endianness::Little),
            plic: plic::Plic::new(),
            clint: clint::Clint::new(),
        }
    }

    pub fn init(&mut self) {
        self.mem.init(DRAM_SIZE);
    }

    pub fn load_byte(&self, addr: u64) -> Result<u8, Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.read_u8(addr - DRAM_BASE)),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn load_data(&mut self, addr: u64, data: &[u8]) -> Result<(), Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.load_data(data, addr - DRAM_BASE)),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn load_half(&self, addr: u64) -> Result<u16, Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.read_u16(addr - DRAM_BASE)),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn load_word(&self, addr: u64) -> Result<u32, Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.read_u32(addr - DRAM_BASE)),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn load_double(&self, addr: u64) -> Result<u64, Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.read_u64(addr - DRAM_BASE)),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn store_byte(&mut self, addr: u64, data: u8) -> Result<(), Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.write_u8(addr - DRAM_BASE, data)),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }

    pub fn store_half(&mut self, addr: u64, data: [u8; 2]) -> Result<(), Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.write_u16(addr - DRAM_BASE, data)),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }

    pub fn store_word(&mut self, addr: u64, data: [u8; 4]) -> Result<(), Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.write_u32(addr - DRAM_BASE, data)),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }

    pub fn store_double(&mut self, addr: u64, data: [u8; 8]) -> Result<(), Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.write_u64(addr - DRAM_BASE, data)),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }
}

impl Bus for RiscvBus {
    type Exception = Exception;

    fn load(&self, addr: u64, size: u64) -> Result<u64, Self::Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.load(addr - DRAM_BASE, size)),
            PLIC_BASE..=PLIC_END => self.plic.load(addr - PLIC_BASE, size),
            CLINT_BASE..=CLINT_END => self.clint.load(addr - CLINT_BASE, size),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    fn store(&mut self, addr: u64, size: u64, data: u64) -> Result<(), Self::Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => Ok(self.mem.store(addr - DRAM_BASE, size, data)),
            PLIC_BASE..=PLIC_END => self.plic.store(addr - PLIC_BASE, size, data),
            CLINT_BASE..=CLINT_END => self.clint.store(addr - CLINT_BASE, size, data),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }
}
