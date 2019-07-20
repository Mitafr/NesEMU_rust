extern crate minifb;

use minifb::{Key, WindowOptions, Window, Scale};

use crate::Context;
use crate::ppu::ppu_palette::PaletteVram;
use crate::ppu::ppu_background::Background;
use crate::ppu::ppu_mem::PpuMem;
use crate::memory::Memory;
use crate::ppu::ppu_register::Register;
use crate::ppu::ppu_register::PpuRegister;
use crate::ppu::ppu_renderer::PpuRenderer;
use crate::ppu::ppu_palette::Palette;
use crate::ppu::ppu_tile::Tile;
use crate::rom::Cartbridge;

const SCALE: Scale = Scale::X2;
const SCREEN_HEIGHT: usize = 240;
const SCREEN_WIDTH: usize = 256 * 2;

#[derive(PartialEq)]
pub enum DebuggerStatus {
    PROCESSING,
    BREAK,
}

pub struct PpuDebugger {
    renderer: Window,
    display: Vec<u32>,
}

impl PpuDebugger {
    pub fn new() -> PpuDebugger {
        let buffer = vec!(0u32; SCREEN_WIDTH * SCREEN_HEIGHT);
        let window_options: WindowOptions = WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: SCALE,
        };
        let window = Window::new("Nametable",
                                    SCREEN_WIDTH,
                                    SCREEN_HEIGHT,
                                    window_options).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        PpuDebugger {
            renderer: window,
            display: buffer,
        }
    }
    pub fn draw_tileset(&mut self, tileset: &Vec<Tile>, palette: &Palette) -> DebuggerStatus {
        for (tile,t) in tileset.iter().enumerate() {
            for (y, tt) in t.iter().enumerate() {
                for (x, tt) in tt.iter().enumerate() {
                    let color = palette.peek_color_background(*tt);
                    if color != 0 {
                        let xcoord = (((tile % 32) * 8 + x)) as u32;
                        let ycoord = (((tile / 32) * 8 + y)) as u32;
                        self.set_pixel(xcoord, ycoord, color & 0xFFFFFF);
                    }
                }
            }
        }
        if !self.is_open() || self.is_close_key_pressed() {
            return DebuggerStatus::BREAK;
        }
        self.renderer.update_with_buffer(&self.display).unwrap();
        DebuggerStatus::PROCESSING
    }
    pub fn draw_palette(&mut self, palette: &Palette) -> DebuggerStatus {
        for (tile,t) in palette.background.iter().enumerate() {
            let xcoord = 256 + (((tile % 32) * 8 + tile * 8)) as u32;
            let ycoord = 256 + (((tile / 32) * 8)) as u32;
            for i in 0..8 {
                for j in 0..8 {
                    self.set_pixel(xcoord + i, ycoord + j, *t);
                }
            }
        }
        if !self.is_open() || self.is_close_key_pressed() {
            return DebuggerStatus::BREAK;
        }
        self.renderer.update_with_buffer(&self.display).unwrap();
        DebuggerStatus::PROCESSING
    }
    pub fn init(&mut self) {
        self.renderer.set_position(0, 0);
    }
    pub fn is_open(&mut self) -> bool {
        self.renderer.is_open()
    }
    pub fn is_close_key_pressed(&mut self) -> bool {
        self.renderer.is_key_down(Key::Escape)
    }
    /*pub fn has_pixel(&mut self, x: u32, y: u32) -> bool {
        self.display[get_coords(x, y)] != 0
    }*/
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        let coords = get_coords(x, y);
        self.display[coords] = color;
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
