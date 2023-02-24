pub trait Bus {
    type Exception;
    fn load(&mut self, addr: u64, size: u64) -> Result<u64, Self::Exception>;
    fn store(&mut self, addr: u64, size: u64, data: u64) -> Result<(), Self::Exception>;
}
