use bitflags::*;

use crate::core::io::Io;
use crate::core::ram::Ram;
use crate::core::interrupt::InterruptKind;

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
    Darkest     = 3,
    Dark        = 2,
    Light       = 1,
    Lightest    = 0,
}

impl From<u8> for Color {
    fn from(item: u8) -> Self {
        match item {
            0x03    =>  Color::Darkest,
            0x02    =>  Color::Dark,
            0x01    =>  Color::Light,
            0x00    =>  Color::Lightest,
            _       =>  panic!(),
        }
    }
}

impl Color {
    pub fn to_u8(&self) -> u8 {
        match self {
            Color::Darkest  =>  3,
            Color::Dark     =>  2,
            Color::Light    =>  1,
            Color::Lightest =>  0,
        }
    }
}

#[derive(Debug)]
struct Palette {
    dot_11: Color,
    dot_10: Color,
    dot_01: Color,
    dot_00: Color,
}

impl From<u8> for Palette {
    fn from(item: u8) -> Self {
        let dot_11 = Color::from((item >> 6) & 0x03);
        let dot_10 = Color::from((item >> 4) & 0x03);
        let dot_01 = Color::from((item >> 2) & 0x03);
        let dot_00 = Color::from(item & 0x03);

        Palette {
            dot_11: dot_11,
            dot_10: dot_10,
            dot_01: dot_01,
            dot_00: dot_00,
        }
    }
}

impl Palette {
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
// const VRAM_SIZE:        usize   = 8192;
const OAM_SPRITES:      usize   = 40;
// const OAM_OFFSET:       usize   = 0xFE00;
// const LCDC_ADDR:        usize   = 0xFF40;
// const STAT_ADDR:        usize   = 0xFF41;
const CLOCKS_PER_LINE: u16 = 456;
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
    bgp:    Palette,
    obp0:   Palette,
    obp1:   Palette,
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
            0xFE00 ..= 0xFE9F   =>  self.oam[(addr&0xFF)/4].read8(addr%4),
            // Registers
            0xFF40  =>  self.lcdc.bits,
            0xFF41  =>  self.stat.bits,
            0xFF42  =>  self.scy,
            0xFF43  =>  self.scx,
            0xFF44  =>  self.ly ,
            0xFF45  =>  self.lyc,
            0xFF46  =>  self.dma,
            0xFF47  =>  self.bgp.to_u8(),
            0xFF48  =>  self.obp0.to_u8(),
            0xFF49  =>  self.obp1.to_u8(),
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
            0xFE00 ..= 0xFE9F   =>  self.oam[(addr&0xFF)/4].write8(addr%4, data),
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
            0xFF47  =>  self.bgp    = Palette::from(data),
            0xFF48  =>  self.obp0   = Palette::from(data),
            0xFF49  =>  self.obp1   = Palette::from(data),
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
            bgp:    Palette::from(0xFC),
            obp0:   Palette::from(0xFF),
            obp1:   Palette::from(0xFF),
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

    pub fn tick(&mut self) -> (Option<InterruptKind>, Option<InterruptKind>) {
        let mut vblank_irq = false;
        let mut lcdc_irq = self.update_mode();
        self.clock = self.clock.wrapping_add(4);

        if self.clock >= CLOCKS_PER_LINE {
            if self.ly == SCREEN_HEIGHT as u8 {
                vblank_irq = true;
                if self.sprite_on() {
                    self.build_sprite();
                }
                if self.stat.contains(Stat::INTR_M1) {
                    lcdc_irq = true;
                }
            } else if self.ly >= (SCREEN_HEIGHT as u8 + LCD_BLANK_HEIGHT) {
                self.ly = 0;
                self.build_bg();
            } else if self.ly < SCREEN_HEIGHT as u8 {
                self.build_bg();
                if self.window_on() {
                    self.build_window();
                }
            }

            if self.ly == self.lyc {
                self.stat.insert(Stat::LYC_STAT);
                if self.stat.contains(Stat::INTR_LYC) {
                    lcdc_irq = true;
                }
            } else {
                self.switch_mode(PpuMode::HBlank);
            }
            self.ly = self.ly.wrapping_add(1);
            self.clock = self.clock.wrapping_sub(CLOCKS_PER_LINE);
        }

        match (vblank_irq, lcdc_irq) {
            (false, false)  =>  (None, None),
            (false, true)   =>  (None, Some(InterruptKind::LcdcStatus)),
            (true, false)   =>  (Some(InterruptKind::Vblank), None),
            (true, true)    =>  (Some(InterruptKind::Vblank),
                                 Some(InterruptKind::LcdcStatus)),
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
    
    fn window_tilemap_offset(&self) -> usize {
        match self.lcdc.contains(Lcdc::WIN_MAP) {
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

    fn update_mode(&mut self) -> bool {
        let mut lcdc_irq = false;
        if self.ly > SCREEN_HEIGHT as u8 {
            self.switch_mode(PpuMode::VBlank);
        } else if self.clock <= 80 {
            self.switch_mode(PpuMode::SearchingOAM);
        } else if self.clock >= 167 && self.clock <= 291 {
            self.switch_mode(PpuMode::TransferPixels);
        } else {
            self.switch_mode(PpuMode::HBlank);
            if self.stat.contains(Stat::INTR_M0) {
                lcdc_irq = true;
            }
        }

        lcdc_irq
    }

    fn sprite_size(&self) -> u8 {
        match self.lcdc.contains(Lcdc::OBJ_SIZE) {
            false   =>  8,
            true    =>  16,
        }
    }

    fn sprite_on(&self) -> bool {
        self.lcdc.contains(Lcdc::OBJ_EN)
    }

    fn window_on(&self) -> bool {
        self.lcdc.contains(Lcdc::WIN_EN)
    }

    fn build_bg(&mut self) {
        for x in 0..SCREEN_WIDTH as u8 {
            let y = self.ly.wrapping_add(self.scy) as u16 / 8 * 32;
            let index = x.wrapping_add(self.scx) as u16 / 8 % 32 + y;
            let tileid = self.get_bg_tileid(index);
            let color = self.get_bg_color(tileid, 
                            x.wrapping_add(self.scx)%8, 
                            self.ly.wrapping_add(self.scy)%8);
            let base = (self.ly as usize * SCREEN_WIDTH + x as usize)%(SCREEN_HEIGHT*SCREEN_WIDTH);
            self.pixels[base] = self.get_bg_palette()[color as usize];
        }
    }

    fn build_sprite(&mut self) {
        let height = self.sprite_size();
        for attr in self.oam.iter() {
            if attr.x == 0 {
                continue;
            }
            for x in 0..8 as u8 {
                for y in 0.. height {
                    let mut posx = x;
                    let mut posy = y;

                    if attr.is_xflip() {
                        posx = 7 - x;
                    }
                    if attr.is_yflip() {
                        posy = 7 - y;
                    }

                    if posx.wrapping_add(attr.offsetx()) >= SCREEN_WIDTH as u8 {
                        continue;
                    }
                    if posy.wrapping_add(attr.offsety()) >= SCREEN_HEIGHT as u8 {
                        continue;
                    }

                    let color = self.get_sprite_color(attr.tileid(), x%8, y%height, height);
                    let base = ((posx.wrapping_add(attr.offsetx()) as usize
                                + (posy.wrapping_add(attr.offsety()) as usize * SCREEN_WIDTH)))
                                %(SCREEN_HEIGHT*SCREEN_WIDTH);
                    if color != 0 {
                        self.pixels[base] = self.get_sprite_palette(*attr)[color as usize];
                    }
                }
            }
        }
    }

    fn build_window(&mut self) {
        if (self.wx >= 167) && (self.wy >= 144) {
            return;
        }
        if self.ly < self.wy {
            return;
        }

        for x in 0..SCREEN_WIDTH as u8 {
            let posx = self.wx.wrapping_sub(7);
            if x < posx {
                continue;
            }
            let y = self.ly.wrapping_sub(self.wy) as u16 / 8 * 32;
            let index = x.wrapping_sub(posx) as u16 / 8 % 32 + y;
            let tileid = self.get_window_tileid(index);
            let color = self.get_bg_color(tileid, 
                            x.wrapping_sub(posx)%8, 
                            self.ly.wrapping_sub(self.wy)%8);
            let base = self.ly as usize * SCREEN_WIDTH + x as usize;
            self.pixels[base] = self.get_bg_palette()[color as usize];
        }
        
    }

    fn get_bg_palette(&self) -> [u8; 4] {
        [   self.bgp.dot_00.to_u8(), self.bgp.dot_01.to_u8(),
            self.bgp.dot_10.to_u8(), self.bgp.dot_11.to_u8()]
    }

    fn get_sprite_palette(&self, oam: Oam) -> [u8; 4] {
        if oam.flags.contains(OamFlags::PALETTE_NO) {
            return [self.obp1.dot_00.to_u8(), self.obp1.dot_01.to_u8(),
                    self.obp1.dot_10.to_u8(), self.obp1.dot_11.to_u8()]
        }

        [   self.obp0.dot_00.to_u8(), self.obp0.dot_01.to_u8(),
            self.obp0.dot_10.to_u8(), self.obp0.dot_11.to_u8()]
    }

    fn get_bg_tileid(&self, index: u16) -> u8 {
        let addr = index as usize + self.bg_tilemap_offset();
        self.read8(addr)
    }

    fn get_window_tileid(&self, index: u16) -> u8 {
        let addr = index as usize + self.window_tilemap_offset();
        self.read8(addr)
    }

    fn get_tile_addr(&self, tileid: u8) -> usize {
        let offset = self.tiledata_offset();

        if offset == TILEDATA0_OFFSET {
            return offset + (tileid.wrapping_add(0x80) as usize) * 0x10;
        }

        offset + (tileid as usize * 0x10)
    }

    fn get_bg_color(&self, tileid: u8, x: u8, y: u8) -> u8 {
        let addr = self.get_tile_addr(tileid);
        let mut pixels = Vec::new();

        for i in 0..8 as usize {
            let line1 = self.read8(addr+i*2);
            let line2 = self.read8(addr+i*2+1);
            for j in 0..8 {
                let lsb = (line1 >> (7-j)) & 0x01;
                let msb = (line2 >> (7-j)) & 0x01;
                pixels.push((msb<<1)+lsb);
            }
        }

        pixels[(x+y*8) as usize]
    }
    
    fn get_sprite_color(&self, tileid: u8, x: u8, y: u8, height: u8) -> u8 {
        let addr = tileid as usize * 0x10 + TILEDATA1_OFFSET;
        let mut pixels = Vec::new();

        for i in 0..height as usize {
            let line1 = self.read8(addr+i*2);
            let line2 = self.read8(addr+i*2+1);
            for j in 0..8 {
                let lsb = (line1 >> (7-j)) & 0x01;
                let msb = (line2 >> (7-j)) & 0x01;
                pixels.push((msb<<1)+lsb);
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

    pub fn is_xflip(&self) -> bool {
        self.flags.contains(OamFlags::XFLIP)
    }
    
    pub fn is_yflip(&self) -> bool {
        self.flags.contains(OamFlags::YFLIP)
    }

    pub fn tileid(&self) -> u8 {
        self.tile
    }

    pub fn offsetx(&self) -> u8 {
        self.x.wrapping_sub(8)
    }
    
    pub fn offsety(&self) -> u8 {
        self.y.wrapping_sub(16)
    }
}

impl Io for Oam {
    fn read8(&self, addr: usize) -> u8 {
        match addr & 0xFF {
            0x00    =>  self.y,
            0x01    =>  self.x,
            0x02    =>  self.tile,
            0x03    =>  self.flags.bits,
            _       =>  panic!("unsupport read at {:04x}", addr),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr & 0xFF {
            0x00    =>  self.y = data,
            0x01    =>  self.x = data,
            0x02    =>  self.tile = data,
            0x03    =>  self.flags = OamFlags::from_bits_truncate(data),
            _       =>  panic!("unsupport write at {:04x}", addr),
        }
    }
}

