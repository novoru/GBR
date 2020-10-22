#[macro_use]
use bitflags::*;

use crate::core::io::Io;

bitflags! {
    struct If: u8 {
        const _BIT7             = 0b10000000;
        const _BIT6             = 0b01000000;
        const _BIT5             = 0b00100000;
        const TRANSITION_PIN    = 0b00010000;
        const SIO_COMPLETE      = 0b00001000;
        const TIMER_OVERFLOW    = 0b00000100;
        const LCDC              = 0b00000010;
        const VBLANK            = 0b00000001;
        const NONE              = 0b00000000;
    }
}

bitflags! {
    struct Ie: u8 {
        const _BIT7             = 0b10000000;
        const _BIT6             = 0b01000000;
        const _BIT5             = 0b00100000;
        const TRANSITION_PIN    = 0b00010000;
        const SIO_COMPLETE      = 0b00001000;
        const TIMER_OVERFLOW    = 0b00000100;
        const LCDC              = 0b00000010;
        const VBLANK            = 0b00000001;
        const NONE              = 0b00000000;
    }
}

pub struct Interrupt {
    irqf: If,
    irqe: Ie,
}

impl Interrupt {
    pub fn new() -> Self {
        Interrupt {
            irqf:   If::empty(),
            irqe:   Ie::empty(),
        }
    }

    pub fn enable(&mut self) {
        self.irqe = Ie::TRANSITION_PIN | Ie::SIO_COMPLETE | Ie::TIMER_OVERFLOW |
                    Ie::LCDC | Ie::VBLANK;
    }
    
    pub fn disable(&mut self) {
        self.irqe = Ie::empty();
    }
}

impl Io for Interrupt {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            0xFF0F    =>  self.irqf.bits() as u8,
            0xFFFF    =>  self.irqe.bits() as u8,
            _       =>  panic!("can't read from: {:04x}", addr),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            0xFF0F    =>  self.irqf = If::from_bits_truncate(data),
            0xFFFF    =>  self.irqe = Ie::from_bits_truncate(data),
            _       =>  panic!("can't write to: {:04x}", addr),
        }
    }
}