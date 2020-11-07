use crate::core::io::Io;

use std::path::Path;
use std::fs::read;

const _ROM_SIZE:             usize   = 32768;
const TITLE_START:          usize   = 0x134;
const TITLE_END:            usize   = 0x142;
// const LICENSEE_CODE_START:  usize   = 0x144;
// const LICENSEE_CODE_END:    usize   = 0x145;
// const SGB_FLAG:             usize   = 0x146;
const CARTRIDGE_TYPE:       usize   = 0x147;
// const ROM_SIZE_ADDR:        usize   = 0x148;
// const RAM_SIZE_ADDR:        usize   = 0x149;
// const DESTINATION_CODE:     usize   = 0x14A;

pub enum BankMode {
    RamBank = 0,
    RomBank = 1,
}

pub enum Cartridge {
    NoMbc {
        rom:    Vec<u8>,
        title:  String,
    },

    Mbc1 {
        rom:            Vec<u8>,
        rombank:        u8,
        title:          String,
        ram:            Vec<u8>,
        rambank:        u8,
        ram_enabled:    bool,
        mode:           BankMode,
    },
}

impl Cartridge {
    pub fn _no_cartridge() -> Self {
        Cartridge::NoMbc {
            rom:        vec![0; _ROM_SIZE],
            title:      "NO CARTRIDGE".to_string(),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let bin = read(path).unwrap();
        let title = String::from_utf8(bin[TITLE_START..TITLE_END]
                    .to_vec())
                    .unwrap();
        let ramsize = match bin[0x149] {
            0   =>  0,
            1   =>  16*1024,    // 16kbit
            2   =>  64*1024,    // 64kbit
            3   =>  256*1024,   // 256kbit
            4   =>  1024*1024,  // 1Mbit
            _   =>  panic!(),
        };

        match bin[CARTRIDGE_TYPE] {
            // No MBC(ROM only)
            0x00    =>  Cartridge::NoMbc {
                            rom:    bin,
                            title:  title,
                        },
            0x01    =>  Cartridge::Mbc1 {
                            rom:            bin,
                            rombank:        1,
                            title:          title,
                            ram:            vec![0; ramsize],
                            rambank:        0,
                            ram_enabled:    false,
                            mode:           BankMode::RomBank,
                        },
            _       =>  unimplemented!("can't load: mbc type={}", bin[CARTRIDGE_TYPE]),
        }
    }
}


impl Io for Cartridge {
    fn read8(&self, addr: usize) -> u8 {
        match self {
            Cartridge::NoMbc { rom, .. }  =>  match addr {
                0x0000 ..= 0x7FFF   =>  rom[addr],
                _                   =>  panic!(),
            },
            Cartridge::Mbc1 { rom, rombank, ram, rambank, .. }  =>  match addr {
                0x0000 ..= 0x3FFF   =>  rom[addr],
                0x4000 ..= 0x7FFF   =>  rom[addr+0x4000*(*rombank as usize - 1)],
                0xA000 ..= 0xBFFF   =>  ram[addr-0xA000+0x2000*(*rambank as usize)],
                _                   =>  panic!(),
            },
        }

    }

    fn write8(&mut self, addr: usize, data: u8) {
        match self {
            Cartridge::NoMbc { rom, .. }  =>  match addr {
                0x0000 ..= 0x7FFF   =>  rom[addr] = data,
                _                   =>  panic!(),
            },
            Cartridge::Mbc1 { rombank, ram, rambank, ram_enabled, mode, .. }  =>  match addr {
                0x0000 ..= 0x1FFF   =>  *ram_enabled = data&0x0F == 0x0A,
                0x2000 ..= 0x3FFF   =>  *rombank = data&0x1F,
                0x4000 ..= 0x5FFF   =>  match mode {
                    BankMode::RamBank   => *rambank = data&0x03,
                    BankMode::RomBank   => *rombank |= (data&0x03) << 5,
                }
                0x6000 ..= 0x7FFF   =>  match data&0x01 == 0x00 {
                    true    =>  *mode = BankMode::RomBank,
                    false   =>  *mode = BankMode::RamBank,
                },
                0xA000 ..= 0xBFFF   =>  if *ram_enabled {
                    match mode {
                        BankMode::RamBank   =>  ram[addr-0xA000+0x2000*(*rambank as usize)] = data,
                        BankMode::RomBank   =>  ram[addr-0xA000] = data,
                    }
                },
                _                   =>  panic!(),
            },
        }
    }
}