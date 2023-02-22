use std::ops::{Index, IndexMut};

pub struct Memory {
    pub data: Vec<u8>,
    pub endianness: Endianness,
}

pub enum Endianness {
    Little,
    Big,
}

impl Memory {
    pub fn new(endianness: Endianness) -> Memory {
        Memory {
            data: vec![],
            endianness,
        }
    }

    pub fn init(&mut self, capacity: u64) {
        for _i in 0..capacity {
            self.data.push(0);
        }
    }

    pub fn load_data(&mut self, data: &[u8], addr: u64) {
        for i in 0..data.len() {
            self.data[addr as usize + i] = data[i];
        }
    }

    pub fn load(&self, addr: u64, size: u64) -> u64 {
        let mut val: u64 = 0;
        match self.endianness {
            Endianness::Little => {
                for i in 0..size {
                    val |= (self.data[addr.wrapping_add(i) as usize] as u64) << (i * 8);
                }
            }
            Endianness::Big => {
                for i in 0..size {
                    val |=
                        (self.data[addr.wrapping_add(i) as usize] as u64) << ((size - i - 1) * 8);
                }
            }
        }
        val
    }

    pub fn store(&mut self, addr: u64, size: u64, val: u64) {
        match self.endianness {
            Endianness::Little => {
                for i in 0..size {
                    self.data[addr.wrapping_add(i) as usize] = ((val >> (i * 8)) & 0xff) as u8;
                }
            }
            Endianness::Big => {
                for i in 0..size {
                    self.data[addr.wrapping_add(i) as usize] =
                        ((val >> ((size - i - 1) * 8)) & 0xff) as u8;
                }
            }
        }
    }

    pub fn read_u8(&self, addr: u64) -> u8 {
        self.load(addr, 1) as u8
    }

    pub fn read_u16(&self, addr: u64) -> u16 {
        self.load(addr, 2) as u16
    }

    pub fn read_u32(&self, addr: u64) -> u32 {
        self.load(addr, 4) as u32
    }

    pub fn read_u64(&self, addr: u64) -> u64 {
        self.load(addr, 8)
    }

    pub fn write_u8(&mut self, addr: u64, val: u8) {
        self.store(addr, 1, val as u64);
    }

    pub fn write_u16(&mut self, addr: u64, val: [u8; 2]) {
        self.store(addr, 1, val[0] as u64);
        self.store(addr + 1, 1, val[1] as u64);
    }

    pub fn write_u32(&mut self, addr: u64, val: [u8; 4]) {
        for i in 0..4 {
            self.store(addr + i, 1, val[i as usize] as u64);
        }
    }

    pub fn write_u64(&mut self, addr: u64, val: [u8; 8]) {
        for i in 0..8 {
            self.store(addr + i, 1, val[i as usize] as u64);
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
