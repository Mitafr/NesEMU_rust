use sdl2;
use sdl2::render::*;
use sdl2::video::*;
use sdl2::pixels::*;

use crate::ppu::palette::PaletteVram;
use crate::ppu::palette::Palette;
use crate::ppu::tile::Tile;

const SCREEN_HEIGHT: u32 = 248;
const SCREEN_WIDTH: u32 = 256 * 2;
const SCALE: u32 = 3;

#[derive(PartialEq)]
pub enum DebuggerStatus {
    PROCESSING,
    BREAK,
}

pub struct PpuDebugger {
    renderer: Canvas<Window>,
    display: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
    texture: sdl2::render::Texture<'static>,
    is_open: bool,
}

impl PpuDebugger {
    pub fn new(sdl_context: &sdl2::Sdl) -> PpuDebugger {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window("Debugger", SCREEN_WIDTH * SCALE, SCREEN_HEIGHT * SCALE)
            .position_centered()
            .set_window_flags(8u32 | 16u32)
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let canvas = window.into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let texture_creator = &canvas.texture_creator() as *const TextureCreator<WindowContext>;
        let texture = unsafe {&*texture_creator}.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
        PpuDebugger {
            renderer: canvas,
            display: [0u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
            texture: texture,
            is_open: false,
        }
    }
    pub fn draw_tileset(&mut self, tileset: &Vec<Tile>, palette: &Palette) -> DebuggerStatus {
        for (tile_index, tile) in tileset.iter().enumerate() {
            for (y, tt) in tile.iter().enumerate() {
                for (x, tile_v) in tt.iter().enumerate() {
                    if *tile_v == 0 {
                        continue;
                    }
                    let col = palette.peek_color_background(*tile_v);
                    let xcoord = (((tile_index % 32) * 8 + x)) as u32;
                    let ycoord = (((tile_index / 32) * 8 + y)) as u32;
                    let color = get_rgb(col);
                    self.set_pixel(xcoord, ycoord, color.0, color.1, color.2);
                }
            }
        }
        if !self.is_open {
            return DebuggerStatus::BREAK;
        }
        DebuggerStatus::PROCESSING
    }
    pub fn draw_palette(&mut self, palette: &Palette) -> DebuggerStatus {
        for (tile,t) in palette.background.iter().enumerate() {
            let xcoord = (SCREEN_WIDTH / 2) + (((tile % 32) * 8)) as u32;
            let ycoord = (((tile / 32) * 8)) as u32;
            if *t == 0 {
                continue;
            }
            let color = get_rgb(*t);
            for i in 0..8 {
                for j in 0..8 {
                    self.set_pixel(xcoord + i, ycoord + j, color.0, color.1, color.2);
                }
            }
        }
        for (tile,t) in palette.sprite.iter().enumerate() {
            let xcoord = (SCREEN_WIDTH / 2) + (((tile % 32) * 8)) as u32;
            let ycoord = 128 + (((tile / 32) * 8)) as u32;
            if *t == 0 {
                continue;
            }
            let color = get_rgb(*t);
            for i in 0..8 {
                for j in 0..8 {
                    self.set_pixel(xcoord + i, ycoord + j, color.0, color.1, color.2);
                }
            }
        }
        if !self.is_open {
            return DebuggerStatus::BREAK;
        }
        DebuggerStatus::PROCESSING
    }
    pub fn draw(&mut self) {
        self.renderer.clear();
        self.texture.update(None, &self.display, (SCREEN_WIDTH * 3) as usize).unwrap();
        self.renderer.copy(&self.texture, None, None).unwrap();
        self.renderer.present();
    }
    pub fn init(&mut self) {
        // self.renderer.set_position(x as isize, y as isize);
    }
    pub fn toggle_view(&mut self) {
        if self.is_open {
            self.renderer.window_mut().hide();
        } else {
            self.renderer.window_mut().show();
        }
        self.is_open = !self.is_open;
    }
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    /*pub fn has_pixel(&mut self, x: u32, y: u32) -> bool {
        self.display[get_coords(x, y)] != 0
    }*/
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let coords = get_coords(x, y);
        self.display[coords as usize] = r;
        self.display[(coords + 1) as usize] = g;
        self.display[(coords + 2) as usize] = b;
    }
}

fn get_coords(x: u32, y: u32) -> u32 {
    let x = x % SCREEN_WIDTH;
    let y = y % SCREEN_HEIGHT;
    (x + SCREEN_WIDTH * y) * 3
}

fn get_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color & 0xFF0000) >> 16) as u8;
    let g = ((color & 0x00FF00) >> 8) as u8;
    let b = (color & 0x0000FF) as u8;
    (r, g, b)
}