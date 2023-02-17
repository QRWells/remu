use crate::mem::Memory;

pub trait Cpu {
    fn new(mem: Memory) -> Self;
    fn reset(&mut self);
    fn load(&mut self, data: Vec<u8>);
    fn execute(&mut self);
}
