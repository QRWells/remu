pub trait Cpu {
    type Exception;
    fn init(&mut self);
    fn reset(&mut self);
    fn load(&mut self, data: Vec<u8>);
    fn handle_exception(&mut self, e: Self::Exception);
    fn run(&mut self);
}
