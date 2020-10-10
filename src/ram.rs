use crate::io::Io;

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

    fn read16(&self, addr: usize) -> u16 {
        (self.ram[addr+1] as u16) << 8 | self.ram[addr] as u16
    }

    fn write8(&mut self, addr: usize, data: u8) {
        self.ram[addr] = data;
    }
    
    fn write16(&mut self, addr: usize, data: u16) {
        self.ram[addr+1] = (data >> 8) as u8;
        self.ram[addr] = (data & 0xFF) as u8;
    }
}