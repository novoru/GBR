#[macro_use]
use bitflags::*;
use std::fmt;
use std::path::Path;

use crate::core::io::Io;
use crate::core::bus::Bus;
use crate::core::pad::Key;
use crate::core::ppu::*;

bitflags! {
    struct Flags: u8 {
        const Z     = 0b10000000;
        const N     = 0b01000000;
        const H     = 0b00100000;
        const C     = 0b00010000;
        const _BIT3 = 0b00001000;
        const _BIT2 = 0b00000100;
        const _BIT1 = 0b00000010;
        const _BIT0 = 0b00000001;
        const NONE  = 0b00000000;
    }
}

pub struct Cpu {
    a:      u8,
    b:      u8,
    d:      u8,
    h:      u8,
    c:      u8,
    e:      u8,
    l:      u8,
    f:      Flags,
    sp:     u16,
    pc:     u16,
    bus:    Bus,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cpu {{\n\ta = 0x{:02x}\n\tb = 0x{:02x}\n\td = 0x{:02x}\n\th = 0x{:02x}\n\
                           \tc = 0x{:02x}\n\te = 0x{:02x}\n\tl = 0x{:02x}\n\tf = 0x{:02x}\n\
                           \tsp= 0x{:04x}\n\tpc= 0x{:04x}\n}}",
            self.a, self.b, self.d, self.h,
            self.c, self.e, self.l, self.f,
            self.sp, self.pc)
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a:      0,
            b:      0,
            d:      0,
            h:      0,
            c:      0,
            e:      0,
            l:      0,
            f:      Flags::empty(),
            sp:     0xFFFE,
            pc:     0x100,
            bus:    Bus::no_cartridge(),
        }
    }
    
    pub fn from_path(path: &Path) -> Self {
        Cpu {
            a:      0,
            b:      0,
            d:      0,
            h:      0,
            c:      0,
            e:      0,
            l:      0,
            f:      Flags::empty(),
            sp:     0xFFFE,
            pc:     0x100,
            bus:    Bus::from_path(path),
        }
    }

    pub fn tick(&mut self) {
        if !self.bus.transfer() {
            let opcode = self.fetch();
            let inst = self.decode(opcode);
            self.execute(&inst);
        }
        self.bus.tick();
    }

    pub fn key_push(&mut self, key: Key) {
        self.bus.key_push(key);
    }

    pub fn key_release(&mut self, key: Key) {
        self.bus.key_release(key);
    }

    pub fn get_pixels(&self) -> [u8; SCREEN_WIDTH*SCREEN_HEIGHT] {
        self.bus.get_pixels()
    }

    fn fetch(&mut self) -> u8 {
        let value = self.bus.read8(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch16(&mut self) -> u16 {
        let lo = self.fetch();
        let hi = self.fetch();
        ((hi as i16) << 8) as u16 | lo as u16
    }

    fn read_af(&self) -> u16 {
        ((self.a as i16) << 8) as u16 | self.f.bits as u16
    }

    fn write_af(&mut self, data: u16) {
        self.a = (data >> 8) as u8;
        self.f = Flags::from_bits_truncate((data & 0xFF) as u8);
    }

    fn read_bc(&self) -> u16 {
        ((self.b as i16) << 8) as u16 | self.c as u16
    }
    
    fn write_bc(&mut self, data: u16) {
        self.b = (data >> 8) as u8;
        self.c = (data & 0xFF) as u8;
    }
    
    fn read_de(&self) -> u16 {
        ((self.d as i16) << 8) as u16 | self.e as u16
    }
    
    fn write_de(&mut self, data: u16) {
        self.d = (data >> 8) as u8;
        self.e = (data & 0xFF) as u8;
    }
    
    fn read_hl(&self) -> u16 {
        ((self.h as i16) << 8) as u16 | self.l as u16
    }
    
    fn write_hl(&mut self, data: u16) {
        self.h = (data >> 8) as u8;
        self.l = (data & 0xFF) as u8;
    }

    fn push(&mut self, data: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write8(self.sp as usize, data);
    }

    fn pop(&mut self) -> u8 {
        let addr = self.sp;
        self.sp = addr.wrapping_add(1);
        self.bus.read8(addr as usize)
    }

    fn decode(&mut self, opcode: u8) -> Instruction {
        match opcode {
            0x00    =>  Instruction {
                name:       "NOP",
                opcode:     0x00,
                cycles:     4,
                operation:  |_| {
                    Ok(())
                },
            },
            0x01    =>  Instruction {
                name:       "LD BC, nn",
                opcode:     0x01,
                cycles:     12,
                operation:  |cpu| {
                    let nn = cpu.fetch16();
                    cpu.write_bc(nn);
                    Ok(())
                },
            },
            0x02    =>  Instruction {
                name:       "LD (BC), A",
                opcode:     0x02,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_bc() as usize;
                    cpu.bus.write8(addr, cpu.a);
                    Ok(())
                },
            },
            0x03    =>  Instruction {
                name:       "INC BC",
                opcode:     0x03,
                cycles:     8,
                operation:  |cpu| {
                    let bc = cpu.read_bc();
                    cpu.write_bc(bc.wrapping_add(1));
                    Ok(())
                },
            },
            0x04    =>  Instruction {
                name:       "INC B",
                opcode:     0x04,
                cycles:     4,
                operation:  |cpu| {
                    let b = cpu.b;
                    cpu.b = b.wrapping_add(1);
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.b^b^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x05    =>  Instruction {
                name:       "DEC B",
                opcode:     0x05,
                cycles:     4,
                operation:  |cpu| {
                    let b = cpu.b;
                    cpu.b = b.wrapping_sub(1);
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.b^b^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x06    =>  Instruction {
                name:       "LD B, n",
                opcode:     0x06,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.b = n;
                    Ok(())
                },
            },
            0x07    =>  Instruction {
                name:       "RLCA",
                opcode:     0x07,
                cycles:     4,
                operation:  |cpu| {
                    let carry = cpu.a & 0x80 == 0x80;
                    cpu.a = cpu.a.rotate_left(1);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },            
            0x08    =>  Instruction {
                name:       "LD (nn), SP",
                opcode:     0x08,
                cycles:     20,
                operation:  |cpu| {
                    let addr = cpu.fetch16() as usize;
                    cpu.bus.write8(addr, (cpu.sp&0xFF) as u8);
                    cpu.bus.write8(addr+1, (cpu.sp >> 8) as u8);
                    Ok(())
                },
            },
            0x09    =>  Instruction {
                name:       "ADD HL, BC",
                opcode:     0x09,
                cycles:     8,
                operation:  |cpu| {
                    let hl = cpu.read_hl();
                    let bc = cpu.read_bc();
                    cpu.write_hl(hl.wrapping_add(bc));
                    cpu.f.remove(Flags::N);
                    if (cpu.read_hl()^hl^bc)&0x0100 == 0x0100 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.read_hl() < hl {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x0A    =>  Instruction {
                name:       "LD A, (BC)",
                opcode:     0x0A,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a = cpu.bus.read8(cpu.read_bc() as usize);
                    Ok(())
                },
            },
            0x0B    =>  Instruction {
                name:       "DEC BC",
                opcode:     0x0B,
                cycles:     8,
                operation:  |cpu| {
                    let bc = cpu.read_bc();
                    cpu.write_bc(bc.wrapping_sub(1));
                    Ok(())
                },
            },
            0x0C    =>  Instruction {
                name:       "INC C",
                opcode:     0x0C,
                cycles:     4,
                operation:  |cpu| {
                    let c = cpu.c;
                    cpu.c = c.wrapping_add(1);
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.c^c^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x0D    =>  Instruction {
                name:       "DEC C",
                opcode:     0x0D,
                cycles:     4,
                operation:  |cpu| {
                    let c = cpu.c;
                    cpu.c = c.wrapping_sub(1);
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.c^c^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x0E    =>  Instruction {
                name:       "LD C, n",
                opcode:     0x0E,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.c = n;
                    Ok(())
                },
            },
            0x0F    =>  Instruction {
                name:       "RRCA",
                opcode:     0x0F,
                cycles:     4,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a.rotate_right(1);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },                 
            0x10    =>  Instruction {
                name:       "STOP",
                opcode:     0x10,
                cycles:     4,
                operation:  |_| {
                    // TODO
                    Ok(())
                },
            },
            0x11    =>  Instruction {
                name:       "LD DE, nn",
                opcode:     0x11,
                cycles:     12,
                operation:  |cpu| {
                    let nn = cpu.fetch16();
                    cpu.write_de(nn);
                    Ok(())
                },
            },
            0x12    =>  Instruction {
                name:       "LD (DE), A",
                opcode:     0x02,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_de() as usize;
                    cpu.bus.write8(addr, cpu.a);
                    Ok(())
                },
            },
            0x13    =>  Instruction {
                name:       "INC DE",
                opcode:     0x13,
                cycles:     8,
                operation:  |cpu| {
                    let de = cpu.read_de();
                    cpu.write_de(de.wrapping_add(1));
                    Ok(())
                },
            },            
            0x14    =>  Instruction {
                name:       "INC D",
                opcode:     0x14,
                cycles:     4,
                operation:  |cpu| {
                    let d = cpu.d;
                    cpu.d = d.wrapping_add(1);
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.d^d^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x15    =>  Instruction {
                name:       "DEC D",
                opcode:     0x15,
                cycles:     4,
                operation:  |cpu| {
                    let d = cpu.d;
                    cpu.d = d.wrapping_sub(1);
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.d^d^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x16    =>  Instruction {
                name:       "LD D, n",
                opcode:     0x16,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.d = n;
                    Ok(())
                },
            },
            0x17    =>  Instruction {
                name:       "RLA",
                opcode:     0x17,
                cycles:     4,
                operation:  |cpu| {
                    let carry = cpu.a & 0x80 == 0x80;
                    cpu.a = cpu.a << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.a |= 1;
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x18    =>  Instruction {
                name:       "JR e",
                opcode:     0x18,
                cycles:     8,
                operation:  |cpu| {
                    let e = cpu.fetch() as i8 as i16;
                    cpu.pc = (cpu.pc as i16 + e) as u16;
                    Ok(())
                },
            },
            0x19    =>  Instruction {
                name:       "ADD HL, DE",
                opcode:     0x19,
                cycles:     8,
                operation:  |cpu| {
                    let hl = cpu.read_hl();
                    let de = cpu.read_de();
                    cpu.write_hl(hl.wrapping_add(de));
                    cpu.f.remove(Flags::N);
                    if (cpu.read_hl()^hl^de)&0x0100 == 0x0100 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.read_hl() < hl {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x1A    =>  Instruction {
                name:       "LD A, (DE)",
                opcode:     0x1A,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a = cpu.bus.read8(cpu.read_de() as usize);
                    Ok(())
                },
            },
            0x1B    =>  Instruction {
                name:       "DEC DE",
                opcode:     0x1B,
                cycles:     8,
                operation:  |cpu| {
                    let de = cpu.read_de();
                    cpu.write_de(de.wrapping_sub(1));
                    Ok(())
                },
            },
            0x1C    =>  Instruction {
                name:       "INC E",
                opcode:     0x1C,
                cycles:     4,
                operation:  |cpu| {
                    let e = cpu.e;
                    cpu.e = e.wrapping_add(1);
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.e^e^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x1D    =>  Instruction {
                name:       "DEC E",
                opcode:     0x1D,
                cycles:     4,
                operation:  |cpu| {
                    let e = cpu.e;
                    cpu.e = e.wrapping_sub(1);
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.e^e^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x1E    =>  Instruction {
                name:       "LD E, n",
                opcode:     0x1E,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.e = n;
                    Ok(())
                },
            },
            0x1F    =>  Instruction {
                name:       "RRA",
                opcode:     0x01F,
                cycles:     4,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.a |= 0x80;
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x20    =>  Instruction {
                name:       "JR NZ, e",
                opcode:     0x20,
                cycles:     8,
                operation:  |cpu| {
                    let e = cpu.fetch() as i8 as i16;
                    if cpu.f & Flags::Z != Flags::Z {
                        cpu.pc = (cpu.pc as i16 + e) as u16;
                    }
                    Ok(())
                },
            },
            0x21    =>  Instruction {
                name:       "LD HL, nn",
                opcode:     0x21,
                cycles:     12,
                operation:  |cpu| {
                    let nn = cpu.fetch16();
                    cpu.write_hl(nn);
                    Ok(())
                },
            },
            0x22    =>  Instruction {
                name:       "LDI (HL), A",
                opcode:     0x22,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl();
                    cpu.write_hl(addr.wrapping_add(1));
                    cpu.bus.write8(addr as usize, cpu.a);
                    Ok(())
                },
            },
            0x23    =>  Instruction {
                name:       "INC HL",
                opcode:     0x23,
                cycles:     8,
                operation:  |cpu| {
                    let hl = cpu.read_hl();
                    cpu.write_hl(hl.wrapping_add(1));
                    Ok(())
                },
            },            
            0x24    =>  Instruction {
                name:       "INC H",
                opcode:     0x24,
                cycles:     4,
                operation:  |cpu| {
                    let h = cpu.h;
                    cpu.h = h.wrapping_add(1);
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.h^h^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x25    =>  Instruction {
                name:       "DEC H",
                opcode:     0x25,
                cycles:     4,
                operation:  |cpu| {
                    let h = cpu.h;
                    cpu.h = h.wrapping_sub(1);
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.h^h^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x26    =>  Instruction {
                name:       "LD H, n",
                opcode:     0x26,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.h = n;
                    Ok(())
                },
            },
            0x27    =>  Instruction {
                name:       "DAA",
                opcode:     0x27,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    if cpu.f & Flags::N == Flags::N {
                        if cpu.f & Flags::H == Flags::H || a&0x0F > 0x09 {
                            cpu.a = a.wrapping_add(0x06);
                        }
                        if cpu.f & Flags::C == Flags::H || a > 0x9F {
                            cpu.a = a.wrapping_add(0x60);
                        }
                    } else {
                        if cpu.f & Flags::H == Flags::H {
                            cpu.a = a.wrapping_sub(0x06);
                        }
                        if cpu.f & Flags::C == Flags::C {
                            cpu.a = a.wrapping_sub(0x60);
                        }
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::H);
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x28    =>  Instruction {
                name:       "JR Z, e",
                opcode:     0x28,
                cycles:     8,
                operation:  |cpu| {
                    let e = cpu.fetch() as i8 as i16;
                    if cpu.f & Flags::Z == Flags::Z {
                        cpu.pc = (cpu.pc as i16 + e) as u16;
                    }
                    Ok(())
                },
            },            
            0x29    =>  Instruction {
                name:       "ADD HL, HL",
                opcode:     0x29,
                cycles:     8,
                operation:  |cpu| {
                    let hl1 = cpu.read_hl();
                    let hl2 = cpu.read_hl();
                    cpu.write_hl(hl1.wrapping_add(hl2));
                    cpu.f.remove(Flags::N);
                    if (cpu.read_hl()^hl1^hl2)&0x0100 == 0x0100 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.read_hl() < hl1 {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2A    =>  Instruction {
                name:       "LDI A, (HL)",
                opcode:     0x2A,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl();
                    cpu.write_hl(addr.wrapping_add(1));
                    cpu.a = cpu.bus.read8(addr as usize);
                    Ok(())
                },
            },
            0x2B    =>  Instruction {
                name:       "DEC HL",
                opcode:     0x2B,
                cycles:     8,
                operation:  |cpu| {
                    let hl = cpu.read_hl();
                    cpu.write_hl(hl.wrapping_sub(1));
                    Ok(())
                },
            },            
            0x2C    =>  Instruction {
                name:       "INC L",
                opcode:     0x2C,
                cycles:     4,
                operation:  |cpu| {
                    let l = cpu.l;
                    cpu.l = l.wrapping_add(1);
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.l^l^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x2D    =>  Instruction {
                name:       "DEC L",
                opcode:     0x2D,
                cycles:     4,
                operation:  |cpu| {
                    let l = cpu.l;
                    cpu.l = l.wrapping_sub(1);
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.l^l^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },            
            0x2E    =>  Instruction {
                name:       "LD L, n",
                opcode:     0x2E,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.l = n;
                    Ok(())
                },
            },
            0x2F    =>  Instruction {
                name:       "CPL",
                opcode:     0x2F,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = !cpu.a;
                    cpu.f.insert(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x30    =>  Instruction {
                name:       "JR NC, e",
                opcode:     0x30,
                cycles:     8,
                operation:  |cpu| {
                    let e = cpu.fetch() as i8 as i16;
                    if cpu.f & Flags::C != Flags::C {
                        cpu.pc = (cpu.pc as i16 + e) as u16;
                    }
                    Ok(())
                },
            },            
            0x31    =>  Instruction {
                name:       "LD SP, nn",
                opcode:     0x31,
                cycles:     12,
                operation:  |cpu| {
                    let nn = cpu.fetch16();
                    cpu.sp = nn;
                    Ok(())
                },
            },
            0x32    =>  Instruction {
                name:       "LDD (HL), A",
                opcode:     0x32,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl();
                    cpu.write_hl(addr.wrapping_sub(1));
                    cpu.bus.write8(addr as usize, cpu.a);
                    Ok(())
                },
            },
            0x33    =>  Instruction {
                name:       "INC SP",
                opcode:     0x33,
                cycles:     8,
                operation:  |cpu| {
                    cpu.sp = cpu.sp.wrapping_add(1);
                    Ok(())
                },
            },            
            0x34    =>  Instruction {
                name:       "INC (HL)",
                opcode:     0x34,
                cycles:     12,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let n = cpu.bus.read8(addr);
                    cpu.bus.write8(addr, n.wrapping_add(1));
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.bus.read8(addr)^n^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x35    =>  Instruction {
                name:       "DEC (HL)",
                opcode:     0x35,
                cycles:     12,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let n = cpu.bus.read8(addr);
                    cpu.bus.write8(addr, n.wrapping_sub(1));
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.bus.read8(addr)^n^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x36    =>  Instruction {
                name:       "LD (HL), n",
                opcode:     0x36,
                cycles:     12,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.bus.write8(cpu.read_hl() as usize, n);
                    Ok(())
                },
            },
            0x37    =>  Instruction {
                name:       "SCF",
                opcode:     0x37,
                cycles:     4,
                operation:  |cpu| {
                    cpu.f.insert(Flags::C);
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    Ok(())
                },
            },
            0x38    =>  Instruction {
                name:       "JR C, e",
                opcode:     0x38,
                cycles:     8,
                operation:  |cpu| {
                    let e = cpu.fetch() as i8 as i16;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.pc = (cpu.pc as i16 + e) as u16;
                    }
                    Ok(())
                },
            },                   
            0x39    =>  Instruction {
                name:       "ADD HL, SP",
                opcode:     0x19,
                cycles:     8,
                operation:  |cpu| {
                    let hl = cpu.read_hl();
                    let sp = cpu.sp;
                    cpu.write_hl(hl.wrapping_add(sp));
                    cpu.f.remove(Flags::N);
                    if (cpu.read_hl()^hl^sp)&0x0100 == 0x0100 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.read_hl() < hl {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3A    =>  Instruction {
                name:       "LDD A, (HL)",
                opcode:     0x3A,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl();
                    cpu.write_hl(addr.wrapping_sub(1));
                    cpu.a = cpu.bus.read8(addr as usize);
                    Ok(())
                },
            },
            0x3B    =>  Instruction {
                name:       "DEC SP",
                opcode:     0x3B,
                cycles:     8,
                operation:  |cpu| {
                    cpu.sp = cpu.sp.wrapping_sub(1);
                    Ok(())
                },
            },            
            0x3C    =>  Instruction {
                name:       "INC A",
                opcode:     0x3C,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    cpu.a = a.wrapping_add(1);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x3D    =>  Instruction {
                name:       "DEC A",
                opcode:     0x3D,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    cpu.a = a.wrapping_sub(1);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^1)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    Ok(())
                },
            },
            0x3E    =>  Instruction {
                name:       "LD A, #",
                opcode:     0x3E,
                cycles:     8,
                operation:  |cpu| {
                    let n = cpu.fetch();
                    cpu.a = n;
                    Ok(())
                },
            },
            0x3F    =>  Instruction {
                name:       "CCF",
                opcode:     0x3F,
                cycles:     4,
                operation:  |cpu| {
                    if cpu.f & Flags::C == Flags::C {
                        cpu.f.remove(Flags::C);
                    } else {
                        cpu.f.insert(Flags::C);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    Ok(())
                },
            },
            0x40    =>  Instruction {
                name:       "LD B, B",
                opcode:     0x40,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.b;
                    Ok(())
                },
            },
            0x41    =>  Instruction {
                name:       "LD B, C",
                opcode:     0x40,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.c;
                    Ok(())
                },
            },
            0x42    =>  Instruction {
                name:       "LD B, D",
                opcode:     0x40,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.d;
                    Ok(())
                },
            },
            0x43    =>  Instruction {
                name:       "LD B, E",
                opcode:     0x43,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.e;
                    Ok(())
                },
            },
            0x44    =>  Instruction {
                name:       "LD B, H",
                opcode:     0x44,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.h;
                    Ok(())
                },
            },
            0x45    =>  Instruction {
                name:       "LD B, L",
                opcode:     0x45,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.l;
                    Ok(())
                },
            },
            0x46    =>  Instruction {
                name:       "LD B, (HL)",
                opcode:     0x46,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x47    =>  Instruction {
                name:       "LD B, A",
                opcode:     0x47,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.a;
                    Ok(())
                },
            },
            0x48    =>  Instruction {
                name:       "LD C, B",
                opcode:     0x48,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.b;
                    Ok(())
                },
            },
            0x49    =>  Instruction {
                name:       "LD C, C",
                opcode:     0x49,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.c;
                    Ok(())
                },
            },
            0x4A    =>  Instruction {
                name:       "LD C, D",
                opcode:     0x4A,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.d;
                    Ok(())
                },
            },
            0x4B    =>  Instruction {
                name:       "LD C, E",
                opcode:     0x4B,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.e;
                    Ok(())
                },
            },
            0x4C    =>  Instruction {
                name:       "LD C, H",
                opcode:     0x4C,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.h;
                    Ok(())
                },
            },
            0x4D    =>  Instruction {
                name:       "LD C, L",
                opcode:     0x4D,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.l;
                    Ok(())
                },
            },
            0x4E    =>  Instruction {
                name:       "LD C, (HL)",
                opcode:     0x4E,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x4F    =>  Instruction {
                name:       "LD C, A",
                opcode:     0x4F,
                cycles:     4,
                operation:  |cpu| {
                    cpu.c = cpu.a;
                    Ok(())
                },
            },
            0x50    =>  Instruction {
                name:       "LD D, B",
                opcode:     0x50,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.b;
                    Ok(())
                },
            },
            0x51    =>  Instruction {
                name:       "LD D, C",
                opcode:     0x51,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.c;
                    Ok(())
                },
            },
            0x52    =>  Instruction {
                name:       "LD D, D",
                opcode:     0x52,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.d;
                    Ok(())
                },
            },
            0x53    =>  Instruction {
                name:       "LD D, E",
                opcode:     0x53,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.e;
                    Ok(())
                },
            },
            0x54    =>  Instruction {
                name:       "LD D, H",
                opcode:     0x54,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.h;
                    Ok(())
                },
            },
            0x55    =>  Instruction {
                name:       "LD D, L",
                opcode:     0x55,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.l;
                    Ok(())
                },
            },
            0x56    =>  Instruction {
                name:       "LD D, (HL)",
                opcode:     0x56,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x57    =>  Instruction {
                name:       "LD D, A",
                opcode:     0x57,
                cycles:     4,
                operation:  |cpu| {
                    cpu.d = cpu.a;
                    Ok(())
                },
            },
            0x58    =>  Instruction {
                name:       "LD E, B",
                opcode:     0x58,
                cycles:     4,
                operation:  |cpu| {
                    cpu.e = cpu.b;
                    Ok(())
                },
            },
            0x59    =>  Instruction {
                name:       "LD E, C",
                opcode:     0x59,
                cycles:     4,
                operation:  |cpu| {
                    cpu.e = cpu.c;
                    Ok(())
                },
            },
            0x5A    =>  Instruction {
                name:       "LD E, D",
                opcode:     0x5A,
                cycles:     4,
                operation:  |cpu| {
                    cpu.e = cpu.d;
                    Ok(())
                },
            },
            0x5B    =>  Instruction {
                name:       "LD E, E",
                opcode:     0x5B,
                cycles:     4,
                operation:  |cpu| {
                    cpu.b = cpu.e;
                    Ok(())
                },
            },
            0x5C    =>  Instruction {
                name:       "LD E, H",
                opcode:     0x5C,
                cycles:     4,
                operation:  |cpu| {
                    cpu.e = cpu.h;
                    Ok(())
                },
            },
            0x5D    =>  Instruction {
                name:       "LD E, L",
                opcode:     0x5D,
                cycles:     4,
                operation:  |cpu| {
                    cpu.e = cpu.l;
                    Ok(())
                },
            },
            0x5E    =>  Instruction {
                name:       "LD E, (HL)",
                opcode:     0x5E,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x5F    =>  Instruction {
                name:       "LD E, A",
                opcode:     0x5F,
                cycles:     4,
                operation:  |cpu| {
                    cpu.e = cpu.a;
                    Ok(())
                },
            },
            0x60    =>  Instruction {
                name:       "LD H, B",
                opcode:     0x60,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.b;
                    Ok(())
                },
            },
            0x61    =>  Instruction {
                name:       "LD H, C",
                opcode:     0x61,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.c;
                    Ok(())
                },
            },
            0x62    =>  Instruction {
                name:       "LD H, D",
                opcode:     0x62,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.d;
                    Ok(())
                },
            },
            0x63    =>  Instruction {
                name:       "LD H, E",
                opcode:     0x63,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.e;
                    Ok(())
                },
            },
            0x64    =>  Instruction {
                name:       "LD H, H",
                opcode:     0x64,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.h;
                    Ok(())
                },
            },
            0x65    =>  Instruction {
                name:       "LD H, L",
                opcode:     0x65,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.l;
                    Ok(())
                },
            },
            0x66    =>  Instruction {
                name:       "LD H, (HL)",
                opcode:     0x66,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x67    =>  Instruction {
                name:       "LD H, A",
                opcode:     0x67,
                cycles:     4,
                operation:  |cpu| {
                    cpu.h = cpu.a;
                    Ok(())
                },
            },
            0x68    =>  Instruction {
                name:       "LD L, B",
                opcode:     0x68,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.b;
                    Ok(())
                },
            },
            0x69    =>  Instruction {
                name:       "LD L, C",
                opcode:     0x69,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.c;
                    Ok(())
                },
            },
            0x6A    =>  Instruction {
                name:       "LD L, D",
                opcode:     0x6A,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.d;
                    Ok(())
                },
            },
            0x6B    =>  Instruction {
                name:       "LD L, E",
                opcode:     0x6B,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.e;
                    Ok(())
                },
            },
            0x6C    =>  Instruction {
                name:       "LD L, H",
                opcode:     0x6C,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.h;
                    Ok(())
                },
            },
            0x6D    =>  Instruction {
                name:       "LD L, L",
                opcode:     0x6D,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.l;
                    Ok(())
                },
            },
            0x6E    =>  Instruction {
                name:       "LD L, (HL)",
                opcode:     0x6E,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x6F    =>  Instruction {
                name:       "LD L, A",
                opcode:     0x6F,
                cycles:     4,
                operation:  |cpu| {
                    cpu.l = cpu.a;
                    Ok(())
                },
            },
            0x70    =>  Instruction {
                name:       "LD (HL), B",
                opcode:     0x70,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.write8(cpu.read_hl() as usize, cpu.b);
                    Ok(())
                },
            },
            0x71    =>  Instruction {
                name:       "LD (HL), C",
                opcode:     0x71,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.write8(cpu.read_hl() as usize, cpu.c);                    
                    Ok(())
                },
            },
            0x72    =>  Instruction {
                name:       "LD (HL), D",
                opcode:     0x62,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.write8(cpu.read_hl() as usize, cpu.d);
                    Ok(())
                },
            },
            0x73    =>  Instruction {
                name:       "LD (HL), E",
                opcode:     0x73,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.write8(cpu.read_hl() as usize, cpu.e);
                    Ok(())
                },
            },
            0x74    =>  Instruction {
                name:       "LD (HL), H",
                opcode:     0x74,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.write8(cpu.read_hl() as usize, cpu.h);
                    Ok(())
                },
            },
            0x75    =>  Instruction {
                name:       "LD (HL), L",
                opcode:     0x75,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.write8(cpu.read_hl() as usize, cpu.l);
                    Ok(())
                },
            },
            0x76    =>  Instruction {
                name:       "HALT",
                opcode:     0x76,
                cycles:     4,
                operation:  |_| {
                    // TODO
                    Ok(())
                },
            },            
            0x77    =>  Instruction {
                name:       "LD (HL), A",
                opcode:     0x77,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.a);
                    Ok(())
                },
            },
            0x78    =>  Instruction {
                name:       "LD A, B",
                opcode:     0x78,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.b;
                    Ok(())
                },
            },
            0x79    =>  Instruction {
                name:       "LD A, C",
                opcode:     0x79,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.c;
                    Ok(())
                },
            },
            0x7A    =>  Instruction {
                name:       "LD A, D",
                opcode:     0x7A,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.d;
                    Ok(())
                },
            },
            0x7B    =>  Instruction {
                name:       "LD A, E",
                opcode:     0x7B,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.e;
                    Ok(())
                },
            },
            0x7C    =>  Instruction {
                name:       "LD A, H",
                opcode:     0x7C,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.h;
                    Ok(())
                },
            },
            0x7D    =>  Instruction {
                name:       "LD A, L",
                opcode:     0x7D,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.l;
                    Ok(())
                },
            },
            0x7E    =>  Instruction {
                name:       "LD A, (HL)",
                opcode:     0x7E,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a = cpu.bus.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            0x7F    =>  Instruction {
                name:       "LD A, A",
                opcode:     0x7F,
                cycles:     4,
                operation:  |cpu| {
                    cpu.a = cpu.a;
                    Ok(())
                },
            },
            0x80    =>  Instruction {
                name:       "ADD A, B",
                opcode:     0x80,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.b;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x81    =>  Instruction {
                name:       "ADD A, C",
                opcode:     0x81,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.c;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x82    =>  Instruction {
                name:       "ADD A, D",
                opcode:     0x82,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.d;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x83    =>  Instruction {
                name:       "ADD A, E",
                opcode:     0x83,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.e;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x84    =>  Instruction {
                name:       "ADD A, H",
                opcode:     0x84,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.h;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x85    =>  Instruction {
                name:       "ADD A, L",
                opcode:     0x85,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.l;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x86    =>  Instruction {
                name:       "ADD A, (HL)",
                opcode:     0x86,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.bus.read8(cpu.read_hl() as usize);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x87    =>  Instruction {
                name:       "ADD A, A",
                opcode:     0x87,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.a;
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x88    =>  Instruction {
                name:       "ADC A, B",
                opcode:     0x88,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.b.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x89    =>  Instruction {
                name:       "ADC A, C",
                opcode:     0x8F,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.c.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x8A    =>  Instruction {
                name:       "ADC A, D",
                opcode:     0x8A,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.d.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x8B    =>  Instruction {
                name:       "ADC A, E",
                opcode:     0x8B,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.e.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x8C    =>  Instruction {
                name:       "ADC A, H",
                opcode:     0x8C,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.h.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x8D    =>  Instruction {
                name:       "ADC A, L",
                opcode:     0x8D,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.l.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x8E    =>  Instruction {
                name:       "ADC A, (HL)",
                opcode:     0x8E,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.bus.read8(cpu.read_hl() as usize).wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x8F    =>  Instruction {
                name:       "ADC A, A",
                opcode:     0x8F,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.a.wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x90    =>  Instruction {
                name:       "SUB A, B",
                opcode:     0x90,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.b;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x91    =>  Instruction {
                name:       "SUB A, C",
                opcode:     0x91,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.c;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x92    =>  Instruction {
                name:       "SUB A, D",
                opcode:     0x92,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.d;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x93    =>  Instruction {
                name:       "SUB A, E",
                opcode:     0x97,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.e;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x94    =>  Instruction {
                name:       "SUB A, H",
                opcode:     0x94,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.h;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x95    =>  Instruction {
                name:       "SUB A, L",
                opcode:     0x95,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.l;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x96    =>  Instruction {
                name:       "SUB A, (HL)",
                opcode:     0x96,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.bus.read8(cpu.read_hl() as usize);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x97    =>  Instruction {
                name:       "SUB A, A",
                opcode:     0x97,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.a;
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x98    =>  Instruction {
                name:       "SBC A, B",
                opcode:     0x98,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.b.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x99    =>  Instruction {
                name:       "SBC A, C",
                opcode:     0x99,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.c.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x9A    =>  Instruction {
                name:       "SBC A, D",
                opcode:     0x9A,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.d.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x9B    =>  Instruction {
                name:       "SBC A, E",
                opcode:     0x9B,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.e.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x9C    =>  Instruction {
                name:       "SBC A, H",
                opcode:     0x9C,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.h.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x9D    =>  Instruction {
                name:       "SBC A, L",
                opcode:     0x9D,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.l.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x9E    =>  Instruction {
                name:       "SBC A, (HL)",
                opcode:     0x9E,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.bus.read8(cpu.read_hl() as usize).wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x9F    =>  Instruction {
                name:       "SBC A, A",
                opcode:     0x9F,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.a.wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xA0    =>  Instruction {
                name:       "AND A, B",
                opcode:     0xA0,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.b;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA1    =>  Instruction {
                name:       "AND A, C",
                opcode:     0xA1,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.c;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA2    =>  Instruction {
                name:       "AND A, D",
                opcode:     0xA2,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.d;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA3    =>  Instruction {
                name:       "AND A, E",
                opcode:     0xA3,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.e;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA4    =>  Instruction {
                name:       "AND A, H",
                opcode:     0xA4,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.h;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA5    =>  Instruction {
                name:       "AND A, L",
                opcode:     0xA5,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.l;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA6    =>  Instruction {
                name:       "AND A, (HL)",
                opcode:     0xA6,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.bus.read8(cpu.read_hl() as usize);
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA7    =>  Instruction {
                name:       "AND A, A",
                opcode:     0xA7,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.a;
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },            
            0xA8    =>  Instruction {
                name:       "XOR A, B",
                opcode:     0xA8,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.b;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xA9    =>  Instruction {
                name:       "XOR A, C",
                opcode:     0xA9,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.c;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xAA    =>  Instruction {
                name:       "XOR A, D",
                opcode:     0xAA,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.d;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xAB    =>  Instruction {
                name:       "XOR A, E",
                opcode:     0xAB,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.e;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xAC    =>  Instruction {
                name:       "XOR A, H",
                opcode:     0xAC,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.h;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xAD    =>  Instruction {
                name:       "XOR A, L",
                opcode:     0xAD,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.l;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xAE    =>  Instruction {
                name:       "XOR A, (HL)",
                opcode:     0xAE,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.bus.read8(cpu.read_hl() as usize);
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xAF    =>  Instruction {
                name:       "XOR A, A",
                opcode:     0xAF,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.a;
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },            
            0xB0    =>  Instruction {
                name:       "OR A, B",
                opcode:     0xB0,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.b;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xB1    =>  Instruction {
                name:       "OR A, C",
                opcode:     0xB1,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.c;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xB2    =>  Instruction {
                name:       "OR A, D",
                opcode:     0xB2,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.d;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xB3    =>  Instruction {
                name:       "OR A, E",
                opcode:     0xB3,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.e;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xB4    =>  Instruction {
                name:       "OR A, H",
                opcode:     0xB4,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.h;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xB5    =>  Instruction {
                name:       "OR A, L",
                opcode:     0xB5,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.l;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xB6    =>  Instruction {
                name:       "OR A, (HL)",
                opcode:     0xB6,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.bus.read8(cpu.read_hl() as usize);
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },            
            0xB7    =>  Instruction {
                name:       "OR A, A",
                opcode:     0xB7,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.a;
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },            
            0xB8    =>  Instruction {
                name:       "CP A, B",
                opcode:     0xB8,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.b;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xB9    =>  Instruction {
                name:       "CP A, C",
                opcode:     0xB9,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.c;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xBA    =>  Instruction {
                name:       "CP A, D",
                opcode:     0xBA,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.d;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xBB    =>  Instruction {
                name:       "CP A, E",
                opcode:     0xBB,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.e;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xBC    =>  Instruction {
                name:       "CP A, H",
                opcode:     0xBC,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.h;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xBD    =>  Instruction {
                name:       "CP A, L",
                opcode:     0xBD,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.l;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xBE    =>  Instruction {
                name:       "CP A, (HL)",
                opcode:     0xBE,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.bus.read8(cpu.read_hl() as usize);
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },            
            0xBF    =>  Instruction {
                name:       "CP A, A",
                opcode:     0xBF,
                cycles:     4,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.a;
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xC0    =>  Instruction {
                name:       "RET NZ",
                opcode:     0xC0,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.f & Flags::Z != Flags::Z {
                        let lo = cpu.pop();
                        let hi = cpu.pop();
                        cpu.pc = ((hi as i16) << 8) as u16 + lo as u16;
                    }
                    Ok(())
                },
            },
            0xC1    =>  Instruction {
                name:       "POP BC",
                opcode:     0xC1,
                cycles:     12,
                operation:  |cpu| {
                    cpu.c = cpu.pop();
                    cpu.b = cpu.pop();
                    Ok(())
                },
            },
            0xC2    =>  Instruction {
                name:       "JP NZ, nn",
                opcode:     0xC2,
                cycles:     12,
                operation:  |cpu| {
                    if cpu.f & Flags::Z != Flags::Z {
                        cpu.pc = cpu.fetch16();
                    }
                    Ok(())
                },
            },
            0xC3    =>  Instruction {
                name:       "JP nn",
                opcode:     0xC3,
                cycles:     12,
                operation:  |cpu| {
                    cpu.pc = cpu.fetch16();
                    Ok(())
                },
            },
            0xC4    =>  Instruction {
                name:       "CALL NZ, nn",
                opcode:     0xC4,
                cycles:     12,
                operation:  |cpu| {
                    let lo = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let hi = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let nn = ((hi as u16) << 8) | lo as u16;
                    if cpu.f & Flags::Z != Flags::Z {
                        cpu.push((cpu.pc >> 8) as u8);
                        cpu.push((cpu.pc & 0xFF) as u8);
                        cpu.pc = nn;
                    }
                    Ok(())
                },
            },
            0xC5    =>  Instruction {
                name:       "PUSH BC",
                opcode:     0xC5,
                cycles:     16,
                operation:  |cpu| {
                    cpu.push(cpu.b);
                    cpu.push(cpu.c);
                    Ok(())
                },
            },
            0xC6    =>  Instruction {
                name:       "ADD A, #",
                opcode:     0xC6,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.fetch();
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xC7    =>  Instruction {
                name:       "RST 0x00",
                opcode:     0xC7,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0000;
                    Ok(())
                },
            },
            0xC8    =>  Instruction {
                name:       "RET Z",
                opcode:     0xC8,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.f & Flags::Z == Flags::Z {
                        let lo = cpu.pop();
                        let hi = cpu.pop();
                        cpu.pc = ((hi as i16) << 8) as u16 + lo as u16;
                    }
                    Ok(())
                },
            },
            0xC9    =>  Instruction {
                name:       "RET",
                opcode:     0xC9,
                cycles:     8,
                operation:  |cpu| {
                    let lo = cpu.pop();
                    let hi = cpu.pop();
                    cpu.pc = ((hi as i16) << 8) as u16 + lo as u16;
                    Ok(())
                },
            },
            0xCA    =>  Instruction {
                name:       "JP Z, nn",
                opcode:     0xCA,
                cycles:     12,
                operation:  |cpu| {
                    if cpu.f & Flags::Z == Flags::Z {
                        cpu.pc = cpu.fetch16();
                    }
                    Ok(())
                },
            },
            0xCB    =>  {
                let opcode_cb = self.fetch();
                self.decode_cb(opcode_cb)
            },
            0xCC    =>  Instruction {
                name:       "CALL Z, nn",
                opcode:     0xCC,
                cycles:     12,
                operation:  |cpu| {
                    let lo = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let hi = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let nn = ((hi as u16) << 8) | lo as u16;
                    if cpu.f & Flags::Z == Flags::Z {
                        cpu.push((cpu.pc >> 8) as u8);
                        cpu.push((cpu.pc & 0xFF) as u8);
                        cpu.pc = nn;
                    }
                    Ok(())
                },
            },
            0xCD    =>  Instruction {
                name:       "CALL nn",
                opcode:     0xCD,
                cycles:     12,
                operation:  |cpu| {
                    let lo = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let hi = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let nn = ((hi as u16) << 8) | lo as u16;
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = nn;
                    Ok(())
                },
            },
            0xCE    =>  Instruction {
                name:       "ADC A, #",
                opcode:     0xCE,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.fetch().wrapping_add(c);
                    cpu.a = a.wrapping_add(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.a^a^n)&0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.a < a {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xCF    =>  Instruction {
                name:       "RST 0x08",
                opcode:     0xCF,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0008;
                    Ok(())
                },
            },
            0xD0    =>  Instruction {
                name:       "RET NC",
                opcode:     0xD0,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.f & Flags::C != Flags::C {
                        let lo = cpu.pop();
                        let hi = cpu.pop();
                        cpu.pc = ((hi as i16) << 8) as u16 + lo as u16;
                    }
                    Ok(())
                },
            },            
            0xD1    =>  Instruction {
                name:       "POP DE",
                opcode:     0xD1,
                cycles:     12,
                operation:  |cpu| {
                    cpu.e = cpu.pop();
                    cpu.d = cpu.pop();
                    Ok(())
                },
            },
            0xD2    =>  Instruction {
                name:       "JP NC, nn",
                opcode:     0xD2,
                cycles:     12,
                operation:  |cpu| {
                    if cpu.f & Flags::C != Flags::C {
                        cpu.pc = cpu.fetch16();
                    }
                    Ok(())
                },
            },
            // 0xD3:    Undefined
            0xD4    =>  Instruction {
                name:       "CALL NC, nn",
                opcode:     0xD4,
                cycles:     12,
                operation:  |cpu| {
                    let lo = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let hi = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let nn = ((hi as u16) << 8) | lo as u16;
                    if cpu.f & Flags::C != Flags::C {
                        cpu.push((cpu.pc >> 8) as u8);
                        cpu.push((cpu.pc & 0xFF) as u8);
                        cpu.pc = nn;
                    }
                    Ok(())
                },
            },
            0xD5    =>  Instruction {
                name:       "PUSH DE",
                opcode:     0xD5,
                cycles:     16,
                operation:  |cpu| {
                    cpu.push(cpu.d);
                    cpu.push(cpu.e);
                    Ok(())
                },
            },
            0xD6    =>  Instruction {
                name:       "SUB A, #",
                opcode:     0xD6,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.fetch();
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xD7    =>  Instruction {
                name:       "RST 0x10",
                opcode:     0xD7,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0010;
                    Ok(())
                },
            },
            0xD8    =>  Instruction {
                name:       "RET C",
                opcode:     0xD8,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.f & Flags::C == Flags::C {
                        let lo = cpu.pop();
                        let hi = cpu.pop();
                        cpu.pc = ((hi as i16) << 8) as u16 + lo as u16;
                    }
                    Ok(())
                },
            },
            0xD9    =>  Instruction {
                name:       "RETI",
                opcode:     0xD9,
                cycles:     8,
                operation:  |cpu| {
                    let lo = cpu.pop();
                    let hi = cpu.pop();
                    cpu.pc = ((hi as i16) << 8) as u16 + lo as u16;
                    cpu.bus.enable_irq();
                    Ok(())
                },
            },            
            0xDA    =>  Instruction {
                name:       "JP C, nn",
                opcode:     0xDA,
                cycles:     12,
                operation:  |cpu| {
                    if cpu.f & Flags::C != Flags::C {
                        cpu.pc = cpu.fetch16();
                    }
                    Ok(())
                },
            },
            // 0xDB:    Undefined            
            0xDC    =>  Instruction {
                name:       "CALL C, nn",
                opcode:     0xDC,
                cycles:     12,
                operation:  |cpu| {
                    let lo = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let hi = cpu.bus.read8(cpu.pc as usize);
                    cpu.pc += 1;
                    let nn = ((hi as u16) << 8) | lo as u16;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.push((cpu.pc >> 8) as u8);
                        cpu.push((cpu.pc & 0xFF) as u8);
                        cpu.pc = nn;
                    }
                    Ok(())
                },
            },
            // 0xDD:    Undefined
            0xDE    =>  Instruction {
                name:       "SBC A, #",
                opcode:     0xDE,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let c = (cpu.f & Flags::C == Flags::C) as u8;
                    let n = cpu.fetch().wrapping_add(c);
                    cpu.a = a.wrapping_sub(n);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xDF    =>  Instruction {
                name:       "RST 0x18",
                opcode:     0xDF,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0018;
                    Ok(())
                },
            },
            0xE0    =>  Instruction {
                name:       "LDH (n), A",
                opcode:     0xE0,
                cycles:     12,
                operation:  |cpu| {
                    let addr = 0xFF00 + (cpu.fetch() as usize);
                    cpu.bus.write8(addr, cpu.a);
                    Ok(())
                },
            },
            0xE1    =>  Instruction {
                name:       "POP HL",
                opcode:     0xE1,
                cycles:     12,
                operation:  |cpu| {
                    cpu.l = cpu.pop();
                    cpu.h = cpu.pop();
                    Ok(())
                },
            },
            0xE2    =>  Instruction {
                name:       "LD (C), A",
                opcode:     0xE2,
                cycles:     8,
                operation:  |cpu| {
                    let addr = 0xFF00 + (cpu.c as usize);
                    cpu.bus.write8(addr, cpu.a);
                    Ok(())
                },
            },
            // 0xE3:    Undefined
            // 0xE4:    Undefined
            0xE5    =>  Instruction {
                name:       "PUSH HL",
                opcode:     0xE5,
                cycles:     16,
                operation:  |cpu| {
                    cpu.push(cpu.h);
                    cpu.push(cpu.l);
                    Ok(())
                },
            },
            0xE6    =>  Instruction {
                name:       "AND A, #",
                opcode:     0xE6,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.fetch();
                    cpu.a = a & n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xE7    =>  Instruction {
                name:       "RST 0x20",
                opcode:     0xE7,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0020;
                    Ok(())
                },
            },
            0xE8    =>  Instruction {
                name:       "ADD SP, #",
                opcode:     0xE8,
                cycles:     16,
                operation:  |cpu| {
                    let sp = cpu.sp;
                    let n = cpu.fetch() as i8 as i16;
                    cpu.sp = (sp as i16).wrapping_add(n) as u16;
                    cpu.f.remove(Flags::Z);
                    cpu.f.remove(Flags::N);
                    let c = (sp ^ n as u16) ^ (sp.wrapping_add(n as u16));
                    if c & 0x10 == 0x10 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if c & 0x100 == 0x100 {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xE9    =>  Instruction {
                name:       "JP (HL)",
                opcode:     0xE9,
                cycles:     4,
                operation:  |cpu| {
                    cpu.pc = cpu.read_hl();
                    Ok(())
                },
            },
            0xEA    =>  Instruction {
                name:       "LD (nn), A",
                opcode:     0xEA,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.fetch16() as usize;
                    cpu.bus.write8(addr, cpu.a);
                    Ok(())
                },
            },
            // 0xEB:    Undefined
            // 0xEC:    Undefined
            // 0xED:    Undefined
            0xEE    =>  Instruction {
                name:       "XOR A, #",
                opcode:     0xEE,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.fetch();
                    cpu.a = a ^ n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xEF    =>  Instruction {
                name:       "RST 0x28",
                opcode:     0xEF,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0028;
                    Ok(())
                },
            },            
            0xF0    =>  Instruction {
                name:       "LDH A, (n)",
                opcode:     0xF0,
                cycles:     12,
                operation:  |cpu| {
                    let addr = 0xFF00 + (cpu.fetch() as usize);
                    cpu.a = cpu.bus.read8(addr);
                    Ok(())
                },
            },            
            0xF1    =>  Instruction {
                name:       "POP AF",
                opcode:     0xF1,
                cycles:     12,
                operation:  |cpu| {
                    cpu.f = Flags::from_bits_truncate(cpu.pop());
                    cpu.a = cpu.pop();
                    Ok(())
                },
            },
            0xF2    =>  Instruction {
                name:       "LD A, (C)",
                opcode:     0xF2,
                cycles:     8,
                operation:  |cpu| {
                    let addr = 0xFF00 + (cpu.c as usize);
                    cpu.a = cpu.bus.read8(addr);
                    Ok(())
                },
            },
            0xF3    =>  Instruction {
                name:       "DI",
                opcode:     0xF3,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.disable_irq();
                    Ok(())
                },
            },
            // 0xF4:    Undefined
            0xF5    =>  Instruction {
                name:       "PUSH AF",
                opcode:     0xF5,
                cycles:     16,
                operation:  |cpu| {
                    cpu.push(cpu.a);
                    cpu.push(cpu.f.bits());
                    Ok(())
                },
            },
            0xF6    =>  Instruction {
                name:       "OR A, #",
                opcode:     0xB6,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.fetch();
                    cpu.a = a | n;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0xF7    =>  Instruction {
                name:       "RST 0x30",
                opcode:     0xF7,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0030;
                    Ok(())
                },
            },
            0xF8    =>  Instruction {
                name:       "LDHL SP, n",
                opcode:     0xF8,
                cycles:     12,
                operation:  |cpu| {
                    let n = cpu.fetch() as i8 as i16;
                    let value = ((cpu.sp as i16).wrapping_add(n)) as u16;
                    cpu.write_hl(value);
                    cpu.f.remove(Flags::Z);
                    cpu.f.remove(Flags::N);
                    if n >= 0 {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if cpu.sp > value {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xF9    =>  Instruction {
                name:       "LD BC, nn",
                opcode:     0xF9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.sp = cpu.read_hl();
                    Ok(())
                },
            },
            0xFA    =>  Instruction {
                name:       "LD A, (nn)",
                opcode:     0xFA,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.fetch16() as usize;
                    cpu.a = cpu.bus.read8(addr);
                    Ok(())
                },
            },
            0xFB    =>  Instruction {
                name:       "EI",
                opcode:     0xFB,
                cycles:     4,
                operation:  |cpu| {
                    cpu.bus.enable_irq();
                    Ok(())
                },
            },
            // 0xFC:    Undefined
            // 0xFD:    Undefined
            0xFE    =>  Instruction {
                name:       "CP A, #",
                opcode:     0xFE,
                cycles:     8,
                operation:  |cpu| {
                    let a = cpu.a;
                    let n = cpu.fetch();
                    if  a == n {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.insert(Flags::N);
                    if a&0x0F < n&0x0F {
                        cpu.f.insert(Flags::H);
                    } else {
                        cpu.f.remove(Flags::H);
                    }
                    if a < n {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0xFF    =>  Instruction {
                name:       "RST 0x38",
                opcode:     0xFF,
                cycles:     32,
                operation:  |cpu| {
                    cpu.push((cpu.pc >> 8) as u8);
                    cpu.push((cpu.pc & 0xFF) as u8);
                    cpu.pc = 0x0038;
                    Ok(())
                },
            },

            _       =>  unimplemented!("can't decode: 0x{:02x}\ncpu={}", opcode, self),
        }
    }

    fn decode_cb(&mut self, opcode: u8) -> Instruction {
        match opcode {
            0x00    =>  Instruction {
                name:       "RLC B",
                opcode:     0x00,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x80 == 0x80;
                    cpu.b = cpu.b.rotate_left(1);
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x01    =>  Instruction {
                name:       "RLC C",
                opcode:     0x01,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x80 == 0x80;
                    cpu.c = cpu.c.rotate_left(1);
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x02    =>  Instruction {
                name:       "RLC D",
                opcode:     0x02,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.d & 0x80 == 0x80;
                    cpu.d = cpu.d.rotate_left(1);
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x03    =>  Instruction {
                name:       "RLC E",
                opcode:     0x03,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x80 == 0x80;
                    cpu.e = cpu.e.rotate_left(1);
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x04    =>  Instruction {
                name:       "RLC H",
                opcode:     0x04,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x80 == 0x80;
                    cpu.h = cpu.h.rotate_left(1);
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x05    =>  Instruction {
                name:       "RLC L",
                opcode:     0x05,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x80 == 0x80;
                    cpu.l = cpu.l.rotate_left(1);
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x06    =>  Instruction {
                name:       "RLC (HL)",
                opcode:     0x06,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x80 == 0x80;
                    cpu.bus.write8(addr, cpu.bus.read8(addr).rotate_left(1));
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x07    =>  Instruction {
                name:       "RLC A",
                opcode:     0x07,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x80 == 0x80;
                    cpu.a = cpu.a.rotate_left(1);
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x08    =>  Instruction {
                name:       "RRC B",
                opcode:     0x08,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x01 == 0x01;
                    cpu.b = cpu.b >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.b |= 0x80;
                    }
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x09    =>  Instruction {
                name:       "RRC C",
                opcode:     0x09,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x01 == 0x01;
                    cpu.c = cpu.c >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.c |= 0x80;
                    }
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x0A    =>  Instruction {
                name:       "RRC D",
                opcode:     0x0A,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.a |= 0x80;
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x0B    =>  Instruction {
                name:       "RRC E",
                opcode:     0x08,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x01 == 0x01;
                    cpu.e = cpu.e >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.e |= 0x80;
                    }
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x0C    =>  Instruction {
                name:       "RRC H",
                opcode:     0x0C,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x01 == 0x01;
                    cpu.h = cpu.h >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.h |= 0x80;
                    }
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x0D    =>  Instruction {
                name:       "RRC L",
                opcode:     0x0D,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x01 == 0x01;
                    cpu.l = cpu.l >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.l |= 0x80;
                    }
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x0E    =>  Instruction {
                name:       "RRC (HL)",
                opcode:     0x0E,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x01 == 0x01;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) >> 1);
                    if cpu.f & Flags::C == Flags::C {
                        cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x80);
                    }
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            
            0x0F    =>  Instruction {
                name:       "RRC A",
                opcode:     0x0F,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.a |= 0x80;
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x10    =>  Instruction {
                name:       "RL B",
                opcode:     0x010,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x80 == 0x80;
                    cpu.b = cpu.b << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.b |= 1;
                    }
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x11    =>  Instruction {
                name:       "RL C",
                opcode:     0x011,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x80 == 0x80;
                    cpu.c = cpu.c << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.c |= 1;
                    }
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x12    =>  Instruction {
                name:       "RL D",
                opcode:     0x010,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.d & 0x80 == 0x80;
                    cpu.d = cpu.d << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.d |= 1;
                    }
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x13    =>  Instruction {
                name:       "RL E",
                opcode:     0x013,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x80 == 0x80;
                    cpu.e = cpu.e << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.e |= 1;
                    }
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x14    =>  Instruction {
                name:       "RL H",
                opcode:     0x014,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x80 == 0x80;
                    cpu.h = cpu.h << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.h |= 1;
                    }
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x15    =>  Instruction {
                name:       "RL L",
                opcode:     0x015,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x80 == 0x80;
                    cpu.l = cpu.l << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.l |= 1;
                    }
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x16    =>  Instruction {
                name:       "RL (HL)",
                opcode:     0x016,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x80 == 0x80;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) << 1);
                    if cpu.f & Flags::C == Flags::C {
                        cpu.bus.write8(addr, cpu.bus.read8(addr) | 1);
                    }
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },            
            0x17    =>  Instruction {
                name:       "RL A",
                opcode:     0x017,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x80 == 0x80;
                    cpu.a = cpu.a << 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.a |= 1;
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x18    =>  Instruction {
                name:       "RR B",
                opcode:     0x018,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x01 == 0x01;
                    cpu.b = cpu.b >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.b |= 0x80;
                    }
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x19    =>  Instruction {
                name:       "RR C",
                opcode:     0x019,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x01 == 0x01;
                    cpu.c = cpu.c >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.c |= 0x80;
                    }
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x1A    =>  Instruction {
                name:       "RR D",
                opcode:     0x01A,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.d & 0x01 == 0x01;
                    cpu.d = cpu.d >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.d |= 0x80;
                    }
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x1B    =>  Instruction {
                name:       "RR E",
                opcode:     0x01B,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x01 == 0x01;
                    cpu.e = cpu.e >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.e |= 0x80;
                    }
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x1C    =>  Instruction {
                name:       "RR H",
                opcode:     0x01C,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x01 == 0x01;
                    cpu.h = cpu.h >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.h |= 0x80;
                    }
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x1D    =>  Instruction {
                name:       "RR L",
                opcode:     0x01D,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x01 == 0x01;
                    cpu.l = cpu.l >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.l |= 0x80;
                    }
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x1E    =>  Instruction {
                name:       "RR (HL)",
                opcode:     0x01E,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x01 == 0x01;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) >> 1);
                    if cpu.f & Flags::C == Flags::C {
                        cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x80);
                    }
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },

            0x1F    =>  Instruction {
                name:       "RR A",
                opcode:     0x01F,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a >> 1;
                    if cpu.f & Flags::C == Flags::C {
                        cpu.a |= 0x80;
                    }
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x20    =>  Instruction {
                name:       "SLA B",
                opcode:     0x20,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x80 == 0x80;
                    cpu.b = cpu.b << 1;
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x21    =>  Instruction {
                name:       "SLA C",
                opcode:     0x21,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x80 == 0x80;
                    cpu.c = cpu.c << 1;
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x22    =>  Instruction {
                name:       "SLA D",
                opcode:     0x22,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.d & 0x80 == 0x80;
                    cpu.d = cpu.d << 1;
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x23    =>  Instruction {
                name:       "SLA E",
                opcode:     0x23,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x80 == 0x80;
                    cpu.e = cpu.e << 1;
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x24    =>  Instruction {
                name:       "SLA H",
                opcode:     0x24,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x80 == 0x80;
                    cpu.h = cpu.h << 1;
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x25    =>  Instruction {
                name:       "SLA L",
                opcode:     0x25,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x80 == 0x80;
                    cpu.l = cpu.l << 1;
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x26    =>  Instruction {
                name:       "SLA (HL)",
                opcode:     0x26,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x80 == 0x80;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) << 1);
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x27    =>  Instruction {
                name:       "SLA A",
                opcode:     0x27,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x80 == 0x80;
                    cpu.a = cpu.a << 1;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x28    =>  Instruction {
                name:       "SRA B",
                opcode:     0x28,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x01 == 0x01;
                    cpu.b = cpu.b >> 1 | cpu.b & 0x80;
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x29    =>  Instruction {
                name:       "SRA C",
                opcode:     0x29,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x01 == 0x01;
                    cpu.c = cpu.c >> 1 | cpu.c & 0x80;
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2A    =>  Instruction {
                name:       "SRA D",
                opcode:     0x2A,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.d & 0x01 == 0x01;
                    cpu.d = cpu.d >> 1 | cpu.d & 0x80;
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2B    =>  Instruction {
                name:       "SRA E",
                opcode:     0x2B,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x01 == 0x01;
                    cpu.e = cpu.e >> 1 | cpu.e & 0x80;
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2C    =>  Instruction {
                name:       "SRA H",
                opcode:     0x2C,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x01 == 0x01;
                    cpu.h = cpu.h >> 1 | cpu.h & 0x80;
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2D    =>  Instruction {
                name:       "SRA L",
                opcode:     0x2D,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x01 == 0x01;
                    cpu.l = cpu.l >> 1 | cpu.l & 0x80;
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2E    =>  Instruction {
                name:       "SRA (HL)",
                opcode:     0x2E,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x01 == 0x01;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) >> 1);
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x2F    =>  Instruction {
                name:       "SRA A",
                opcode:     0x2F,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a >> 1;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x30    =>  Instruction {
                name:       "SWAP B",
                opcode:     0x30,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.b & 0xF0;
                    let lo = cpu.b & 0x0F;
                    cpu.b = hi | lo;
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x31    =>  Instruction {
                name:       "SWAP C",
                opcode:     0x31,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.c & 0xF0;
                    let lo = cpu.c & 0x0F;
                    cpu.c = hi | lo;
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x32    =>  Instruction {
                name:       "SWAP D",
                opcode:     0x30,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.d & 0xF0;
                    let lo = cpu.d & 0x0F;
                    cpu.d = hi | lo;
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x33    =>  Instruction {
                name:       "SWAP E",
                opcode:     0x30,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.e & 0xF0;
                    let lo = cpu.e & 0x0F;
                    cpu.e = hi | lo;
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x34    =>  Instruction {
                name:       "SWAP H",
                opcode:     0x30,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.h & 0xF0;
                    let lo = cpu.h & 0x0F;
                    cpu.h = hi | lo;
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x35    =>  Instruction {
                name:       "SWAP L",
                opcode:     0x35,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.l & 0xF0;
                    let lo = cpu.l & 0x0F;
                    cpu.l = hi | lo;
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x36    =>  Instruction {
                name:       "SWAP (HL)",
                opcode:     0x36,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let hi = cpu.bus.read8(addr) & 0xF0;
                    let lo = cpu.bus.read8(addr) & 0x0F;
                    cpu.bus.write8(addr, hi | lo);
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },
            0x37    =>  Instruction {
                name:       "SWAP A",
                opcode:     0x37,
                cycles:     8,
                operation:  |cpu| {
                    let hi = cpu.a & 0xF0;
                    let lo = cpu.a & 0x0F;
                    cpu.a = hi | lo;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    cpu.f.remove(Flags::C);
                    Ok(())
                },
            },            
            0x38    =>  Instruction {
                name:       "SRL B",
                opcode:     0x38,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.b & 0x01 == 0x01;
                    cpu.b = cpu.b >> 1;
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x39    =>  Instruction {
                name:       "SRL C",
                opcode:     0x39,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.c & 0x01 == 0x01;
                    cpu.c = cpu.c >> 1;
                    if cpu.c == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3A    =>  Instruction {
                name:       "SRL D",
                opcode:     0x3A,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.d & 0x01 == 0x01;
                    cpu.d = cpu.d >> 1;
                    if cpu.d == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3B    =>  Instruction {
                name:       "SRL E",
                opcode:     0x3B,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.e & 0x01 == 0x01;
                    cpu.e = cpu.e >> 1;
                    if cpu.e == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3C    =>  Instruction {
                name:       "SRL H",
                opcode:     0x3C,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.h & 0x01 == 0x01;
                    cpu.h = cpu.h >> 1;
                    if cpu.h == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3D    =>  Instruction {
                name:       "SRL L",
                opcode:     0x3D,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.l & 0x01 == 0x01;
                    cpu.l = cpu.l >> 1;
                    if cpu.l == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3E    =>  Instruction {
                name:       "SRL (HL)",
                opcode:     0x3E,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let carry = cpu.bus.read8(addr) & 0x01 == 0x01;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) >> 1);
                    if cpu.bus.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x3F    =>  Instruction {
                name:       "SRL A",
                opcode:     0x2F,
                cycles:     8,
                operation:  |cpu| {
                    let carry = cpu.a & 0x01 == 0x01;
                    cpu.a = cpu.a >> 1;
                    if cpu.a == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.remove(Flags::H);
                    if carry {
                        cpu.f.insert(Flags::C);
                    } else {
                        cpu.f.remove(Flags::C);
                    }
                    Ok(())
                },
            },
            0x40    =>  Instruction {
                name:       "BIT 0, B",
                opcode:     0x40,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x41    =>  Instruction {
                name:       "BIT 0, C",
                opcode:     0x41,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x42    =>  Instruction {
                name:       "BIT 0, D",
                opcode:     0x42,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x43    =>  Instruction {
                name:       "BIT 0, E",
                opcode:     0x43,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x44    =>  Instruction {
                name:       "BIT 0, H",
                opcode:     0x44,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x45    =>  Instruction {
                name:       "BIT 0, L",
                opcode:     0x45,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x46    =>  Instruction {
                name:       "BIT 0, (HL)",
                opcode:     0x46,
                cycles:     16,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x47    =>  Instruction {
                name:       "BIT 0, A",
                opcode:     0x47,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x01 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x48    =>  Instruction {
                name:       "BIT 1, B",
                opcode:     0x48,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x49    =>  Instruction {
                name:       "BIT 1, C",
                opcode:     0x49,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x4A    =>  Instruction {
                name:       "BIT 1, D",
                opcode:     0x4A,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x4B    =>  Instruction {
                name:       "BIT 1, E",
                opcode:     0x4B,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x4C    =>  Instruction {
                name:       "BIT 1, H",
                opcode:     0x4C,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x4D    =>  Instruction {
                name:       "BIT 1, L",
                opcode:     0x4D,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x4E    =>  Instruction {
                name:       "BIT 1, (HL)",
                opcode:     0x4E,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x4F    =>  Instruction {
                name:       "BIT 1, A",
                opcode:     0x4F,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x02 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x50    =>  Instruction {
                name:       "BIT 2, B",
                opcode:     0x50,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x51    =>  Instruction {
                name:       "BIT 2, C",
                opcode:     0x51,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x52    =>  Instruction {
                name:       "BIT 2, D",
                opcode:     0x52,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x53    =>  Instruction {
                name:       "BIT 2, E",
                opcode:     0x53,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x54    =>  Instruction {
                name:       "BIT 2, H",
                opcode:     0x54,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x55    =>  Instruction {
                name:       "BIT 2, L",
                opcode:     0x55,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x56    =>  Instruction {
                name:       "BIT 2, (HL)",
                opcode:     0x56,
                cycles:     16,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x57    =>  Instruction {
                name:       "BIT 2, A",
                opcode:     0x57,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x04 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x58    =>  Instruction {
                name:       "BIT 3, B",
                opcode:     0x58,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x59    =>  Instruction {
                name:       "BIT 3, C",
                opcode:     0x59,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x5A    =>  Instruction {
                name:       "BIT 3, D",
                opcode:     0x5A,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x5B    =>  Instruction {
                name:       "BIT 3, E",
                opcode:     0x5B,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x5C    =>  Instruction {
                name:       "BIT 3, H",
                opcode:     0x5C,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x5D    =>  Instruction {
                name:       "BIT 3, L",
                opcode:     0x5D,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x5E    =>  Instruction {
                name:       "BIT 3, (HL)",
                opcode:     0x5E,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x5F    =>  Instruction {
                name:       "BIT 3, A",
                opcode:     0x5F,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x08 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x60    =>  Instruction {
                name:       "BIT 4, B",
                opcode:     0x60,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x61    =>  Instruction {
                name:       "BIT 4, C",
                opcode:     0x61,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x62    =>  Instruction {
                name:       "BIT 4, D",
                opcode:     0x62,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x63    =>  Instruction {
                name:       "BIT 4, E",
                opcode:     0x63,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x64    =>  Instruction {
                name:       "BIT 4, H",
                opcode:     0x64,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x65    =>  Instruction {
                name:       "BIT 4, L",
                opcode:     0x65,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x66    =>  Instruction {
                name:       "BIT 4, (HL)",
                opcode:     0x66,
                cycles:     16,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x67    =>  Instruction {
                name:       "BIT 4, A",
                opcode:     0x67,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x10 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x68    =>  Instruction {
                name:       "BIT 5, B",
                opcode:     0x68,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x69    =>  Instruction {
                name:       "BIT 5, C",
                opcode:     0x69,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x6A    =>  Instruction {
                name:       "BIT 5, D",
                opcode:     0x6A,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x6B    =>  Instruction {
                name:       "BIT 5, E",
                opcode:     0x6B,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x6C    =>  Instruction {
                name:       "BIT 5, H",
                opcode:     0x6C,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x6D    =>  Instruction {
                name:       "BIT 5, L",
                opcode:     0x6D,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x6E    =>  Instruction {
                name:       "BIT 5, (HL)",
                opcode:     0x6E,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x6F    =>  Instruction {
                name:       "BIT 5, A",
                opcode:     0x6F,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x20 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x70    =>  Instruction {
                name:       "BIT 6, B",
                opcode:     0x70,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x71    =>  Instruction {
                name:       "BIT 6, C",
                opcode:     0x71,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x72    =>  Instruction {
                name:       "BIT 6, D",
                opcode:     0x72,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x73    =>  Instruction {
                name:       "BIT 6, E",
                opcode:     0x73,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x74    =>  Instruction {
                name:       "BIT 6, H",
                opcode:     0x74,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x75    =>  Instruction {
                name:       "BIT 6, L",
                opcode:     0x75,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x76    =>  Instruction {
                name:       "BIT 6, (HL)",
                opcode:     0x76,
                cycles:     16,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x77    =>  Instruction {
                name:       "BIT 6, A",
                opcode:     0x77,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x40 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x78    =>  Instruction {
                name:       "BIT 7, B",
                opcode:     0x78,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.b & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x79    =>  Instruction {
                name:       "BIT 7, C",
                opcode:     0x79,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.c & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x7A    =>  Instruction {
                name:       "BIT 7, D",
                opcode:     0x7A,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.d & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x7B    =>  Instruction {
                name:       "BIT 7, E",
                opcode:     0x7B,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.e & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x7C    =>  Instruction {
                name:       "BIT 7, H",
                opcode:     0x7C,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.h & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x7D    =>  Instruction {
                name:       "BIT 7, L",
                opcode:     0x7D,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.l & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x7E    =>  Instruction {
                name:       "BIT 7, (HL)",
                opcode:     0x7E,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.bus.read8(cpu.read_hl() as usize) & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x7F    =>  Instruction {
                name:       "BIT 7, A",
                opcode:     0x7F,
                cycles:     8,
                operation:  |cpu| {
                    if cpu.a & 0x80 == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    cpu.f.insert(Flags::H);
                    Ok(())
                },
            },
            0x80    =>  Instruction {
                name:       "RES 0, B",
                opcode:     0x80,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x01;
                    Ok(())
                },
            },
            0x81    =>  Instruction {
                name:       "RES 0, C",
                opcode:     0x81,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x01;
                    Ok(())
                },
            },
            0x82    =>  Instruction {
                name:       "RES 0, D",
                opcode:     0x82,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x01;
                    Ok(())
                },
            },
            0x83    =>  Instruction {
                name:       "RES 0, E",
                opcode:     0x83,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x01;
                    Ok(())
                },
            },
            0x84    =>  Instruction {
                name:       "RES 0, H",
                opcode:     0x84,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x01;
                    Ok(())
                },
            },
            0x85    =>  Instruction {
                name:       "RES 0, L",
                opcode:     0x85,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x01;
                    Ok(())
                },
            },
            0x86    =>  Instruction {
                name:       "RES 0, (HL)",
                opcode:     0x86,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x01);
                    Ok(())
                },
            },
            0x87    =>  Instruction {
                name:       "RES 0, A",
                opcode:     0x87,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x01;
                    Ok(())
                },
            },
            0x88    =>  Instruction {
                name:       "RES 1, B",
                opcode:     0x88,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x02;
                    Ok(())
                },
            },
            0x89    =>  Instruction {
                name:       "RES 1, C",
                opcode:     0x89,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x02;
                    Ok(())
                },
            },
            0x8A    =>  Instruction {
                name:       "RES 1, D",
                opcode:     0x8A,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x02;
                    Ok(())
                },
            },
            0x8B    =>  Instruction {
                name:       "RES 1, E",
                opcode:     0x8B,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x02;
                    Ok(())
                },
            },
            0x8C    =>  Instruction {
                name:       "RES 1, H",
                opcode:     0x8C,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x02;
                    Ok(())
                },
            },
            0x8D    =>  Instruction {
                name:       "RES 1, L",
                opcode:     0x8D,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x02;
                    Ok(())
                },
            },
            0x8E    =>  Instruction {
                name:       "RES 1, (HL)",
                opcode:     0x8E,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x02);
                    Ok(())
                },
            },
            0x8F    =>  Instruction {
                name:       "RES 1, A",
                opcode:     0x8F,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x02;
                    Ok(())
                },
            },
            0x90    =>  Instruction {
                name:       "RES 2, B",
                opcode:     0x90,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x04;
                    Ok(())
                },
            },
            0x91    =>  Instruction {
                name:       "RES 2, C",
                opcode:     0x91,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x04;
                    Ok(())
                },
            },
            0x92    =>  Instruction {
                name:       "RES 2, D",
                opcode:     0x92,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x04;
                    Ok(())
                },
            },
            0x93    =>  Instruction {
                name:       "RES 2, E",
                opcode:     0x93,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x04;
                    Ok(())
                },
            },
            0x94    =>  Instruction {
                name:       "RES 2, H",
                opcode:     0x94,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x04;
                    Ok(())
                },
            },
            0x95    =>  Instruction {
                name:       "RES 2, L",
                opcode:     0x95,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x04;
                    Ok(())
                },
            },
            0x96    =>  Instruction {
                name:       "RES 2, (HL)",
                opcode:     0x96,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x04);
                    Ok(())
                },
            },
            0x97    =>  Instruction {
                name:       "RES 2, A",
                opcode:     0x97,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x04;
                    Ok(())
                },
            },
            0x98    =>  Instruction {
                name:       "RES 3, B",
                opcode:     0x98,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x08;
                    Ok(())
                },
            },
            0x99    =>  Instruction {
                name:       "RES 3, C",
                opcode:     0x99,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x08;
                    Ok(())
                },
            },
            0x9A    =>  Instruction {
                name:       "RES 3, D",
                opcode:     0x9A,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x08;
                    Ok(())
                },
            },
            0x9B    =>  Instruction {
                name:       "RES 3, E",
                opcode:     0x9B,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x08;
                    Ok(())
                },
            },
            0x9C    =>  Instruction {
                name:       "RES 3, H",
                opcode:     0x9C,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x08;
                    Ok(())
                },
            },
            0x9D    =>  Instruction {
                name:       "RES 3, L",
                opcode:     0x9D,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x08;
                    Ok(())
                },
            },
            0x9E    =>  Instruction {
                name:       "RES 3, (HL)",
                opcode:     0x9E,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x08);
                    Ok(())
                },
            },
            0x9F    =>  Instruction {
                name:       "RES 3, A",
                opcode:     0x9F,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x08;
                    Ok(())
                },
            },
            0xA0    =>  Instruction {
                name:       "RES 4, B",
                opcode:     0xA0,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x10;
                    Ok(())
                },
            },
            0xA1    =>  Instruction {
                name:       "RES 4, C",
                opcode:     0xA1,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x10;
                    Ok(())
                },
            },
            0xA2    =>  Instruction {
                name:       "RES 4, D",
                opcode:     0xA2,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x10;
                    Ok(())
                },
            },
            0xA3    =>  Instruction {
                name:       "RES 4, E",
                opcode:     0xA3,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x10;
                    Ok(())
                },
            },
            0xA4    =>  Instruction {
                name:       "RES 4, H",
                opcode:     0xA4,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x10;
                    Ok(())
                },
            },
            0xA5    =>  Instruction {
                name:       "RES 4, L",
                opcode:     0xA5,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x10;
                    Ok(())
                },
            },
            0xA6    =>  Instruction {
                name:       "RES 4, (HL)",
                opcode:     0xA6,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x10);
                    Ok(())
                },
            },
            0xA7    =>  Instruction {
                name:       "RES 4, A",
                opcode:     0xA7,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x10;
                    Ok(())
                },
            },
            0xA8    =>  Instruction {
                name:       "RES 5, B",
                opcode:     0xA8,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x20;
                    Ok(())
                },
            },
            0xA9    =>  Instruction {
                name:       "RES 5, C",
                opcode:     0xA9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x20;
                    Ok(())
                },
            },
            0xAA    =>  Instruction {
                name:       "RES 5, D",
                opcode:     0xAA,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x20;
                    Ok(())
                },
            },
            0xAB    =>  Instruction {
                name:       "RES 5, E",
                opcode:     0xAB,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x20;
                    Ok(())
                },
            },
            0xAC    =>  Instruction {
                name:       "RES 5, H",
                opcode:     0xAC,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x20;
                    Ok(())
                },
            },
            0xAD    =>  Instruction {
                name:       "RES 5, L",
                opcode:     0xAD,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x20;
                    Ok(())
                },
            },
            0xAE    =>  Instruction {
                name:       "RES 5, (HL)",
                opcode:     0xAE,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x20);
                    Ok(())
                },
            },
            0xAF    =>  Instruction {
                name:       "RES 5, A",
                opcode:     0xAF,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x20;
                    Ok(())
                },
            },
            0xB0    =>  Instruction {
                name:       "RES 6, B",
                opcode:     0xB0,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x40;
                    Ok(())
                },
            },
            0xB1    =>  Instruction {
                name:       "RES 6, C",
                opcode:     0xB1,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x40;
                    Ok(())
                },
            },
            0xB2    =>  Instruction {
                name:       "RES 6, D",
                opcode:     0xB2,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x40;
                    Ok(())
                },
            },
            0xB3    =>  Instruction {
                name:       "RES 6, E",
                opcode:     0xB3,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x40;
                    Ok(())
                },
            },
            0xB4    =>  Instruction {
                name:       "RES 6, H",
                opcode:     0xB4,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x40;
                    Ok(())
                },
            },
            0xB5    =>  Instruction {
                name:       "RES 6, L",
                opcode:     0xB5,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x40;
                    Ok(())
                },
            },
            0xB6    =>  Instruction {
                name:       "RES 6, (HL)",
                opcode:     0xB6,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x40);
                    Ok(())
                },
            },
            0xB7    =>  Instruction {
                name:       "RES 6, A",
                opcode:     0xB7,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x40;
                    Ok(())
                },
            },
            0xB8    =>  Instruction {
                name:       "RES 7, B",
                opcode:     0xB8,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b &= !0x80;
                    Ok(())
                },
            },
            0xB9    =>  Instruction {
                name:       "RES 7, C",
                opcode:     0xB9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c &= !0x80;
                    Ok(())
                },
            },
            0xBA    =>  Instruction {
                name:       "RES 7, D",
                opcode:     0xBA,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d &= !0x80;
                    Ok(())
                },
            },
            0xBB    =>  Instruction {
                name:       "RES 7, E",
                opcode:     0xBB,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e &= !0x80;
                    Ok(())
                },
            },
            0xBC    =>  Instruction {
                name:       "RES 7, H",
                opcode:     0xBC,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h &= !0x80;
                    Ok(())
                },
            },
            0xBD    =>  Instruction {
                name:       "RES 7, L",
                opcode:     0xBD,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l &= !0x80;
                    Ok(())
                },
            },
            0xBE    =>  Instruction {
                name:       "RES 7, (HL)",
                opcode:     0xBE,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) & !0x80);
                    Ok(())
                },
            },
            0xBF    =>  Instruction {
                name:       "RES 3, A",
                opcode:     0xBF,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a &= !0x80;
                    Ok(())
                },
            },
            0xC0    =>  Instruction {
                name:       "SET 0, B",
                opcode:     0xC0,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x01;
                    Ok(())
                },
            },
            0xC1    =>  Instruction {
                name:       "SET 0, C",
                opcode:     0xC1,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x01;
                    Ok(())
                },
            },
            0xC2    =>  Instruction {
                name:       "SET 0, D",
                opcode:     0xC2,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x01;
                    Ok(())
                },
            },
            0xC3    =>  Instruction {
                name:       "SET 0, E",
                opcode:     0xC3,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x01;
                    Ok(())
                },
            },
            0xC4    =>  Instruction {
                name:       "SET 0, H",
                opcode:     0xC4,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x01;
                    Ok(())
                },
            },
            0xC5    =>  Instruction {
                name:       "SET 0, L",
                opcode:     0xC5,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x01;
                    Ok(())
                },
            },
            0xC6    =>  Instruction {
                name:       "SET 0, (HL)",
                opcode:     0xC6,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x01);
                    Ok(())
                },
            },
            0xC7    =>  Instruction {
                name:       "SET 0, A",
                opcode:     0xC7,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x01;
                    Ok(())
                },
            },
            0xC8    =>  Instruction {
                name:       "SET 1, B",
                opcode:     0xC8,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x02;
                    Ok(())
                },
            },
            0xC9    =>  Instruction {
                name:       "SET 1, C",
                opcode:     0xC9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x02;
                    Ok(())
                },
            },
            0xCA    =>  Instruction {
                name:       "SET 1, D",
                opcode:     0xCA,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x02;
                    Ok(())
                },
            },
            0xCB    =>  Instruction {
                name:       "SET 1, E",
                opcode:     0xCB,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x02;
                    Ok(())
                },
            },
            0xCC    =>  Instruction {
                name:       "SET 1, H",
                opcode:     0xCC,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x02;
                    Ok(())
                },
            },
            0xCD    =>  Instruction {
                name:       "SET 1, L",
                opcode:     0xCD,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x02;
                    Ok(())
                },
            },
            0xCE    =>  Instruction {
                name:       "SET 1, (HL)",
                opcode:     0xCE,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x02);
                    Ok(())
                },
            },
            0xCF    =>  Instruction {
                name:       "SET 1, A",
                opcode:     0xCF,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x02;
                    Ok(())
                },
            },
            0xD0    =>  Instruction {
                name:       "SET 2, B",
                opcode:     0xD0,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x04;
                    Ok(())
                },
            },
            0xD1    =>  Instruction {
                name:       "SET 2, C",
                opcode:     0xD1,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x04;
                    Ok(())
                },
            },
            0xD2    =>  Instruction {
                name:       "SET 2, D",
                opcode:     0xD2,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x04;
                    Ok(())
                },
            },
            0xD3    =>  Instruction {
                name:       "SET 2, E",
                opcode:     0xD3,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x04;
                    Ok(())
                },
            },
            0xD4    =>  Instruction {
                name:       "SET 2, H",
                opcode:     0xD4,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x04;
                    Ok(())
                },
            },
            0xD5    =>  Instruction {
                name:       "SET 2, L",
                opcode:     0xD5,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x04;
                    Ok(())
                },
            },
            0xD6    =>  Instruction {
                name:       "SET 2, (HL)",
                opcode:     0xD6,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x04);
                    Ok(())
                },
            },
            0xD7    =>  Instruction {
                name:       "SET 2, A",
                opcode:     0xD7,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x04;
                    Ok(())
                },
            },
            0xD8    =>  Instruction {
                name:       "SET 3, B",
                opcode:     0xD8,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x08;
                    Ok(())
                },
            },
            0xD9    =>  Instruction {
                name:       "SET 3, C",
                opcode:     0xD9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x08;
                    Ok(())
                },
            },
            0xDA    =>  Instruction {
                name:       "SET 3, D",
                opcode:     0xDA,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x08;
                    Ok(())
                },
            },
            0xDB    =>  Instruction {
                name:       "SET 3, E",
                opcode:     0xDB,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x08;
                    Ok(())
                },
            },
            0xDC    =>  Instruction {
                name:       "SET 3, H",
                opcode:     0xDC,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x08;
                    Ok(())
                },
            },
            0xDD    =>  Instruction {
                name:       "SET 3, L",
                opcode:     0xDD,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x08;
                    Ok(())
                },
            },
            0xDE    =>  Instruction {
                name:       "SET 3, (HL)",
                opcode:     0xDE,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x08);
                    Ok(())
                },
            },
            0xDF    =>  Instruction {
                name:       "SET 3, A",
                opcode:     0xDF,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x08;
                    Ok(())
                },
            },
            0xE0    =>  Instruction {
                name:       "SET 4, B",
                opcode:     0xE0,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x10;
                    Ok(())
                },
            },
            0xE1    =>  Instruction {
                name:       "SET 4, C",
                opcode:     0xE1,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x10;
                    Ok(())
                },
            },
            0xE2    =>  Instruction {
                name:       "SET 4, D",
                opcode:     0xE2,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x10;
                    Ok(())
                },
            },
            0xE3    =>  Instruction {
                name:       "SET 4, E",
                opcode:     0xE3,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x10;
                    Ok(())
                },
            },
            0xE4    =>  Instruction {
                name:       "SET 4, H",
                opcode:     0xE4,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x10;
                    Ok(())
                },
            },
            0xE5    =>  Instruction {
                name:       "SET 4, L",
                opcode:     0xE5,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x10;
                    Ok(())
                },
            },
            0xE6    =>  Instruction {
                name:       "SET 4, (HL)",
                opcode:     0xE6,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x10);
                    Ok(())
                },
            },
            0xE7    =>  Instruction {
                name:       "SET 4, A",
                opcode:     0xE7,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x10;
                    Ok(())
                },
            },
            0xE8    =>  Instruction {
                name:       "SET 5, B",
                opcode:     0xE8,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x20;
                    Ok(())
                },
            },
            0xE9    =>  Instruction {
                name:       "SET 5, C",
                opcode:     0xE9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x20;
                    Ok(())
                },
            },
            0xEA    =>  Instruction {
                name:       "SET 5, D",
                opcode:     0xEA,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x20;
                    Ok(())
                },
            },
            0xEB    =>  Instruction {
                name:       "SET 5, E",
                opcode:     0xEB,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x20;
                    Ok(())
                },
            },
            0xEC    =>  Instruction {
                name:       "SET 5, H",
                opcode:     0xEC,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x20;
                    Ok(())
                },
            },
            0xED    =>  Instruction {
                name:       "SET 5, L",
                opcode:     0xED,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x20;
                    Ok(())
                },
            },
            0xEE    =>  Instruction {
                name:       "SET 5, (HL)",
                opcode:     0xEE,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x20);
                    Ok(())
                },
            },
            0xEF    =>  Instruction {
                name:       "SET 5, A",
                opcode:     0xEF,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x20;
                    Ok(())
                },
            },
            0xF0    =>  Instruction {
                name:       "SET 6, B",
                opcode:     0xF0,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x40;
                    Ok(())
                },
            },
            0xF1    =>  Instruction {
                name:       "SET 6, C",
                opcode:     0xF1,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x40;
                    Ok(())
                },
            },
            0xF2    =>  Instruction {
                name:       "SET 6, D",
                opcode:     0xF2,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x40;
                    Ok(())
                },
            },
            0xF3    =>  Instruction {
                name:       "SET 6, E",
                opcode:     0xF3,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x40;
                    Ok(())
                },
            },
            0xF4    =>  Instruction {
                name:       "SET 6, H",
                opcode:     0xF4,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x40;
                    Ok(())
                },
            },
            0xF5    =>  Instruction {
                name:       "SET 6, L",
                opcode:     0xF5,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x40;
                    Ok(())
                },
            },
            0xF6    =>  Instruction {
                name:       "SET 6, (HL)",
                opcode:     0xF6,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x40);
                    Ok(())
                },
            },
            0xF7    =>  Instruction {
                name:       "SET 6, A",
                opcode:     0xF7,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x40;
                    Ok(())
                },
            },
            0xF8    =>  Instruction {
                name:       "SET 7, B",
                opcode:     0xF8,
                cycles:     8,
                operation:  |cpu| {
                    cpu.b |= 0x80;
                    Ok(())
                },
            },
            0xF9    =>  Instruction {
                name:       "SET 7, C",
                opcode:     0xF9,
                cycles:     8,
                operation:  |cpu| {
                    cpu.c |= 0x80;
                    Ok(())
                },
            },
            0xFA    =>  Instruction {
                name:       "SET 7, D",
                opcode:     0xFA,
                cycles:     8,
                operation:  |cpu| {
                    cpu.d |= 0x80;
                    Ok(())
                },
            },
            0xFB    =>  Instruction {
                name:       "SET 7, E",
                opcode:     0xFB,
                cycles:     8,
                operation:  |cpu| {
                    cpu.e |= 0x80;
                    Ok(())
                },
            },
            0xFC    =>  Instruction {
                name:       "SET 7, H",
                opcode:     0xFC,
                cycles:     8,
                operation:  |cpu| {
                    cpu.h |= 0x80;
                    Ok(())
                },
            },
            0xFD    =>  Instruction {
                name:       "SET 7, L",
                opcode:     0xFD,
                cycles:     8,
                operation:  |cpu| {
                    cpu.l |= 0x80;
                    Ok(())
                },
            },
            0xFE    =>  Instruction {
                name:       "SET 7, (HL)",
                opcode:     0xFE,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.bus.write8(addr, cpu.bus.read8(addr) | 0x80);
                    Ok(())
                },
            },
            0xFF    =>  Instruction {
                name:       "SET 3, A",
                opcode:     0xFF,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a |= 0x80;
                    Ok(())
                },
            },
        }
    }

    fn execute(&mut self, inst: &Instruction) {
        (inst.operation)(self).unwrap();
    }
}

struct Instruction {
    name:       &'static str,
    opcode:     u8,
    cycles:     u8,
    operation:  fn(cpu: &mut Cpu) -> Result<(), ()>,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instruction {{ name='{}', cycles={}, opcode=0x{:02x} }}",
            self.name, self.cycles, self.opcode)
    }
}

// tests

#[test]
fn test_ldbn() {    
    let mut cpu = Cpu::new();
    let opcode = 0x06;

    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(0x01, 42);
    cpu.tick();
    
    assert_eq!(cpu.b, 42);
    assert_eq!(cpu.decode(opcode).to_string(), 
            "Instruction { name='LD B, n', cycles=8, opcode=0x06 }")
}

#[test]
fn test_ldr1r2() {    
    let mut cpu = Cpu::new();
    let opcode = 0x7E;
    let addr = 0xFF;

    cpu.write_hl(addr);

    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(addr as usize, 42);
    cpu.tick();
    
    assert_eq!(cpu.a, 42);
    assert_eq!(cpu.decode(opcode).to_string(), 
            "Instruction { name='LD A, (HL)', cycles=8, opcode=0x7e }")
}

#[test]
fn test_push() {    
    let mut cpu = Cpu::new();
    let opcode = 0xF5;  // PUSH AF

    cpu.write_af(0xaadd);
    cpu.sp = 0xFF;

    cpu.bus.write8(0x00, opcode);
    cpu.tick();

    assert_eq!(cpu.bus.read8((cpu.sp+1) as usize), 0xaa);
    assert_eq!(cpu.bus.read8((cpu.sp) as usize), 0xdd);
    assert_eq!(cpu.decode(opcode).to_string(), 
            "Instruction { name=\'PUSH AF\', cycles=16, opcode=0xf5 }")
}

#[test]
fn test_pop() {    
    let mut cpu = Cpu::new();
    let op_push = 0xC5;     // PUSH BC
    let op_pop = 0xE1;      // POP hl

    cpu.sp = 0xFF;
    cpu.write_bc(0xaadd);
    
    cpu.bus.write8(0x00, op_push);
    cpu.bus.write8(0x01, op_pop);
    cpu.tick();
    cpu.tick();

    assert_eq!(cpu.h, 0xaa);
    assert_eq!(cpu.l, 0xdd);
    assert_eq!(cpu.decode(op_pop).to_string(), 
            "Instruction { name=\'POP HL\', cycles=12, opcode=0xe1 }")
}

#[test]
fn test_ldhl_sp_n() {    
    let mut cpu = Cpu::new();
    let opcode = 0xF8;      // LDHL SP, n
    let n = 0xFA;           // n = -6

    cpu.sp = 0x30;          // sp = 48
    
    cpu.bus.write8(0x00, opcode);   // hl = sp + n = 48 + (-6)
    cpu.bus.write8(0x01, n as u8);
    cpu.tick();

    assert_eq!(cpu.l, 0x2A);
    assert_eq!(cpu.decode(opcode).to_string(), 
            "Instruction { name=\'LDHL SP, n\', cycles=12, opcode=0xf8 }")
}

#[test]
fn test_addan() {    
    let mut cpu = Cpu::new();
    let opcode = 0x80;      // ADD A, B
    cpu.a = 32;
    cpu.b = 10;
    
    cpu.bus.write8(0x00, opcode);   // a = a + b
    cpu.tick();

    assert_eq!(cpu.a, 42);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);

    cpu.pc = 0;
    cpu.a = 0x08;
    cpu.b = 0x08;
    
    cpu.bus.write8(0x00, opcode);   // a = a + b
    cpu.tick();

    assert_eq!(cpu.a, 0x10);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, true);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
    
    cpu.pc = 0;
    cpu.a = 0x80;
    cpu.b = 0x80;
    
    cpu.bus.write8(0x00, opcode);   // a = a + b
    cpu.tick();

    assert_eq!(cpu.a, 0);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, true);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, true);
}

#[test]
fn test_adcan() {    
    let mut cpu = Cpu::new();
    let opcode = 0x88;      // ADC A, B
    cpu.a = 32;
    cpu.b = 9;
    cpu.f.insert(Flags::C);
    
    cpu.bus.write8(0x00, opcode);   // a = a + b + carry flag
    cpu.tick();

    assert_eq!(cpu.a, 42);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);

    cpu.pc = 0;
    cpu.a = 0x08;
    cpu.b = 0x07;
    cpu.f.insert(Flags::C);
    
    cpu.bus.write8(0x00, opcode);   // a = a + b + carry flag
    cpu.tick();

    assert_eq!(cpu.a, 0x10);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, true);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
    
    cpu.pc = 0;
    cpu.a = 0x80;
    cpu.b = 0x7F;
    cpu.f.insert(Flags::C);
    
    cpu.bus.write8(0x00, opcode);   // a = a + b + carry flag
    cpu.tick();

    assert_eq!(cpu.a, 0);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, true);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, true);
}

#[test]
fn test_suban() {    
    let mut cpu = Cpu::new();
    let opcode = 0x90;      // SUB A, B
    cpu.a = 0x0F;
    cpu.b = 0x0F;
    
    cpu.bus.write8(0x00, opcode);   // a = a - b
    cpu.tick();

    assert_eq!(cpu.a, 0x00);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, true);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);

    cpu.pc = 0;
    cpu.a = 0x20;
    cpu.b = 0x12;
    
    cpu.bus.write8(0x00, opcode);   // a = a - b
    cpu.tick();

    assert_eq!(cpu.a, 0x0E);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, true);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
    
    cpu.pc = 0;
    cpu.a = 0xE0;
    cpu.b = 0xF0;
    
    cpu.bus.write8(0x00, opcode);   // a = a - b
    cpu.tick();

    assert_eq!(cpu.a, 0xF0);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, true);
}

#[test]
fn test_sbcan() {    
    let mut cpu = Cpu::new();
    let opcode = 0x98;      // SBC A, B
    cpu.a = 0x0F;
    cpu.b = 0x0E;
    cpu.f.insert(Flags::C);
    
    cpu.bus.write8(0x00, opcode);   // a = a - b - carry flag
    cpu.tick();

    assert_eq!(cpu.a, 0x00);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, true);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);

    cpu.pc = 0;
    cpu.a = 0x20;
    cpu.b = 0x11;
    cpu.f.insert(Flags::C);
    
    cpu.bus.write8(0x00, opcode);   // a = a - b - carry flag
    cpu.tick();

    assert_eq!(cpu.a, 0x0E);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, true);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
    
    cpu.pc = 0;
    cpu.a = 0xE0;
    cpu.b = 0xEF;
    cpu.f.insert(Flags::C);
    
    cpu.bus.write8(0x00, opcode);   // a = a - b - carry flag
    cpu.tick();

    assert_eq!(cpu.a, 0xF0);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, true);
}

#[test]
fn test_and() {    
    let mut cpu = Cpu::new();
    let opcode = 0xA0;      // AND A, B
    cpu.a = 0b1111_1010;
    cpu.b = 0b0000_1111;
    
    cpu.bus.write8(0x00, opcode);   // a = a & b
    cpu.tick();

    assert_eq!(cpu.a, 0b0000_1010);
}

#[test]
fn test_or() {    
    let mut cpu = Cpu::new();
    let opcode = 0xB0;      // OR A, B
    cpu.a = 0b1011_0000;
    cpu.b = 0b0000_1101;
    
    cpu.bus.write8(0x00, opcode);   // a = a | b
    cpu.tick();

    assert_eq!(cpu.a, 0b1011_1101);
}

#[test]
fn test_xor() {    
    let mut cpu = Cpu::new();
    let opcode = 0xA8;      // XOR A, B
    cpu.a = 0b1010_0000;
    cpu.b = 0b0000_0011;
    
    cpu.bus.write8(0x00, opcode);   // a = a ^ b
    cpu.tick();

    assert_eq!(cpu.a, 0b1010_0011);
}

#[test]
fn test_cp() {    
    let mut cpu = Cpu::new();
    let opcode = 0xB8;      // CP A, B
    cpu.a = 0b0000_1111;
    cpu.b = 0b0000_1111;
    
    cpu.bus.write8(0x00, opcode);   // a == b
    cpu.tick();

    assert_eq!(cpu.a, 0b0000_1111);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, true);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
}

#[test]
fn test_inc() {    
    let mut cpu = Cpu::new();
    let opcode = 0x3C;      // INC A
    cpu.a = 0;
    
    cpu.bus.write8(0x00, opcode);   // a += 1
    cpu.tick();

    assert_eq!(cpu.a, 1);
}

#[test]
fn test_dec() {    
    let mut cpu = Cpu::new();
    let opcode = 0x3D;      // DEC A
    cpu.a = 0;
    
    cpu.bus.write8(0x00, opcode);   // a += 1
    cpu.tick();

    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn test_addhln() {    
    let mut cpu = Cpu::new();
    let opcode = 0x09;      // ADD HL, BC
    cpu.write_hl(0xFFF0);
    cpu.write_bc(0x10);
    
    cpu.bus.write8(0x00, opcode);   // a = hl + bc
    cpu.tick();

    assert_eq!(cpu.read_hl(), 0x00);
}

#[test]
fn test_addspn() {    
    let mut cpu = Cpu::new();
    let opcode = 0xE8;      // ADD SP, #
    cpu.sp = 0xFFF0;
    
    cpu.bus.write8(0x00, opcode);   // a = sp + #
    cpu.bus.write8(0x01, 0x10);
    cpu.tick();

    assert_eq!(cpu.sp, 0x00);
}

#[test]
fn test_incnn() {    
    let mut cpu = Cpu::new();
    let opcode = 0x03;      // INC BC
    cpu.write_bc(0xFFF0);
    
    cpu.bus.write8(0x00, opcode);   // a = bc + 1
    cpu.tick();

    assert_eq!(cpu.read_bc(), 0xFFF1);
}

#[test]
fn test_decnn() {    
    let mut cpu = Cpu::new();
    let opcode = 0x0B;      // DEC BC
    cpu.write_bc(0xFFF0);
    
    cpu.bus.write8(0x00, opcode);   // a = bc - 1
    cpu.tick();

    assert_eq!(cpu.read_bc(), 0xFFEF);
}

#[test]
fn test_rlca() {    
    let mut cpu = Cpu::new();
    let opcode = 0x07;      // RLCA
    cpu.a = 0b1001_1001;
    
    cpu.bus.write8(0x00, opcode);   // a = a.rotate_shift(1)
    cpu.tick();

    assert_eq!(cpu.a, 0b0011_0011);
}

#[test]
fn test_rla() {    
    let mut cpu = Cpu::new();
    let opcode = 0x17;      // RLA
    cpu.a = 0b1001_1001;
    
    cpu.bus.write8(0x00, opcode);   // a = a.rotate_shift(1)
    cpu.tick();

    assert_eq!(cpu.a, 0b0011_0010);
}

#[test]
fn test_rrca() {    
    let mut cpu = Cpu::new();
    let opcode = 0x0F;      // RRCA
    cpu.a = 0b1001_1001;
    
    cpu.bus.write8(0x00, opcode);   // a = a.rotate_right(1)
    cpu.tick();

    assert_eq!(cpu.a, 0b1100_1100);
}

#[test]
fn test_rra() {    
    let mut cpu = Cpu::new();
    let opcode = 0x1F;      // RRA
    cpu.a = 0b1001_1001;
    
    cpu.bus.write8(0x00, opcode);   // a = a.rotate_right(1)
    cpu.tick();

    assert_eq!(cpu.a, 0b0100_1100);
}

#[test]
fn test_rlcb() {    
    let mut cpu = Cpu::new();
    let opcode = 0x00;      // RLC B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b.rotate_left(1)
    cpu.tick();

    assert_eq!(cpu.b, 0b0011_0011);
}

#[test]
fn test_rlb() {    
    let mut cpu = Cpu::new();
    let opcode = 0x10;      // RL B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b.rotate_left(1)
    cpu.tick();

    assert_eq!(cpu.b, 0b0011_0010);
}

#[test]
fn test_rrc() {    
    let mut cpu = Cpu::new();
    let opcode = 0x08;      // RRC B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b.rotate_right(1)
    cpu.tick();

    assert_eq!(cpu.b, 0b0100_1100);
}

#[test]
fn test_rrn() {    
    let mut cpu = Cpu::new();
    let opcode = 0x18;      // RR B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b.rotate_right(1)
    cpu.tick();

    assert_eq!(cpu.b, 0b0100_1100);
}

#[test]
fn test_slan() {    
    let mut cpu = Cpu::new();
    let opcode = 0x18;      // SLA B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b << 1
    cpu.tick();

    assert_eq!(cpu.b, 0b0100_1100);
}

#[test]
fn test_sran() {    
    let mut cpu = Cpu::new();
    let opcode = 0x28;      // SRA B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b >> 1
    cpu.tick();

    assert_eq!(cpu.b, 0b1100_1100);
}

#[test]
fn test_srln() {    
    let mut cpu = Cpu::new();
    let opcode = 0x38;      // SRL B
    cpu.b = 0b1001_1001;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b = b >> 1
    cpu.tick();

    assert_eq!(cpu.b, 0b0100_1100);
}

#[test]
fn test_bitbr() {    
    let mut cpu = Cpu::new();
    let opcode = 0x47;      // BIT 0, A
    cpu.a = 0b0000_0000;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // if b & 0x01 == 0 { Flags::Z = 0; }
    cpu.tick();

    assert_eq!(cpu.f & Flags::Z == Flags::Z, true);
}

#[test]
fn test_setbr() {    
    let mut cpu = Cpu::new();
    let opcode = 0xC0;      // SET 0, B
    cpu.a = 0b0000_0000;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b |= 0x01
    cpu.tick();

    assert_eq!(cpu.b, 0x01);
}

#[test]
fn test_resbr() {    
    let mut cpu = Cpu::new();
    let opcode = 0xA0;      // RES 4, B
    cpu.b = 0b1111_1111;
    
    cpu.bus.write8(0x00, 0xCB);
    cpu.bus.write8(0x01, opcode);   // b &= !0x10
    cpu.tick();

    assert_eq!(cpu.b, 0b1110_1111);
}

#[test]
fn test_jpnn() {    
    let mut cpu = Cpu::new();
    let opcode = 0xC3;      // JP nn
    
    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(0x01, 0x12);
    cpu.bus.write8(0x02, 0x34);
    cpu.tick();

    assert_eq!(cpu.pc, 0x3412);
}

#[test]
fn test_jpccnn() {    
    let mut cpu = Cpu::new();
    let opcode = 0xC2;      // JP NZ, nn

    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(0x01, 0x12);
    cpu.bus.write8(0x02, 0x34);
    cpu.tick();

    assert_eq!(cpu.pc, 0x3412);
}

#[test]
fn test_jphl() {    
    let mut cpu = Cpu::new();
    let opcode = 0xE9;      // JP (HL)

    cpu.write_hl(0x1234);

    cpu.bus.write8(0x00, opcode);
    cpu.tick();

    assert_eq!(cpu.pc, 0x1234);
}

#[test]
fn test_jre() {    
    let mut cpu = Cpu::new();
    let opcode = 0x18;      // JR e

    cpu.write_hl(0x1234);

    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(0x01, -2 as i8 as u8);
    cpu.tick();

    assert_eq!(cpu.pc, 0x00);
}

#[test]
fn test_jrcce() {    
    let mut cpu = Cpu::new();
    let opcode = 0x20;      // JR NZ e

    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(0x01, -2 as i8 as u8);
    cpu.tick();

    assert_eq!(cpu.pc, 0x00);
}

#[test]
fn test_callnn() {    
    let mut cpu = Cpu::new();
    let opcode = 0xCD;      // CALL nn
    cpu.sp = 0x100;

    cpu.bus.write8(0x00, opcode);
    cpu.bus.write8(0x01, 0x12);
    cpu.bus.write8(0x02, 0x34);
    cpu.tick();

    assert_eq!(cpu.pc, 0x3412);
    assert_eq!(cpu.sp, 0x00FE);
    assert_eq!(cpu.bus.read8(cpu.sp as usize), 0x03);
}

#[test]
fn test_rstn() {    
    let mut cpu = Cpu::new();
    let opcode = 0xFF;      // RST 0x38
    cpu.sp = 0x100;

    cpu.bus.write8(0x00, opcode);
    cpu.tick();

    assert_eq!(cpu.pc, 0x0038);
    assert_eq!(cpu.sp, 0x00FE);
    let lo = cpu.bus.read8(cpu.sp as usize) as u16;
    let hi = (cpu.bus.read8((cpu.sp+1) as usize) as u16) << 8 ;
    assert_eq!(hi | lo, 0x01);
    // assert_eq!(cpu.bus.read16(cpu.sp as usize), 0x01);
}

#[test]
fn test_ret() {    
    let mut cpu = Cpu::new();
    let opcode1 = 0xC5;     // PUSH BC
    let opcode2 = 0xC9;     // RET
    cpu.sp = 0x100;
    cpu.write_bc(0x1234);

    cpu.bus.write8(0x00, opcode1);
    cpu.bus.write8(0x01, opcode2);
    cpu.tick();
    cpu.tick();

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0x0100);
}

#[test]
fn test_retcc() {    
    let mut cpu = Cpu::new();
    let opcode1 = 0xC5;     // PUSH BC
    let opcode2 = 0xC0;     // RET NZ
    cpu.sp = 0x100;
    cpu.write_bc(0x1234);

    cpu.bus.write8(0x00, opcode1);
    cpu.bus.write8(0x01, opcode2);
    cpu.tick();
    cpu.tick();

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0x0100);
}

#[test]
fn test_reti() {    
    let mut cpu = Cpu::new();
    let opcode1 = 0xC5;     // PUSH BC
    let opcode2 = 0xD9;     // RETI
    cpu.sp = 0x100;
    cpu.write_bc(0x1234);

    cpu.bus.write8(0x00, opcode1);
    cpu.bus.write8(0x01, opcode2);
    cpu.tick();
    cpu.tick();

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0x0100);
    assert_eq!(cpu.bus.read8(0xFFFF as usize), 0b11111)
}