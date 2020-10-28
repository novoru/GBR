#[macro_use]
use bitflags::*;

use std::fmt;

bitflags!{
    struct P1: u8 {
        const _BIT7 = 0b10000000;
        const _BIT6 = 0b01000000;
        const P15   = 0b00100000;
        const P14   = 0b00010000;
        const P13   = 0b00001000;
        const P12   = 0b00000100;
        const P11   = 0b00000010;
        const P10   = 0b00000001;
        const NONE  = 0b00000000;
    }
}

pub enum Key {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

pub struct Pad {
    register:   P1,
}

impl fmt::Display for Pad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pad {{ 0b{:08b} }}", self.register)
    }
}

impl Pad {
    pub fn new() -> Self {
        Pad {
            register:   P1::P15 | P1::P14 | P1::P13 | P1::P12 | P1::P11 | P1::P10,
        }
    }

    pub fn read8(&self) -> u8 {
        self.register.bits()
    }

    pub fn write8(&mut self, data: u8) {
        self.register = P1::from_bits_truncate(data);
    }

    fn _update(&mut self) {
        if self.register.contains(P1::P14) | self.register.contains(P1::P15) {
            self.register.insert(P1::P10 | P1::P11 | P1::P12 | P1::P13);
        }
    }

    pub fn push_key(&mut self, key: Key) {
        match key {
            Key::Right | Key::A       =>  self.register.remove(P1::P10),
            Key::Left  | Key::B       =>  self.register.remove(P1::P11),
            Key::Up    | Key::Select  =>  self.register.remove(P1::P12),
            Key::Down  | Key::Start   =>  self.register.remove(P1::P13),
        }
        // self.update();
    }
    
    pub fn release_key(&mut self, key: Key) {
        match key {
            Key::Right | Key::A       =>  self.register.insert(P1::P10),
            Key::Left  | Key::B       =>  self.register.insert(P1::P11),
            Key::Up    | Key::Select  =>  self.register.insert(P1::P12),
            Key::Down  | Key::Start   =>  self.register.insert(P1::P13),
        }
        // self.update();
    }

}