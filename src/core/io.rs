pub trait Io {
    fn read8(&self, addr: usize) -> u8;
    fn write8(&mut self, addr: usize, data: u8);
    fn read16(&self, addr: usize) -> u16;
    fn write16(&mut self, addr: usize, data: u16);
}