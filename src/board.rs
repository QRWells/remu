use crate::cpu::Cpu;

pub struct Board {
    cpu: Box<dyn Cpu>,
}
