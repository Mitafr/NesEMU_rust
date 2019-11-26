use sdl2;
use sdl2::render::*;
use sdl2::video::*;
use sdl2::pixels::*;

use crate::ppu::tile::Tile;
use crate::ppu::sprite::Sprite;
use crate::ppu::palette::Palette;
use crate::ppu::palette::PaletteVram;

const SCALE: u32 = 2;
const SCREEN_HEIGHT: u32 = 224;
const SCREEN_WIDTH: u32 = 256;

pub struct Renderer {
    renderer: Canvas<Window>,
    display: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
    texture: sdl2::render::Texture,
}

impl Renderer {
    pub fn new(sdl_context: &sdl2::Sdl, name: &str) -> Renderer {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window(name, SCREEN_WIDTH * SCALE, SCREEN_HEIGHT * SCALE)
            .position_centered()
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
        Renderer {
            renderer: canvas,
            display: [0u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
            texture: texture_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap(),
        }
    }
    pub fn draw_window(&mut self) {
        self.renderer.clear();
        self.texture.update(None, &self.display, (SCREEN_WIDTH * 3) as usize).unwrap();
        self.renderer.copy(&self.texture, None, None).unwrap();
        self.renderer.present();
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let coords = get_coords(x, y);
        self.display[coords as usize] = r;
        self.display[(coords + 1) as usize] = g;
        self.display[(coords + 2) as usize] = b;
    }
    pub fn set_tile(&mut self, tile: &mut Tile, palette: &mut Palette) {
        let yoffset = ((tile.index as u32 / ((SCREEN_HEIGHT + 36) / 8)) * 8) as u32;
        let xoffset = ((tile.index as u32 % (SCREEN_WIDTH / 8)) * 8) as u32;
        for (i, x) in tile.get_pixels().iter().enumerate() {
            for (j, y) in x.iter().enumerate() {
                let color = get_rgb(palette.peek_color_background(*y));
                self.set_pixel(j as u32 + xoffset, i as u32 + yoffset, color.0, color.1, color.2);
            }
        }
    }
    pub fn set_sprite(&mut self, sprite: &mut Sprite, palette: &mut Palette) {
        let xcoord = sprite.coord_x;
        let ycoord = sprite.coord_y;
        for (i, x) in sprite.get_pixels().iter().enumerate() {
            for (j, y) in x.iter().enumerate() {
                let color = get_rgb(palette.peek_color_sprite(*y));
                self.set_pixel(xcoord as u32 + j as u32, ycoord as u32 + i as u32, color.0, color.1, color.2);
            }
        }
    }
    pub fn reset(&mut self) {
        self.display = [0u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize];
        self.texture.update(None, &self.display, (SCREEN_WIDTH * 3) as usize).unwrap();
        self.renderer.clear();
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