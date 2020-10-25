#[macro_use]
use bitflags::*;

use crate::core::io::Io;

bitflags! {
    struct If: u8 {
        const _BIT7     = 0b10000000;
        const _BIT6     = 0b01000000;
        const _BIT5     = 0b00100000;
        const JOYPAD    = 0b00010000;
        const SERIAL    = 0b00001000;
        const TIMER     = 0b00000100;
        const LCDC      = 0b00000010;
        const VBLANK    = 0b00000001;
        const NONE      = 0b00000000;
    }
}

bitflags! {
    struct Ie: u8 {
        const _BIT7     = 0b10000000;
        const _BIT6     = 0b01000000;
        const _BIT5     = 0b00100000;
        const JOYPAD    = 0b00010000;
        const SERIAL    = 0b00001000;
        const TIMER     = 0b00000100;
        const LCDC      = 0b00000010;
        const VBLANK    = 0b00000001;
        const NONE      = 0b00000000;
    }
}

const VBLANK_ISR_ADDR:      usize = 0x0040;
const LCDC_STAT_ISR_ADDR:   usize = 0x0048;
const TIMER_ISR_ADDR:       usize = 0x0050;
const SERIAL_ISR_ADDR:      usize = 0x0058;
const JOYPAD_ISR_ADDR:      usize = 0x0060;

pub enum InterruptKind {
    Vblank,
    LcdcStatus,
    Timer,
    Serial,
    Joypad,
}

pub struct Interrupt {
    ime:    bool,
    irqf:   If,
    irqe:   Ie,
}

impl Interrupt {
    pub fn new() -> Self {
        Interrupt {
            ime:    false,
            irqf:   If::empty(),
            irqe:   Ie::empty(),
        }
    }

    pub fn enable(&mut self) {
        self.ime = true;
    }
    
    pub fn disable(&mut self) {
        self.ime = false;
    }

    pub fn has_irq(&self) -> bool {
        self.irqf.bits() != 0x00
    }

    pub fn is_enabled_irq(&self) -> bool {
        self.ime
    }

    fn interrupt_kind(&self) -> Option<InterruptKind> {
        if !self.ime {
            return None;
        }

        if  self.irqe.contains(Ie::VBLANK) &&
            self.irqf.contains(If::VBLANK) {
                return Some(InterruptKind::Vblank);
        }
        if  self.irqe.contains(Ie::LCDC) &&
            self.irqf.contains(If::LCDC) {
                return Some(InterruptKind::LcdcStatus);
        }
        if  self.irqe.contains(Ie::TIMER) &&
            self.irqf.contains(If::TIMER) {
                return Some(InterruptKind::Timer);
        }
        if  self.irqe.contains(Ie::SERIAL) &&
            self.irqf.contains(If::SERIAL) {
                return Some(InterruptKind::Serial);
        }
        if  self.irqe.contains(Ie::JOYPAD) &&
            self.irqf.contains(If::JOYPAD) {
                return Some(InterruptKind::Joypad);
        }

        None
    }

    pub fn isr_addr(&self) -> Option<usize> {
        let kind = self.interrupt_kind()?;
        match kind {
            InterruptKind::Vblank       =>  Some(VBLANK_ISR_ADDR),
            InterruptKind::LcdcStatus   =>  Some(LCDC_STAT_ISR_ADDR),
            InterruptKind::Timer        =>  Some(TIMER_ISR_ADDR),
            InterruptKind::Serial       =>  Some(SERIAL_ISR_ADDR),
            InterruptKind::Joypad       =>  Some(JOYPAD_ISR_ADDR),
        }
    }

    pub fn set_irq(&mut self, kind: InterruptKind) {
        match kind {
            InterruptKind::Vblank       =>  self.irqf.insert(If::VBLANK),
            InterruptKind::LcdcStatus   =>  self.irqf.insert(If::LCDC),
            InterruptKind::Timer        =>  self.irqf.insert(If::TIMER),
            InterruptKind::Serial       =>  self.irqf.insert(If::SERIAL),
            InterruptKind::Joypad       =>  self.irqf.insert(If::JOYPAD),
        }
    }

    pub fn remove_irq(&mut self, kind: InterruptKind) {
        match kind {
            InterruptKind::Vblank       =>  self.irqf.remove(If::VBLANK),
            InterruptKind::LcdcStatus   =>  self.irqf.remove(If::LCDC),
            InterruptKind::Timer        =>  self.irqf.remove(If::TIMER),
            InterruptKind::Serial       =>  self.irqf.remove(If::SERIAL),
            InterruptKind::Joypad       =>  self.irqf.remove(If::JOYPAD),
        }
    }

    pub fn is_set(&self, kind: InterruptKind) -> bool {
        match kind {
            InterruptKind::Vblank       =>  self.irqf.contains(If::VBLANK),
            InterruptKind::LcdcStatus   =>  self.irqf.contains(If::LCDC),
            InterruptKind::Timer        =>  self.irqf.contains(If::TIMER),
            InterruptKind::Serial       =>  self.irqf.contains(If::SERIAL),
            InterruptKind::Joypad       =>  self.irqf.contains(If::JOYPAD),
        }
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