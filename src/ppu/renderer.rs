extern crate minifb;

use minifb::{Key, WindowOptions, Window, Scale};

use crate::ppu::tile::Tile;
use crate::ppu::colors::COLORS;
use crate::ppu::palette::Palette;
use crate::ppu::palette::PaletteVram;

const SCALE: Scale = Scale::X4;
const SCREEN_HEIGHT: usize = 224;
const SCREEN_WIDTH: usize = 256;

pub struct PpuRenderer {
    renderer: Window,
    display: Vec<u32>,
}

impl PpuRenderer {
    pub fn new(name: &str) -> PpuRenderer {
        let buffer = vec!(0u32; SCREEN_WIDTH * SCREEN_HEIGHT);
        let window_options: WindowOptions = WindowOptions {
            borderless: true,
            title: true,
            resize: false,
            scale: SCALE,
        };
        let window = Window::new(name,
                                    SCREEN_WIDTH,
                                    SCREEN_HEIGHT,
                                    window_options).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        PpuRenderer {
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
    pub fn is_debugger_key_pressed(&mut self) -> bool {
        self.renderer.is_key_down(Key::F1)
    }
    /*pub fn has_pixel(&mut self, x: u32, y: u32) -> bool {
        self.display[get_coords(x, y)] != 0
    }*/
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        let coords = get_coords(x, y);
        self.display[coords] = color;
    }
    pub fn set_tile(&mut self, tile: &mut Tile, palette: &mut Palette) {
        let yoffset = (tile.index as u32 / 34) as u32;
        let xoffset = (tile.index as u32 % 32) as u32;
        let offsetcoords = get_coords(xoffset * 8, yoffset * 8);
        for (i, x) in tile.get_pixels().iter().enumerate() {
            let ycoord = i as u32;
            for (j, y) in x.iter().enumerate() {
                if *y != 0u8 {
                    let xcoord = j as u32;
                    let coords = get_coords(xcoord, ycoord);
                    self.display[coords + offsetcoords] = palette.peek_color_background(*y);
                }
            }
        }
    }
    pub fn set_pixel_direct(&mut self, v: usize, color: u32) {
        self.display[v] = color;
    }
}

fn get_coords(x: u32, y: u32) -> usize {
    let x = x as usize % SCREEN_WIDTH;
    let y = y as usize % SCREEN_HEIGHT;
    x + SCREEN_WIDTH * y
}