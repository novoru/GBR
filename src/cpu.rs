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
        const _3    = 0b00001000;
        const _2    = 0b00000100;
        const _1    = 0b00000010;
        const _0    = 0b00000001;
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
    mmu:    Mmu,
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
        let cartridge = Cartridge::new();
        Cpu {
            a:      0,
            b:      0,
            d:      0,
            h:      0,
            c:      0,
            e:      0,
            l:      0,
            f:      Flags::empty(),
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

    fn read_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f.bits as u16
    }

    fn write_af(&mut self, data: u16) {
        self.a = (data >> 8) as u8;
        self.f = Flags::from_bits_truncate((data & 0xFF) as u8);
    }

    fn read_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    
    fn write_bc(&mut self, data: u16) {
        self.b = (data >> 8) as u8;
        self.c = (data & 0xFF) as u8;
    }
    
    fn read_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    
    fn write_de(&mut self, data: u16) {
        self.d = (data >> 8) as u8;
        self.e = (data & 0xFF) as u8;
    }
    
    fn read_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    
    fn write_hl(&mut self, data: u16) {
        self.h = (data >> 8) as u8;
        self.l = (data & 0xFF) as u8;
    }

    fn push(&mut self, data: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.mmu.write8(self.sp as usize, data);
    }

    fn pop(&mut self) -> u8 {
        let addr = self.sp;
        self.sp = addr.wrapping_add(1);
        self.mmu.read8(addr as usize)
    }

    fn decode(&mut self, opcode: u8) -> Instruction {
        match opcode {
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
                    cpu.mmu.write8(addr, cpu.a);
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
            
            0x08    =>  Instruction {
                name:       "LD (nn), SP",
                opcode:     0x08,
                cycles:     20,
                operation:  |cpu| {
                    let addr = cpu.fetch16() as usize;
                    cpu.mmu.write16(addr, cpu.sp);
                    Ok(())
                },
            },

            0x0A    =>  Instruction {
                name:       "LD A, (BC)",
                opcode:     0x0A,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a = cpu.mmu.read8(cpu.read_bc() as usize);
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
                    cpu.mmu.write8(addr, cpu.a);
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

            0x1A    =>  Instruction {
                name:       "LD A, (DE)",
                opcode:     0x1A,
                cycles:     8,
                operation:  |cpu| {
                    cpu.a = cpu.mmu.read8(cpu.read_de() as usize);
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
                    cpu.mmu.write8(addr as usize, cpu.a);
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
            
            0x2A    =>  Instruction {
                name:       "LDI A, (HL)",
                opcode:     0x2A,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl();
                    cpu.write_hl(addr.wrapping_add(1));
                    cpu.a = cpu.mmu.read8(addr as usize);
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
                    cpu.mmu.write8(addr as usize, cpu.a);
                    Ok(())
                },
            },
            
            0x34    =>  Instruction {
                name:       "INC (HL)",
                opcode:     0x34,
                cycles:     12,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    let n = cpu.mmu.read8(addr);
                    cpu.mmu.write8(addr, n.wrapping_add(1));
                    if cpu.mmu.read8(addr) == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.mmu.read8(addr)^n^1)&0x10 == 0x10 {
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
                    let n = cpu.mmu.read8(addr);
                    cpu.mmu.write8(addr, n.wrapping_sub(1));
                    if cpu.b == 0 {
                        cpu.f.insert(Flags::Z);
                    } else {
                        cpu.f.remove(Flags::Z);
                    }
                    cpu.f.remove(Flags::N);
                    if (cpu.mmu.read8(addr)^n^1)&0x10 == 0x10 {
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
                    cpu.mmu.write8(cpu.read_hl() as usize, n);
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
                    cpu.a = cpu.mmu.read8(addr as usize);
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
                    cpu.b = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    cpu.c = cpu.mmu.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            // 0x4F
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
                    cpu.d = cpu.mmu.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            // 0x57
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
                    cpu.e = cpu.mmu.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            // 0x5F
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
                    cpu.h = cpu.mmu.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            // 0x67
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
                    cpu.l = cpu.mmu.read8(cpu.read_hl() as usize);
                    Ok(())
                },
            },
            // 0x6F
            0x70    =>  Instruction {
                name:       "LD (HL), B",
                opcode:     0x70,
                cycles:     4,
                operation:  |cpu| {
                    cpu.mmu.write8(cpu.read_hl() as usize, cpu.b);
                    Ok(())
                },
            },
            0x71    =>  Instruction {
                name:       "LD (HL), C",
                opcode:     0x71,
                cycles:     4,
                operation:  |cpu| {
                    cpu.mmu.write8(cpu.read_hl() as usize, cpu.c);                    
                    Ok(())
                },
            },
            0x72    =>  Instruction {
                name:       "LD (HL), D",
                opcode:     0x62,
                cycles:     4,
                operation:  |cpu| {
                    cpu.mmu.write8(cpu.read_hl() as usize, cpu.d);
                    Ok(())
                },
            },
            0x73    =>  Instruction {
                name:       "LD (HL), E",
                opcode:     0x73,
                cycles:     4,
                operation:  |cpu| {
                    cpu.mmu.write8(cpu.read_hl() as usize, cpu.e);
                    Ok(())
                },
            },
            0x74    =>  Instruction {
                name:       "LD (HL), H",
                opcode:     0x74,
                cycles:     4,
                operation:  |cpu| {
                    cpu.mmu.write8(cpu.read_hl() as usize, cpu.h);
                    Ok(())
                },
            },
            0x75    =>  Instruction {
                name:       "LD (HL), L",
                opcode:     0x75,
                cycles:     4,
                operation:  |cpu| {
                    cpu.mmu.write8(cpu.read_hl() as usize, cpu.l);
                    Ok(())
                },
            },

            
            0x77    =>  Instruction {
                name:       "LD (HL), A",
                opcode:     0x77,
                cycles:     8,
                operation:  |cpu| {
                    let addr = cpu.read_hl() as usize;
                    cpu.mmu.write8(addr, cpu.a);
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
                    cpu.a = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize).wrapping_add(c);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize).wrapping_add(c);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize);
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
                    let n = cpu.mmu.read8(cpu.read_hl() as usize);
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

            0xE0    =>  Instruction {
                name:       "LDH (n), A",
                opcode:     0xE0,
                cycles:     12,
                operation:  |cpu| {
                    let addr = 0xFF00 + (cpu.fetch16() as usize);
                    cpu.mmu.write8(addr, cpu.a);
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
                    cpu.mmu.write8(addr, cpu.a);
                    Ok(())
                },
            },

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

            0xEA    =>  Instruction {
                name:       "LD (nn), A",
                opcode:     0xEA,
                cycles:     16,
                operation:  |cpu| {
                    let addr = cpu.fetch16() as usize;
                    cpu.mmu.write8(addr, cpu.a);
                    Ok(())
                },
            },
            
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
            
            0xF0    =>  Instruction {
                name:       "LDH A, (n)",
                opcode:     0xF0,
                cycles:     12,
                operation:  |cpu| {
                    let addr = 0xFF00 + (cpu.fetch16() as usize);
                    cpu.a = cpu.mmu.read8(addr);
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
                    cpu.a = cpu.mmu.read8(addr);
                    Ok(())
                },
            },
            
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
                    cpu.a = cpu.mmu.read8(addr);
                    Ok(())
                },
            },

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

            _       =>  unimplemented!("can't decode: 0x{:02x}", opcode),
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

    cpu.mmu.write8(0x00, opcode);
    cpu.mmu.write8(0x01, 42);
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

    cpu.mmu.write8(0x00, opcode);
    cpu.mmu.write8(addr as usize, 42);
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

    cpu.mmu.write8(0x00, opcode);
    cpu.tick();

    assert_eq!(cpu.mmu.read8((cpu.sp+1) as usize), 0xaa);
    assert_eq!(cpu.mmu.read8((cpu.sp) as usize), 0xdd);
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
    
    cpu.mmu.write8(0x00, op_push);
    cpu.mmu.write8(0x01, op_pop);
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
    
    cpu.mmu.write8(0x00, opcode);   // hl = sp + n = 48 + (-6)
    cpu.mmu.write8(0x01, n as u8);
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a + b
    cpu.tick();

    assert_eq!(cpu.a, 42);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);

    cpu.pc = 0;
    cpu.a = 0x08;
    cpu.b = 0x08;
    
    cpu.mmu.write8(0x00, opcode);   // a = a + b
    cpu.tick();

    assert_eq!(cpu.a, 0x10);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, false);
    assert_eq!((cpu.f & Flags::H) == Flags::H, true);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
    
    cpu.pc = 0;
    cpu.a = 0x80;
    cpu.b = 0x80;
    
    cpu.mmu.write8(0x00, opcode);   // a = a + b
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a + b + carry flag
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a + b + carry flag
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a + b + carry flag
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a - b
    cpu.tick();

    assert_eq!(cpu.a, 0x00);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, true);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, false);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);

    cpu.pc = 0;
    cpu.a = 0x20;
    cpu.b = 0x12;
    
    cpu.mmu.write8(0x00, opcode);   // a = a - b
    cpu.tick();

    assert_eq!(cpu.a, 0x0E);
    assert_eq!((cpu.f & Flags::Z) == Flags::Z, false);
    assert_eq!((cpu.f & Flags::N) == Flags::N, true);
    assert_eq!((cpu.f & Flags::H) == Flags::H, true);
    assert_eq!((cpu.f & Flags::C) == Flags::C, false);
    
    cpu.pc = 0;
    cpu.a = 0xE0;
    cpu.b = 0xF0;
    
    cpu.mmu.write8(0x00, opcode);   // a = a - b
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a - b - carry flag
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a - b - carry flag
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a - b - carry flag
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
    
    cpu.mmu.write8(0x00, opcode);   // a = a & b
    cpu.tick();

    assert_eq!(cpu.a, 0b0000_1010);
}

#[test]
fn test_or() {    
    let mut cpu = Cpu::new();
    let opcode = 0xB0;      // OR A, B
    cpu.a = 0b1011_0000;
    cpu.b = 0b0000_1101;
    
    cpu.mmu.write8(0x00, opcode);   // a = a | b
    cpu.tick();

    assert_eq!(cpu.a, 0b1011_1101);
}

#[test]
fn test_xor() {    
    let mut cpu = Cpu::new();
    let opcode = 0xA8;      // XOR A, B
    cpu.a = 0b1010_0000;
    cpu.b = 0b0000_0011;
    
    cpu.mmu.write8(0x00, opcode);   // a = a ^ b
    cpu.tick();

    assert_eq!(cpu.a, 0b1010_0011);
}

#[test]
fn test_cp() {    
    let mut cpu = Cpu::new();
    let opcode = 0xB8;      // CP A, B
    cpu.a = 0b0000_1111;
    cpu.b = 0b0000_1111;
    
    cpu.mmu.write8(0x00, opcode);   // a == b
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
    
    cpu.mmu.write8(0x00, opcode);   // a += 1
    cpu.tick();

    assert_eq!(cpu.a, 1);
}

#[test]
fn test_dec() {    
    let mut cpu = Cpu::new();
    let opcode = 0x3D;      // DEC A
    cpu.a = 0;
    
    cpu.mmu.write8(0x00, opcode);   // a += 1
    cpu.tick();

    assert_eq!(cpu.a, 0xFF);
}