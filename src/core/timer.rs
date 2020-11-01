#[macro_use]
use bitflags::*;

use crate::core::io::Io;

bitflags! {
    struct Tac: u8 {
        const TIMER_EN  = 0b00000100;
        const CLK_SEL1  = 0b00000010;
        const CLK_SEL0  = 0b00000001;
    }
}

const TAC00_DIV: u16    = 1024;
const TAC01_DIV: u16    = 16;
const TAC10_DIV: u16    = 64;
const TAC11_DIV: u16    = 256;
const DIV: u16          = 256;

#[derive(Debug)]
pub struct Timer {
    div:    u8,
    tima:   u8,
    tma:    u8,
    tac:    Tac,
    count:  u16
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div:    0,
            tima:   0,
            tma:    0,
            tac:    Tac::empty(),
            count:  0,
        }
    }

    pub fn tick(&mut self) -> bool {
        let mut overflow = false;
        self.count = self.count.wrapping_add(1);
        if self.tac.contains(Tac::TIMER_EN) {
            match self.tac.bits() & 0b11 {
                0b00    =>  {
                    if self.count % TAC00_DIV == 0 {
                        self.tima = self.tima.wrapping_add(1);
                        if self.tima == 0 {
                            self.tima = self.tma;
                            overflow = true;
                        }
                    }
                },
                0b10    =>  {
                    if self.count % TAC01_DIV == 0 {
                        self.tima = self.tima.wrapping_add(1);
                        if self.tima == 0 {
                            self.tima = self.tma;
                            overflow = true;
                        }
                    }
                },
                0b01    =>  {
                    if self.count % TAC10_DIV == 0 {
                        self.tima = self.tima.wrapping_add(1);
                        if self.tima == 0 {
                            self.tima = self.tma;
                            overflow = true;
                        }
                    }
                },
                0b11    =>  {
                    if self.count % TAC11_DIV == 0 {
                        self.tima = self.tima.wrapping_add(1);
                        if self.tima == 0 {
                            self.tima = self.tma;
                            overflow = true;
                        }
                    }
                },
                _       =>  panic!(),
            }
        }
        if self.count % DIV == 0 { self.div = self.div.wrapping_add(1); }

        overflow
    }
    
}

impl Io for Timer {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            0xFF04  =>  self.div,
            0xFF05  =>  self.tima,
            0xFF06  =>  self.tma,
            0xFF07  =>  self.tac.bits(),
            _       =>  panic!("can't read from: {:04x}", addr),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            0xFF04  =>  self.div    = 0,
            0xFF05  =>  self.tima   = data,
            0xFF06  =>  self.tma    = data,
            0xFF07  =>  self.tac    = Tac::from_bits_truncate(data),
            _       =>  panic!("can't write to: {:04x}", addr),
        }
    }
}