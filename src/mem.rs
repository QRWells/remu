use std::ops::{Index, IndexMut};

pub struct Memory {
    pub mem: Vec<u8>,
    pub size: usize,
    pub endianness: Endianness,
}

pub enum Endianness {
    Little,
    Big,
}

impl Memory {
    pub fn new(size: usize, endianness: Endianness) -> Memory {
        Memory {
            mem: vec![0; size],
            size,
            endianness,
        }
    }

    pub fn load_data(&mut self, data: &[u8], addr: usize) {
        for i in 0..data.len() {
            self.mem[addr + i] = data[i];
        }
    }

    pub fn load(&self, addr: usize, size: usize) -> u64 {
        let mut val: u64 = 0;
        match self.endianness {
            Endianness::Little => {
                for i in 0..size {
                    val |= (self.mem[addr + i] as u64) << (i * 8);
                }
            }
            Endianness::Big => {
                for i in 0..size {
                    val |= (self.mem[addr + i] as u64) << ((size - i - 1) * 8);
                }
            }
        }
        val
    }

    pub fn store(&mut self, addr: usize, size: usize, val: u64) {
        match self.endianness {
            Endianness::Little => {
                for i in 0..size {
                    self.mem[addr + i] = ((val >> (i * 8)) & 0xff) as u8;
                }
            }
            Endianness::Big => {
                for i in 0..size {
                    self.mem[addr + i] = ((val >> ((size - i - 1) * 8)) & 0xff) as u8;
                }
            }
        }
    }

    pub fn read_u8(&self, addr: usize) -> u8 {
        self.load(addr, 1) as u8
    }

    pub fn read_u16(&self, addr: usize) -> u16 {
        self.load(addr, 2) as u16
    }

    pub fn read_u32(&self, addr: usize) -> u32 {
        self.load(addr, 4) as u32
    }

    pub fn read_u64(&self, addr: usize) -> u64 {
        self.load(addr, 8)
    }

    pub fn write_u8(&mut self, addr: usize, val: u8) {
        self.store(addr, 1, val as u64);
    }

    pub fn write_u16(&mut self, addr: usize, val: u16) {
        self.store(addr, 2, val as u64);
    }

    pub fn write_u32(&mut self, addr: usize, val: u32) {
        self.store(addr, 4, val as u64);
    }

    pub fn write_u64(&mut self, addr: usize, val: u64) {
        self.store(addr, 8, val);
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mem[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.mem[index]
    }
}
