use sdl2;
use sdl2::render::*;
use sdl2::video::*;
use sdl2::pixels::*;

use crate::ppu::palette::PaletteVram;
use crate::ppu::palette::Palette;
use crate::ppu::sprite::SpriteMem;
use crate::ppu::tile::Tile;

const SCREEN_HEIGHT: u32 = 240 * 2;
const SCREEN_WIDTH: u32 = 256 * 2;

#[derive(PartialEq)]
pub enum DebuggerStatus {
    PROCESSING,
    BREAK,
}

pub struct PpuDebugger {
    renderer: Canvas<Window>,
    display: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
    texture: Texture,
    is_open: bool,
}

impl PpuDebugger {
    pub fn new(sdl_context: &sdl2::Sdl, scale: f32) -> PpuDebugger {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window("Debugger", (SCREEN_WIDTH as f32 * scale).floor() as u32, (SCREEN_HEIGHT as f32 * scale).floor() as u32)
            .position_centered()
            .set_window_flags(8u32)
            .resizable()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let canvas = window.into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let texture_creator = canvas.texture_creator();
        PpuDebugger {
            renderer: canvas,
            display: [0u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
            texture: texture_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap(),
            is_open: false,
        }
    }
    pub fn draw_tileset(&mut self, tileset: &Vec<Tile>, palette: &Palette) -> DebuggerStatus {
        self.draw_rect(0, 0, 256, 240);
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
                    self.set_pixel_rgb(xcoord, ycoord, color);
                }
            }
        }
        if !self.is_open {
            return DebuggerStatus::BREAK;
        }
        DebuggerStatus::PROCESSING
    }
    pub fn draw_palette(&mut self, palette: &Palette) -> DebuggerStatus {
        self.draw_rect(256, 0, 256, 240);
        for (tile,t) in palette.background.iter().enumerate() {
            let xcoord = (SCREEN_WIDTH / 2) + (((tile % 32) * 8)) as u32;
            let ycoord = (((tile / 32) * 8)) as u32;
            if *t == 0 {
                continue;
            }
            let color = get_rgb(*t);
            for i in 0..8 {
                for j in 0..8 {
                    self.set_pixel_rgb(xcoord + i, ycoord + j, color);
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
                    self.set_pixel_rgb(xcoord + i, ycoord + j, color);
                }
            }
        }
        if !self.is_open {
            return DebuggerStatus::BREAK;
        }
        DebuggerStatus::PROCESSING
    }
    pub fn draw_nametable(&mut self, nametable: &[u8], tileset: &Vec<Tile>, palette: &Palette) {
        self.draw_rect(0, 240, 256, 240);
        for (b, i) in nametable.iter().enumerate() {
            if *i == 0 {
                continue;
            }
            let tile = tileset.get(*i as usize).unwrap();
            for (y, tt) in tile.iter().enumerate() {
                for (x, tile_v) in tt.iter().enumerate() {
                    let col = palette.peek_color_background(*tile_v);
                    let xcoord = 1 + (((b % 32) * 8 + x)) as u32;
                    let ycoord = 241 + (((b as usize / 32) * 8 + y)) as u32;
                    let color = get_rgb(col);
                    if color == (0,0,0) || color == get_rgb(palette.peek_color_background(0)) {
                        continue;
                    }
                    self.set_pixel_rgb(xcoord, ycoord, color);
                }
            }
        }
        self.draw_rect(256, 240, 256, 240);
    }
    pub fn draw_sprites(&mut self, tileset: &Vec<Tile>, mem: &SpriteMem, palette: &Palette) {
        self.draw_rect(256, 240, 256, 240);
        let spr_mem = mem.get_oam();
        for i in (0..spr_mem.len()).step_by(4) {
            let mut y = 0;
            let mut x = 0;
            let mut index = 0;
            let mut attr = 0;
            for j in 0..4 {
                if j % 4 == 0 {
                    y = spr_mem[i+j];
                }
                if j % 4 == 1 {
                    index = spr_mem[i+j];
                }
                if j % 4 == 2 {
                    attr = spr_mem[i+j];
                }
                if j % 4 == 3 {
                    x = spr_mem[i+j];
                }
            }
            let tile = tileset.get(index as usize).unwrap();
            for (y2, tt) in tile.iter().enumerate() {
                for (x2, tile_v) in tt.iter().enumerate() {
                    let col = palette.peek_color_sprite(attr & 3, *tile_v);
                    let xcoord = 256 + (((x as usize) + x2)) as u32;
                    let ycoord = 240 + (((y as usize) + y2)) as u32;
                    let color = get_rgb(col);
                    if color == (0,0,0) || color == get_rgb(palette.peek_color_sprite(0, 0)) {
                        self.set_pixel_rgb(xcoord, ycoord, color);
                        continue;
                    }
                    self.set_pixel_rgb(xcoord, ycoord, color);
                }
            }
        }
    }
    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        for i in x..x+w {
            for j in y..y+h {
                if i != x && i != x+w - 1 && j != y && j != y+h - 1 {
                    continue;
                }
                self.set_pixel_rgb(i, j, (255,255,255));
            }
        }
    }
    pub fn draw(&mut self) {
        self.renderer.clear();
        self.texture.update(None, &self.display, (SCREEN_WIDTH * 3) as usize).unwrap();
        self.renderer.copy(&self.texture, None, None).unwrap();
        self.renderer.present();
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
    pub fn set_pixel_rgb(&mut self, x: u32, y: u32, color: (u8,u8,u8)) {
        let coords = get_coords(x, y);
        self.display[coords as usize] = color.0;
        self.display[(coords + 1) as usize] = color.1;
        self.display[(coords + 2) as usize] = color.2;
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