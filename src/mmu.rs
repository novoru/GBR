use crate::io::Io;
use crate::ram::Ram;
use crate::cartridge::Cartridge;

pub struct Mmu {
    cartridge:  Cartridge,
    ram:        Ram,
}

impl Mmu {
    pub fn from_cartridge(cartridge: Cartridge) -> Self {
        Mmu {
            cartridge:  cartridge,
            ram:        Ram::new(),
        }
    }
}

impl Io for Mmu {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            // 16kB ROM bank #0
            0x0000 ..= 0x3FFF   =>  self.cartridge.read8(addr),
            // 16kB switchable ROM bank
            0x4000 ..= 0x7FFF   =>  self.cartridge.read8(addr),
            // 8kB Video RAM
            // 0x8000 ..= 0x9FFF   =>  self.vram.read8(addr),
            // 8kB switchable RAM ban
            0xA000 ..= 0xBFFF   =>  self.cartridge.read8(addr),
            // 8kB Internal RAM
            0xC000 ..= 0xDFFF   =>  self.ram.read8(addr),
            // Echo of 8kB Internal RAM
            0xE000 ..= 0xFDFF   =>  self.ram.read8(addr),
            // Sprite Attribute Memory (OAM)
            // 0xFE00 ..= 0xFE9F   =>  self.oam.read8(addr),
            // Empty but unusable for I/O
            0xFEA0 ..= 0xFEFF   =>  panic!("unsupport read at {:04x}", addr),
            // I/O ports
            // 0xFF00 ..= 0xFF3B   =>  self.ioports.read8(addr),
            // Empty but unusable for I/O
            0xFF4C ..= 0xFF7F   =>  panic!("unsupport read at {:04x}", addr),
            // Internal RAM
            // 0xFF80 ..= 0xFFFE   =>  self.ram.read8(addr),
            // Interrupt Enable Register
            // 0xFFFF              =>  self.ier.read8(addr),
            _                   =>  unimplemented!(),
        }
    }

    fn read16(&self, addr: usize) -> u16 {
        match addr {
            // 16kB ROM bank #0
            0x0000 ..= 0x3FFF   =>  self.cartridge.read16(addr),
            // 16kB switchable ROM bank
            0x4000 ..= 0x7FFF   =>  self.cartridge.read16(addr),
            // 8kB Video RAM
            // 0x8000 ..= 0x9FFF   =>  self.vram.read16(addr),
            // 8kB switchable RAM ban
            0xA000 ..= 0xBFFF   =>  self.cartridge.read16(addr),
            // 8kB Internal RAM
            0xC000 ..= 0xDFFF   =>  self.ram.read16(addr),
            // Echo of 8kB Internal RAM
            0xE000 ..= 0xFDFF   =>  self.ram.read16(addr),
            // Sprite Attribute Memory (OAM)
            // 0xFE00 ..= 0xFE9F   =>  self.oam.read16(addr),
            // Empty but unusable for I/O
            0xFEA0 ..= 0xFEFF   =>  panic!("unsupport read at {:04x}", addr),
            // I/O ports
            // 0xFF00 ..= 0xFF3B   =>  self.ioports.read16(addr),
            // Empty but unusable for I/O
            0xFF4C ..= 0xFF7F   =>  panic!("unsupport read at {:04x}", addr),
            // Internal RAM
            // 0xFF80 ..= 0xFFFE   =>  self.ram.read16(addr),
            // Interrupt Enable Register
            // 0xFFFF              =>  self.ier.read16(addr),
            _                   =>  unimplemented!(),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            // 16kB ROM bank #0
            0x0000 ..= 0x3FFF   =>  self.cartridge.write8(addr, data),
            // 16kB switchable ROM bank
            0x4000 ..= 0x7FFF   =>  self.cartridge.write8(addr, data),
            // 8kB Video RAM
            // 0x8000 ..= 0x9FFF   =>  self.vram.read8(addr),
            // 8kB switchable RAM ban
            0xA000 ..= 0xBFFF   =>  self.cartridge.write8(addr, data),
            // 8kB Internal RAM
            0xC000 ..= 0xDFFF   =>  self.ram.write8(addr, data),
            // Echo of 8kB Internal RAM
            0xE000 ..= 0xFDFF   =>  self.ram.write8(addr, data),
            // Sprite Attribute Memory (OAM)
            // 0xFE00 ..= 0xFE9F   =>  self.oam.write8(addr),
            // Empty but unusable for I/O
            0xFEA0 ..= 0xFEFF   =>  panic!("unsupport write at {:04x}", addr),
            // I/O ports
            // 0xFF00 ..= 0xFF3B   =>  self.ioports.write8(addr),
            // Empty but unusable for I/O
            0xFF4C ..= 0xFF7F   =>  panic!("unsupport write at {:04x}", addr),
            // Internal RAM
            // 0xFF80 ..= 0xFFFE   =>  self.ram.write8(addr),
            // Interrupt Enable Register
            // 0xFFFF              =>  self.ier.write8(addr),
            _                   =>  unimplemented!(),
        }
    }

    fn write16(&mut self, addr: usize, data: u16) {
        match addr {
            // 16kB ROM bank #0
            0x0000 ..= 0x3FFF   =>  self.cartridge.write16(addr, data),
            // 16kB switchable ROM bank
            0x4000 ..= 0x7FFF   =>  self.cartridge.write16(addr, data),
            // 8kB Video RAM
            // 0x8000 ..= 0x9FFF   =>  self.vram.read8(addr),
            // 8kB switchable RAM ban
            0xA000 ..= 0xBFFF   =>  self.cartridge.write16(addr, data),
            // 8kB Internal RAM
            0xC000 ..= 0xDFFF   =>  self.ram.write16(addr, data),
            // Echo of 8kB Internal RAM
            0xE000 ..= 0xFDFF   =>  self.ram.write16(addr, data),
            // Sprite Attribute Memory (OAM)
            // 0xFE00 ..= 0xFE9F   =>  self.oam.write16(addr),
            // Empty but unusable for I/O
            0xFEA0 ..= 0xFEFF   =>  panic!("unsupport write at {:04x}", addr),
            // I/O ports
            // 0xFF00 ..= 0xFF3B   =>  self.ioports.write16(addr),
            // Empty but unusable for I/O
            0xFF4C ..= 0xFF7F   =>  panic!("unsupport write at {:04x}", addr),
            // Internal RAM
            // 0xFF80 ..= 0xFFFE   =>  self.ram.write16(addr),
            // Interrupt Enable Register
            // 0xFFFF              =>  self.ier.write16(addr),
            _                   =>  unimplemented!(),
        }
    }
}