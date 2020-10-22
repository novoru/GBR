use crate::core::io::Io;

const HRAM_SIZE: usize   = 128;

pub struct HRam {
    ram:    [u8; HRAM_SIZE],
}

impl HRam {
    pub fn new() -> Self {
        HRam {
            ram:    [0; HRAM_SIZE]
        }
    }
}

impl Io for HRam {
    fn read8(&self, addr: usize) -> u8 {
        self.ram[addr]
    }

    fn write8(&mut self, addr: usize, data: u8) {
        self.ram[addr] = data;
    }
}