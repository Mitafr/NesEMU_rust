use sdl2;
use sdl2::render::*;
use sdl2::video::*;
use sdl2::pixels::*;

use std::time::Instant;

const SCREEN_HEIGHT: u32 = 224;
const SCREEN_WIDTH: u32 = 256;

pub struct Renderer {
    renderer: Canvas<Window>,
    display: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
    texture: sdl2::render::Texture,
    last_frame_time: Instant,
}

impl Renderer {
    pub fn new(sdl_context: &sdl2::Sdl, name: &str, scale: f32) -> Renderer {
        println!("RENDERER: Initializing ...");
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window(name, (SCREEN_WIDTH as f32 * scale).floor() as u32, (SCREEN_HEIGHT as f32 * scale).floor() as u32)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let canvas = window.into_canvas()
            .accelerated()
            .target_texture()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let texture_creator = canvas.texture_creator();
        println!("RENDERER: Initialized successfully");
        Renderer {
            renderer: canvas,
            display: [0u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
            texture: texture_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap(),
            last_frame_time: Instant::now(),
        }
    }
    pub fn draw_window(&mut self) {
        self.renderer.clear();
        self.texture.update(None, &self.display, (SCREEN_WIDTH * 3) as usize).unwrap();
        self.renderer.copy(&self.texture, None, None).unwrap();
        self.renderer.present();
        /*let ms = self.last_frame_time.elapsed().as_millis();
        println!("{:?} FPS", (1000/ms) as f64);
        self.last_frame_time = Instant::now();*/
    }
    pub fn set_pixel_rgb(&mut self, x: u32, y: u32, color: (u8,u8,u8)) {
        let coords = get_coords(x, y);
        self.display[coords as usize] = color.0;
        self.display[(coords + 1) as usize] = color.1;
        self.display[(coords + 2) as usize] = color.2;
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

pub fn get_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color & 0xFF0000) >> 16) as u8;
    let g = ((color & 0x00FF00) >> 8) as u8;
    let b = (color & 0x0000FF) as u8;
    (r, g, b)
}