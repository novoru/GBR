#[macro_use]
use bitflags::*;
use std::fmt;

use crate::io::Io;
use crate::mmu::Mmu;
use crate::cartridge::Cartridge;

bitflags! {
    struct Flags: u8 {
        const Z     = 0b10000000;
        const N     = 0b01000000;
        const H     = 0b00100000;
        const C     = 0b00010000;
        const NONE  = 0b00000000;
    }
}

impl Default for Flags {
    fn default() -> Flags {
        Flags::NONE
    }
}

pub struct Cpu {
    a:      u8,
    b:      u8,
    d:      u8,
    h:      u8,
    f:      u8,
    c:      u8,
    e:      u8,
    l:      u8,
    flags:  Flags,
    sp:     u16,
    pc:     u16,
    mmu:    Mmu,
}

impl Cpu {
    pub fn new() -> Self {
        let cartridge = Cartridge::new();
        Cpu {
            a:      0,
            b:      0,
            d:      0,
            h:      0,
            f:      0,
            c:      0,
            e:      0,
            l:      0,
            flags:  Flags::empty(),
            sp:     0,
            pc:     0,
            mmu:    Mmu::from_cartridge(cartridge),
        }
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        let inst = self.decode(opcode);
        self.execute(&inst);
    }

    fn fetch(&mut self) -> u8 {
        let value = self.mmu.read8(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch16(&mut self) -> u16 {
        let value = self.mmu.read16(self.pc as usize);
        self.pc = self.pc.wrapping_add(2);
        value
    }

    fn decode(&mut self, opcode: u8) -> Instruction {
        match opcode {
            0x06    =>  Instruction {
                name:       "LD B, n",
                opcode:     0x06,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.b += n;
                    Ok(())
                },
            },
            0x0E    =>  Instruction {
                name:       "LD C, n",
                opcode:     0x0E,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.c += n;
                    Ok(())
                },
            },
            0x16    =>  Instruction {
                name:       "LD D, n",
                opcode:     0x16,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.d += n;
                    Ok(())
                },
            },
            0x1E    =>  Instruction {
                name:       "LD E, n",
                opcode:     0x1E,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.e += n;
                    Ok(())
                },
            },
            0x26    =>  Instruction {
                name:       "LD H, n",
                opcode:     0x26,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.h += n;
                    Ok(())
                },
            },
            0x2E    =>  Instruction {
                name:       "LD L, n",
                opcode:     0x2E,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.l += n;
                    Ok(())
                },
            },
            _       =>  panic!("can't decode: 0x{:02x}", opcode),
        }
    }

    fn execute(&mut self, inst: &Instruction) {
        (inst.operation)(self).unwrap();
    }
}

struct Instruction {
    name:       &'static str,
    opcode:     u8,
    operation:  fn(cpu: &mut Cpu) -> Result<(), ()>,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instruction {{ name='{}', opcode=0x{:02x} }}", self.name, self.opcode)
    }
}

#[test]
fn test_ldbn() {    
    let mut cpu = Cpu::new();
    let opcode = 0x06;

    cpu.mmu.write8(0x00, opcode);
    cpu.mmu.write8(0x01, 42);
    cpu.tick();
    
    assert_eq!(cpu.b, 42);
    assert_eq!(format!("{}", cpu.decode(opcode)), "Instruction { name='LD B, n', opcode=0x06 }")
}