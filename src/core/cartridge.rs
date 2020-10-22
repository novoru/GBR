use crate::core::io::Io;

use std::path::Path;
use std::fs::read;
use std::fmt;

const ROM_SIZE:     usize   = 32768;
const SIZE_ADDR:    usize   = 0x148;
const TITLE_START:  usize   = 0x134;
const TITLE_END:    usize   = 0x142;

#[derive(Debug)]
enum Mbc {
    Mbc0,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
    Huc1,
}

pub struct Cartridge {
    rom:    Vec<u8>,
    title:  String,
    size:   u8,
    mbc:    Option<Mbc>,
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title:{}, size: {}, mbc: {:?}", self.title, self.size, self.mbc)
    }
}

impl Cartridge {
    pub fn no_cartridge() -> Self {
        Cartridge {
            rom:    vec![0;ROM_SIZE],
            title:  "NO CARTRIDGE".to_string(),
            size:   0,
            mbc:    None,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let bin = read(path).unwrap();
        let title = String::from_utf8(bin[TITLE_START..TITLE_END]
                    .to_vec())
                    .unwrap();
        let size = bin[SIZE_ADDR];

        Cartridge {
            rom:    bin,
            title:  title,
            size:   size,
            mbc:    None,
        }
    }
}

impl Io for Cartridge {
    fn read8(&self, addr: usize) -> u8 {
        self.rom[addr]
    }

    fn write8(&mut self, addr: usize, data: u8) {
        self.rom[addr] = data;
    }
}