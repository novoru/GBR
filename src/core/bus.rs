use crate::core::io::Io;
use crate::core::ram::Ram;
use crate::core::cartridge::Cartridge;
use crate::core::interrupt::*;
use crate::core::pad::{ Pad, Key };
use crate::core::ppu::*;
use crate::core::hram::HRam;

use std::path::Path;

const DMA_START_ADDR: usize = 0xFF46;
const OAM_START_ADDR: usize = 0xFE00;

pub struct Bus {
    cartridge:  Cartridge,
    ram:        Ram,
    hram:       HRam,
    ppu:        Ppu,
    interrupt:  Interrupt,
    pad:        Pad,
}

impl Bus {
    pub fn no_cartridge() -> Self {
        Bus {
            cartridge:  Cartridge::no_cartridge(),
            ram:        Ram::new(),
            hram:       HRam::new(),
            ppu:        Ppu::new(),
            interrupt:  Interrupt::new(),
            pad:        Pad::new(),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        Bus {
            cartridge:  Cartridge::from_path(path),
            ram:        Ram::new(),
            hram:       HRam::new(),
            ppu:        Ppu::new(),
            interrupt:  Interrupt::new(),
            pad:        Pad::new(),
        }
    }

    pub fn enable_irq(&mut self) {
        self.interrupt.enable()
    }

    pub fn disable_irq(&mut self) {
        self.interrupt.disable()
    }

    pub fn key_push(&mut self, key: Key) {
        self.pad.key_push(key);
    }

    pub fn key_release(&mut self, key: Key) {
        self.pad.key_release(key);
    }

    pub fn get_pixels(&self) -> [u8; SCREEN_WIDTH*SCREEN_HEIGHT] {
        self.ppu.get_pixels()
    }

    pub fn transfer(&mut self) {
        if self.ppu.dma_started() {
            for i in 0..0xA0 {
                let addr = (self.read8(DMA_START_ADDR) as usize * 0x100 + i) as usize;
                let data = self.read8(addr);
                self.write8(OAM_START_ADDR + i, data);
            }
            self.ppu.stop_dma();
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick();
    }
}

impl Io for Bus {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            // 16kB ROM bank #0
            0x0000 ..= 0x3FFF   =>  self.cartridge.read8(addr),
            // 16kB switchable ROM bank
            0x4000 ..= 0x7FFF   =>  self.cartridge.read8(addr),
            // 8kB Video RAM
            0x8000 ..= 0x9FFF   =>  self.ppu.read8(addr),
            // 8kB switchable RAM ban
            0xA000 ..= 0xBFFF   =>  self.cartridge.read8(addr),
            // 8kB Internal RAM
            0xC000 ..= 0xDFFF   =>  self.ram.read8(addr&0x1FFF),
            // Echo of 8kB Internal RAM
            0xE000 ..= 0xFDFF   =>  self.ram.read8(addr&0x1FFF),
            // Sprite Attribute Memory (OAM)
            0xFE00 ..= 0xFE9F   =>  self.ppu.read8(addr),
            // Empty but unusable for I/O
            0xFEA0 ..= 0xFEFF   =>  panic!("unsupport read at {:04x}", addr),
            // I/O ports
            0xFF00              =>  self.pad.read8(),
            // 0xFF00 ..= 0xFF3B   =>  self.ioports.read8(addr),
            // Interrupt Flag Register
            0xFF0F              =>  self.interrupt.read8(addr),
            // LCD Registers
            0xFF40 ..= 0xFF4B    =>  self.ppu.read8(addr),
            // Empty but unusable for I/O
            0xFF4C ..= 0xFF7F   =>  panic!("unsupport read at {:04x}", addr),
            // Internal RAM
            0xFF80 ..= 0xFFFE   =>  self.hram.read8(addr&0x7F),
            // Interrupt Enable Register
            0xFFFF              =>  self.interrupt.read8(addr),
            _                   =>  unimplemented!("0x{:08x}", addr),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            // 16kB ROM bank #0
            0x0000 ..= 0x3FFF   =>  self.cartridge.write8(addr, data),
            // 16kB switchable ROM bank
            0x4000 ..= 0x7FFF   =>  self.cartridge.write8(addr, data),
            // 8kB Video RAM
            0x8000 ..= 0x9FFF   =>  self.ppu.write8(addr, data),
            // 8kB switchable RAM bank
            0xA000 ..= 0xBFFF   =>  self.cartridge.write8(addr, data),
            // 8kB Internal RAM
            0xC000 ..= 0xDFFF   =>  self.ram.write8(addr&0x1FFF, data),
            // Echo of 8kB Internal RAM
            0xE000 ..= 0xFDFF   =>  self.ram.write8(addr&0x1FFF, data),
            // Sprite Attribute Memory (OAM)
            0xFE00 ..= 0xFE9F   =>  self.ppu.write8(addr, data),
            // Empty but unusable for I/O
            0xFEA0 ..= 0xFEFF   =>  panic!("unsupport write at {:04x}", addr),
            // I/O ports
            0xFF00              =>  self.pad.write8(data),            
            // 0xFF00 ..= 0xFF3B   =>  self.ioports.write8(addr),
            // Interrupt Flag Register
            0xFF0F              =>  self.interrupt.write8(addr, data),
            // LCD Registers
            0xFF40 ..= 0xFF4B    =>  self.ppu.write8(addr, data),
            // Empty but unusable for I/O
            0xFF4C ..= 0xFF7F   =>  panic!("unsupport write at {:04x}", addr),
            // Internal RAM
            0xFF80 ..= 0xFFFE   =>  self.hram.write8(addr&0x7F, data),
            // Interrupt Enable Register
            0xFFFF              =>  self.interrupt.write8(addr, data),
            _                   =>  unimplemented!("0x{:08x}", addr),
        }
    }
}