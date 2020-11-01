use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics;
use ggez::nalgebra::Point2;
use ggez::timer;
use std::path::Path;
use std::thread;
use std::time;

use crate::core::cpu::Cpu;
use crate::core::pad::Key;

const SCREEN_WIDTH:     u32 = 160;
const SCREEN_HEIGHT:    u32 = 144;

const COLORS: [[u8; 4]; 4] = [
    [0x0F, 0x38, 0x0F, 0xFF],
    [0x30, 0x62, 0x30, 0xFF],
    [0x8B, 0xAC, 0x0F, 0xFF],
    [0x9B, 0xBC, 0x0F, 0xFF],
];

pub struct MainWindow {
    cpu:        Cpu,
    palette:    Vec<graphics::spritebatch::SpriteBatch>,
    pixels:     [u8; (SCREEN_WIDTH*SCREEN_HEIGHT) as usize],
}


impl MainWindow {
    pub fn new(path: &Path, ctx: &mut Context) -> MainWindow {        
        MainWindow {
            cpu:        Cpu::from_path(path),
            palette:    MainWindow::get_init_palette(ctx),
            pixels:     [3; (SCREEN_WIDTH*SCREEN_HEIGHT) as usize],
        }
    }

    fn get_init_palette(ctx: &mut Context) -> Vec<graphics::spritebatch::SpriteBatch> {
        let mut palette = Vec::new();

        for color in &COLORS {
            let green = graphics::Image::from_rgba8(
                ctx,
                1,
                1,
                color,
            ).unwrap();
            palette.push(graphics::spritebatch::SpriteBatch::new(green));
        }
        palette
    }

    pub fn update_pixels(&mut self, pixels: [u8;(SCREEN_WIDTH*SCREEN_HEIGHT) as usize]) {
        self.pixels = pixels;
    }
}

impl EventHandler for MainWindow {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        for _ in 0..17556 {
            self.cpu.tick();
            self.update_pixels(self.cpu.get_pixels());
        }

        if timer::ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::delta(ctx));
            println!("Average FPS: {}", timer::fps(ctx));
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);

        self.palette = MainWindow::get_init_palette(ctx);

        for i in 0..self.pixels.len() as u32 {
            let x = (i % SCREEN_WIDTH) as f32;
            let y = (i / SCREEN_WIDTH % SCREEN_HEIGHT) as f32;
            let p = graphics::DrawParam::new()
                .dest(Point2::new(x, y));
                
            self.palette[self.pixels[i as usize] as usize].add(p);
        }
        let param = graphics::DrawParam::new()
            .dest(Point2::new(0.0, 0.0));

        for gray in &self.palette {
            graphics::draw(ctx, gray, param)?;
        }

        self.palette.clear();

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool
    ) {
        match keycode {
            KeyCode::Left       =>  self.cpu.push_key(Key::Left),
            KeyCode::Right      =>  self.cpu.push_key(Key::Right),
            KeyCode::Up         =>  self.cpu.push_key(Key::Up),
            KeyCode::Down       =>  self.cpu.push_key(Key::Down),
            KeyCode::Z          =>  self.cpu.push_key(Key::A),
            KeyCode::X          =>  self.cpu.push_key(Key::B),
            KeyCode::Return     =>  self.cpu.push_key(Key::Start),
            KeyCode::Back       =>  self.cpu.push_key(Key::Select),
            _                   =>  (),
        }
    }
    
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods
    ) {
        println!("Key released: {:?}, modifier {:?}",
                keycode, keymod);

        match keycode {
            KeyCode::Left       =>  self.cpu.release_key(Key::Left),
            KeyCode::Right      =>  self.cpu.release_key(Key::Right),
            KeyCode::Up         =>  self.cpu.release_key(Key::Up),
            KeyCode::Down       =>  self.cpu.release_key(Key::Down),
            KeyCode::Z          =>  self.cpu.release_key(Key::A),
            KeyCode::X          =>  self.cpu.release_key(Key::B),
            KeyCode::Return     =>  self.cpu.release_key(Key::Start),
            KeyCode::Back       =>  self.cpu.release_key(Key::Select),
            _                   =>  (),
        }
    }
}

pub fn run(path: &Path) {
    let (mut ctx, mut event_loop) =
       ContextBuilder::new("GBR", "Noboru")
            .window_setup(ggez::conf::WindowSetup::default().vsync(false))
            .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32))
            .build()
            .unwrap();

    let mut window = MainWindow::new(path, &mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut window) {
        Ok(_)   => println!("Exited cleanly."),
        Err(e)  => println!("Error occured: {}", e)
    }
}