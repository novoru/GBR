pub trait Io {
    fn read8(&self, addr: usize) -> u8;
    fn write8(&mut self, addr: usize, data: u8);
}