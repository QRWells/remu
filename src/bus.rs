pub trait Bus {
    type Exception;
    fn load(&self, addr: u64, size: usize) -> Result<u64, Self::Exception>;
    fn store(&mut self, addr: u64, size: usize, data: u64) -> Result<(), Self::Exception>;
}
