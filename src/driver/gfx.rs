extern crate minifb;

use minifb::{Key, WindowOptions, Window, Scale};

use crate::ppu::Ppu;

const SCALE: Scale = Scale::X1;
const SCREEN_HEIGHT: usize = 240;
const SCREEN_WIDTH: usize = 256;

pub struct Gfx<'a> {
    ppu: &'a Ppu,
    renderer: Window,
    display: [u32; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
}

impl<'a> Gfx<'a> {
    pub fn new(ppu: &'a Ppu, name: &str) -> Gfx<'a> {
        let buffer = [0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        let window_options: WindowOptions = WindowOptions {
            borderless: true,
            title: true,
            resize: false,
            scale: SCALE,
        };
        let window = Window::new(name,
                                    SCREEN_HEIGHT,
                                    SCREEN_HEIGHT,
                                    window_options).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        Gfx {
            ppu: ppu,
            renderer: window,
            display: buffer,
        }
    }
    pub fn draw_window(&mut self) {
        self.renderer.update_with_buffer(&self.display).unwrap();
    }
    pub fn is_open(&mut self) -> bool {
        self.renderer.is_open()
    }
    pub fn is_close_key_pressed(&mut self) -> bool {
        self.renderer.is_key_down(Key::Escape)
    }
    /*pub fn has_pixel(&mut self, x: usize, y: usize) -> bool {
        self.display[x][y] != 0
    }*/
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        let x = x % SCREEN_WIDTH as u32;
        let y = y % SCREEN_HEIGHT as u32;
        self.display[(x + SCREEN_WIDTH as u32 * y) as usize] = color;
    }
}