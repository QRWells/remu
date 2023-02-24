pub trait Cpu {
    type Exception;
    type Interrupt;
    fn init(&mut self);
    fn reset(&mut self);
    fn load(&mut self, data: Vec<u8>);
    fn handle_interrupt(&mut self, int: Self::Interrupt);
    fn handle_exception(&mut self, e: Self::Exception);
    fn run(&mut self);
}
