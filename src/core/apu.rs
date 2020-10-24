use crate::core::io::Io;

pub struct Apu {

}

impl Apu {
    pub fn new() -> Self {
        Apu {
            
        }
    }
}

impl Io for Apu {
    fn read8(&self, _addr: usize) -> u8 {
        0
    }

    fn write8(&mut self, _addr: usize, _data: u8) {
        
    }
}