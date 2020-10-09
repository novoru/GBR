use crate::io::Io;

const ROM_SIZE: usize   = 32768;

enum Mbc {
    Mbc0,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
    Huc1,
}

pub struct Cartridge {
    rom:    [u8; ROM_SIZE],
    mbc:    Option<Mbc>,
}

impl Cartridge {
    pub fn new() -> Self {
        Cartridge {
            rom:    [0; ROM_SIZE],
            mbc:    None,
        }
    }
}

impl Io for Cartridge {
    fn read8(&self, addr: usize) -> u8 {
        self.rom[addr]
    }

    fn read16(&self, addr: usize) -> u16 {
        (self.rom[addr] as u16) << 8 | self.rom[addr+1] as u16
    }

    fn write8(&mut self, addr: usize, data: u8) {
        self.rom[addr] = data;
    }
    
    fn write16(&mut self, addr: usize, data: u16) {
        self.rom[addr] = (data >> 8) as u8;
        self.rom[addr+1] = (data & 0xFF) as u8;
    }
}