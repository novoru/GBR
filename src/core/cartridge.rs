use crate::core::io::Io;
use crate::core::mbc::{ Mbc, new_nombc};

use std::path::Path;
use std::fs::read;
use std::fmt;

const ROM_SIZE:             usize   = 32768;
const TITLE_START:          usize   = 0x134;
const TITLE_END:            usize   = 0x142;
// const LICENSEE_CODE_START:  usize   = 0x144;
// const LICENSEE_CODE_END:    usize   = 0x145;
// const SGB_FLAG:             usize   = 0x146;
const CARTRIDGE_TYPE:       usize   = 0x147;
// const ROM_SIZE_ADDR:        usize   = 0x148;
// const RAM_SIZE_ADDR:        usize   = 0x149;
// const DESTINATION_CODE:     usize   = 0x14A;

pub struct Cartridge {
    rom:    Vec<u8>,
    title:  String,
    mbc:    Option<Mbc>,
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title:{}, mbc: {:?}", self.title, self.mbc)
    }
}

impl Cartridge {
    pub fn no_cartridge() -> Self {
        Cartridge {
            rom:        vec![0; ROM_SIZE],
            title:      "NO CARTRIDGE".to_string(),
            mbc:        None,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let bin = read(path).unwrap();
        let title = String::from_utf8(bin[TITLE_START..TITLE_END]
                    .to_vec())
                    .unwrap();

        let mbc = match bin[CARTRIDGE_TYPE] {
            // No MBC(ROM only)
            0x00    =>  Some(new_nombc()),
            _       =>  unimplemented!(),
        };

        Cartridge {
            rom:        bin,
            title:      title,
            mbc:        mbc,
        }
    }
}

impl Io for Cartridge {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF   =>  self.rom[addr],
            0x4000 ..= 0x7FFF   =>  self.rom[addr],
            0xA000 ..= 0xBFFF   =>  {
                match &self.mbc {
                    Some(mbc)   =>  mbc.read8(addr),
                    None        =>  panic!(),
                }
            },
            _                   =>  panic!(),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            0x0000 ..= 0x3FFF   =>  self.rom[addr] = data,
            0x4000 ..= 0x7FFF   =>  self.rom[addr] = data,
            0xA000 ..= 0xBFFF   =>  self.mbc.as_mut().unwrap().write8(addr, data),
            _                   =>  panic!(),
        }
    }
}