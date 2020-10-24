use crate::core::io::Io;

use std::fmt;

const RAM_SIZE: usize = 8192;

pub enum Mbc {
    NoMbc {
        rom:    [u8; RAM_SIZE],
    },
}

pub fn new_nombc() -> Mbc {
    Mbc::NoMbc {
        rom:    [0; RAM_SIZE],
    }
}

impl fmt::Debug for Mbc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mbc::NoMbc{rom: _} =>  write!(f, "NoMbc"),
        }
    }
}

impl Io for Mbc {
    fn read8(&self, addr: usize) -> u8 {
        match self {
            Mbc::NoMbc{rom} =>  rom[addr-0xA000],
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match self {
            Mbc::NoMbc {rom}  =>  rom[addr-0xA000] = data,
        }
    }
}