use crate::core::io::Io;

const RAM_SIZE: usize   = 8192;

pub struct Ram {
    ram:    [u8; RAM_SIZE],
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            ram:    [0; RAM_SIZE]
        }
    }
}

impl Io for Ram {
    fn read8(&self, addr: usize) -> u8 {
        self.ram[addr]
    }

    fn write8(&mut self, addr: usize, data: u8) {
        self.ram[addr] = data;
    }
}