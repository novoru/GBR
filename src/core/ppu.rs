#[macro_use]
use bitflags::*;

use crate::core::io::Io;
use crate::core::ram::Ram;

bitflags! {
    struct Lcdc: u8 {
        const LCD_EN    = 0b10000000;
        const WIN_MAP   = 0b01000000;
        const WIN_EN    = 0b00100000;
        const TILE_SEL  = 0b00010000;
        const BG_MAP    = 0b00001000;
        const OBJ_SIZE  = 0b00000100;
        const OBJ_EN    = 0b00000010;
        const BG_EN     = 0b00000001;
    }
}

bitflags! {
    struct Stat: u8 {
        const _BIT7         = 0b10000000;
        const INTR_LYC      = 0b01000000;
        const INTR_M2       = 0b00100000;
        const INTR_M1       = 0b00010000;
        const INTR_M0       = 0b00001000;
        const LYC_STAT      = 0b00000100;
        const MODE_FLAG1    = 0b00000010;
        const MODE_FLAG0    = 0b00000001;
    }
}

#[derive(Debug)]
pub enum Color {
    Darkest     = 0,
    Dark        = 1,
    Light       = 2,
    Lightest    = 3,
}

impl From<u8> for Color {
    fn from(item: u8) -> Self {
        match item {
            0x00    =>  Color::Darkest,
            0x01    =>  Color::Dark,
            0x02    =>  Color::Light,
            0x03    =>  Color::Lightest,
            _       =>  panic!(),
        }
    }
}

impl Color {
    pub fn to_u8(&self) -> u8 {
        match self {
            Color::Darkest  =>  0,
            Color::Dark     =>  1,
            Color::Light    =>  2,
            Color::Lightest =>  3,
        }
    }
}

#[derive(Debug)]
struct Bgp {
    dot_11: Color,
    dot_10: Color,
    dot_01: Color,
    dot_00: Color,
}

impl From<u8> for Bgp {
    fn from(item: u8) -> Self {
        let dot_11 = Color::from((item >> 6) & 0x03);
        let dot_10 = Color::from((item >> 4) & 0x03);
        let dot_01 = Color::from((item >> 2) & 0x03);
        let dot_00 = Color::from(item & 0x03);

        Bgp {
            dot_11: dot_11,
            dot_10: dot_10,
            dot_01: dot_01,
            dot_00: dot_00,
        }
    }
}

impl Bgp {
    pub fn new() -> Self {
        Bgp {
            dot_11: Color::Darkest,
            dot_10: Color::Darkest,
            dot_01: Color::Darkest,
            dot_00: Color::Darkest,
        }
    }

    pub fn to_u8(&self) -> u8 {
        self.dot_11.to_u8() << 6 |
        self.dot_10.to_u8() << 4 |
        self.dot_01.to_u8() << 2 |
        self.dot_00.to_u8()
    }
}

enum PpuMode {
    VBlank,
    HBlank,
    SearchingOAM,
    TransferPixels,
}

pub const SCREEN_WIDTH:     usize   = 160;
pub const SCREEN_HEIGHT:    usize   = 144;
const LCD_BLANK_HEIGHT: u8 = 10;
const VRAM_SIZE:        usize   = 8192;
const OAM_SPRITES:      usize   = 40;
const FIFO_SIZE:        usize   = 8;
const LCDC_ADDR:        usize   = 0xFF40;
const STAT_ADDR:        usize   = 0xFF41;
const CYCLE_PER_LINE: u16 = 456;
const TILEMAP0_OFFSET: usize = 0x9800;
const TILEMAP1_OFFSET: usize = 0x9C00;
const TILEDATA0_OFFSET: usize = 0x8800;
const TILEDATA1_OFFSET: usize = 0x8000;

pub struct Ppu {
    clock: u16,
    pixels: [u8; SCREEN_WIDTH*SCREEN_HEIGHT],
    lcdc:   Lcdc,
    stat:   Stat,
    scy:    u8,
    scx:    u8,
    ly:     u8,
    lyc:    u8,
    dma:    u8,
    bgp:    Bgp,
    obp0:   u8,
    obp1:   u8,
    wy:     u8,
    wx:     u8,
    vram:   Ram,
    oam:    [Oam; OAM_SPRITES],
    oam_dma_started:    bool,
}

impl Io for Ppu {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            // 8kB Video RAM
            0x8000 ..= 0x9FFF   =>  self.vram.read8(addr&0x1FFF),
            // Sprite Attribute Memory (OAM)
            0xFE00 ..= 0xFE9F   =>  self.oam[addr&0xFF].read8(addr),
            // Registers
            0xFF40  =>  self.lcdc.bits,
            0xFF41  =>  self.stat.bits,
            0xFF42  =>  self.scy,
            0xFF43  =>  self.scx,
            0xFF44  =>  self.ly ,
            0xFF45  =>  self.lyc,
            0xFF46  =>  self.dma,
            0xFF47  =>  self.bgp.to_u8(),
            0xFF48  =>  self.obp0,
            0xFF49  =>  self.obp1,
            0xFF4A  =>  self.wy,
            0xFF4B  =>  self.wx,
            // ToDo: LCD Color Palettes (CGB only)
            // 0xFF68
            // 0xFF69
            // 0xFF6A
            _       =>  panic!(),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            // 8kB Video RAM
            0x8000 ..= 0x9FFF   =>  self.vram.write8(addr&0x1FFF, data),
            // Sprite Attribute Memory (OAM)
            0xFE00 ..= 0xFE9F   =>  self.oam[addr&0xFF].write8(addr, data),
            // Registers
            0xFF40  =>  self.lcdc   = Lcdc::from_bits_truncate(data),
            0xFF41  =>  self.stat   = Stat::from_bits_truncate(data),
            0xFF42  =>  self.scy    = data,
            0xFF43  =>  self.scx    = data,
            0xFF44  =>  self.ly     = data,
            0xFF45  =>  self.lyc    = data,
            0xFF46  =>  {
                self.dma    = data;
                self.oam_dma_started = true;
            },
            0xFF47  =>  self.bgp    = Bgp::from(data),
            0xFF48  =>  self.obp0   = 0,
            0xFF49  =>  self.obp1   = 0,
            0xFF4A  =>  self.wy     = data,
            0xFF4B  =>  self.wx     = data,
            // ToDo: LCD Color Palettes (CGB only)
            // 0xFF68
            // 0xFF69
            // 0xFF6A
            _       =>  panic!(),
        }
    }
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            clock: 0,
            pixels: [0; SCREEN_WIDTH*SCREEN_HEIGHT],
            lcdc:   Lcdc::from_bits_truncate(0x91),
            stat:   Stat::empty(),
            scy:    0,
            scx:    0,
            ly:     0,
            lyc:    0,
            dma:    0,
            bgp:    Bgp::new(),
            obp0:   0,
            obp1:   0,
            wy:     0,
            wx:     0,
            vram:   Ram::new(),
            oam:    [Oam::new(); OAM_SPRITES],
            oam_dma_started:    false,
        }
    }

    pub fn get_pixels(&self) -> [u8; SCREEN_WIDTH*SCREEN_HEIGHT] {
        self.pixels
    }

    pub fn tick(&mut self) {
        self.clock += 1;
        if !self.lcdc.contains(Lcdc::LCD_EN) {
            return;
        }

        self.update_mode();

        if self.clock >= CYCLE_PER_LINE {
            if self.ly == SCREEN_HEIGHT as u8 {

            } else if self.ly >= SCREEN_HEIGHT as u8 + LCD_BLANK_HEIGHT {
                self.ly = 0;
                self.draw_line();
            } else if self.ly < SCREEN_HEIGHT as u8 {
                self.draw_line();
                // if self.window_enabled() {
                    // self.build_window_tile();
                // }
            }

            if self.ly == self.lyc {
                self.stat.insert(Stat::LYC_STAT);
            } else {
                self.stat.remove(Stat::MODE_FLAG1);
                self.stat.remove(Stat::MODE_FLAG0);
            }
            self.ly += 1;
            self.clock -= CYCLE_PER_LINE;
        }
    }

    pub fn dma_started(&self) -> bool {
        self.oam_dma_started
    }

    pub fn stop_dma(&mut self) {
        self.oam_dma_started = false;
    }

    fn bg_tilemap_offset(&self) -> usize {
        match self.lcdc.contains(Lcdc::BG_MAP) {
            false   =>  TILEMAP0_OFFSET,
            true    =>  TILEMAP1_OFFSET
        }
    }

    fn tiledata_offset(&self) -> usize {
        match self.lcdc.contains(Lcdc::TILE_SEL) {
            false   =>  TILEDATA0_OFFSET,
            true    =>  TILEDATA1_OFFSET
        }
    }

    fn mode(&self) -> PpuMode {
        match self.read8(STAT_ADDR) & 0x03 {
            0x00    =>  PpuMode::HBlank,
            0x01    =>  PpuMode::VBlank,
            0x02    =>  PpuMode::SearchingOAM,
            0x03    =>  PpuMode::TransferPixels,
            _       =>  panic!(),
        }
    }

    fn switch_mode(&mut self, mode: PpuMode) {
        match mode {
            PpuMode::HBlank   =>  {
                self.stat.remove(Stat::MODE_FLAG1);
                self.stat.remove(Stat::MODE_FLAG0);
            },
            PpuMode::VBlank   =>  {
                self.stat.remove(Stat::MODE_FLAG1);
                self.stat.insert(Stat::MODE_FLAG0);
            },
            PpuMode::SearchingOAM   =>  {
                self.stat.insert(Stat::MODE_FLAG1);
                self.stat.remove(Stat::MODE_FLAG0);
            },
            PpuMode::TransferPixels   =>  {
                self.stat.insert(Stat::MODE_FLAG1);
                self.stat.insert(Stat::MODE_FLAG0);
            },
        }
    }

    fn update_mode(&mut self) {
        if self.ly > SCREEN_HEIGHT as u8 {
            self.switch_mode(PpuMode::VBlank);
        } else if self.clock <= 80 {
            self.switch_mode(PpuMode::SearchingOAM);
        } else if self.clock >= 167 && self.clock <= 291 {
            self.switch_mode(PpuMode::TransferPixels);
        } else {
            self.switch_mode(PpuMode::HBlank);
        }
    }

    fn _search_oam(&mut self) {
        // Find visible sprites
        /*
        oam.x != 0
        LY + 16 >= oam.y
        LY + 16 <  o am.y + h
        */
    }

    fn draw_line(&mut self) {
        self.build_bg();
    }

    fn build_bg(&mut self) {
        for x in 0..SCREEN_WIDTH as u8 {
            let y = self.ly.wrapping_add(self.scy) as u16 / 8 * 32;
            let index = x.wrapping_add(self.scx) as u16 / 8 % 32 + y;
            let tileid = self.get_tileid(index);
            let color = self.get_color(tileid, 
                                x.wrapping_add(self.scx)%8, 
                                self.ly.wrapping_add(self.scy)%8);
            let base = self.ly as usize * SCREEN_WIDTH + x as usize;
            self.pixels[base] = color;
        }
    }

    fn get_tileid(&self, index: u16) -> u8 {
        let addr = index as usize + self.bg_tilemap_offset();
        self.read8(addr)
    }

    fn get_color(&self, tileid: u8, x: u8, y: u8) -> u8 {
        let addr = tileid as usize * 0x10 + self.tiledata_offset();
        let mut pixels = Vec::new();

        for i in 0..8 {
            let line1 = self.read8(addr+i*2);
            let line2 = self.read8(addr+i*2+1);
            for j in 0..8 {
                let msb = line1 >> (7-j) & 0x01;
                let lsb = line2 >> (7-j) & 0x01;
                pixels.push(msb<<1+lsb);
            }
        }

        pixels[(x+y*8) as usize]
    }

}


bitflags! {
    struct OamFlags: u8 {
        const PRIORITY          = 0b10000000;
        const YFLIP             = 0b01000000;
        const XFLIP             = 0b00100000;
        const PALETTE_NO        = 0b00010000;
        const VRAM_BANK         = 0b00001000;
        const PALETTE_NO_BIT3   = 0b00000100;
        const PALETTE_NO_BIT2   = 0b00000010;
        const PALETTE_NO_BIT1   = 0b00000001;
    }
}

#[derive(Debug, Copy, Clone)]
struct Oam {
    y:      u8,
    x:      u8,
    tile:   u8,
    flags:  OamFlags,
}

impl Oam {
    pub fn new() -> Self {
        Oam {
            y:      0,
            x:      0,
            tile:   0,
            flags:  OamFlags::empty(),
        }
    }
}

impl Io for Oam {
    fn read8(&self, addr: usize) -> u8 {
        match addr & 0xFF {
            0x00    =>  self.y,
            0x01    =>  self.x,
            0x02    =>  self.tile,
            0x03    =>  self.flags.bits,
            _       =>  panic!(),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr & 0xFF {
            0x00    =>  self.y = data,
            0x01    =>  self.x = data,
            0x02    =>  self.tile = data,
            0x03    =>  self.flags = OamFlags::from_bits_truncate(data),
            _       =>  panic!(),
        }
    }
}

