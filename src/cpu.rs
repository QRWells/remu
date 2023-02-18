pub trait Cpu {
    fn init(&mut self);
    fn reset(&mut self);
    fn load(&mut self, data: Vec<u8>);
    fn execute(&mut self);
}
