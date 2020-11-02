use bitflags::*;

bitflags!{
    struct P1: u8 {
        const P15   = 0b00100000;
        const P14   = 0b00010000;
        const P13   = 0b00001000;
        const P12   = 0b00000100;
        const P11   = 0b00000010;
        const P10   = 0b00000001;
    }
}

bitflags!{
    struct KeyState: u8 {
        const START     = 0b10000000;
        const SELECT    = 0b01000000;
        const B         = 0b00100000;
        const A         = 0b00010000;
        const DOWN      = 0b00001000;
        const UP        = 0b00000100;
        const LEFT      = 0b00000010;
        const RIGHT     = 0b00000001;
    }
}

#[derive(Debug)]
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
    state:      KeyState,
}

impl Pad {
    pub fn new() -> Self {
        Pad {
            register:   P1::P13 | P1::P12 | P1::P11 | P1::P10,
            state:      KeyState::A         | KeyState::B       |
                        KeyState::SELECT    | KeyState::START   |
                        KeyState::RIGHT     | KeyState::LEFT    |
                        KeyState::UP        | KeyState::DOWN,
        }
    }

    pub fn read8(&self) -> u8 {
        if !self.register.contains(P1::P15) {
            return self.register.bits() & 0xF0 | (self.state.bits() >> 4) & 0x0F;
        }

        if !self.register.contains(P1::P14) {
            return self.register.bits() & 0xF0 | self.state.bits() & 0x0F;
        }

        self.register.bits() & 0x0F
    }

    pub fn write8(&mut self, data: u8) {
        self.register = P1::from_bits_truncate(data);
    }

    pub fn push_key(&mut self, key: Key) {
        match key {
            Key::Right  =>  self.state.remove(KeyState::RIGHT),
            Key::A      =>  self.state.remove(KeyState::A),
            Key::Left   =>  self.state.remove(KeyState::LEFT),
            Key::B      =>  self.state.remove(KeyState::B),
            Key::Up     =>  self.state.remove(KeyState::UP),
            Key::Select =>  self.state.remove(KeyState::SELECT),
            Key::Down   =>  self.state.remove(KeyState::DOWN),
            Key::Start  =>  self.state.remove(KeyState::START),
        }
    }
    
    pub fn release_key(&mut self, key: Key) {
        match key {
            Key::Right  =>  self.state.insert(KeyState::RIGHT),
            Key::A      =>  self.state.insert(KeyState::A),
            Key::Left   =>  self.state.insert(KeyState::LEFT),
            Key::B      =>  self.state.insert(KeyState::B),
            Key::Up     =>  self.state.insert(KeyState::UP),
            Key::Select =>  self.state.insert(KeyState::SELECT),
            Key::Down   =>  self.state.insert(KeyState::DOWN),
            Key::Start  =>  self.state.insert(KeyState::START),
        }
    }

}