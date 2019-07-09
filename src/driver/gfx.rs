use sdl2;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::fmt;

const SCALE: u32 = 1;
const SCREEN_HEIGHT: u32 = 240;
const SCREEN_WIDTH: u32 = 256;

pub struct Gfx {
    renderer: Canvas<Window>,
    display: [[u8; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
}

impl Gfx {
    pub fn new(sdl_context: &sdl2::Sdl, name: &str) -> Gfx {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window(name, SCREEN_WIDTH * SCALE, SCREEN_HEIGHT * SCALE)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
        Gfx {
            renderer: canvas,
            display: [[0; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
        }
    }
    pub fn has_pixel(&mut self, x: usize, y: usize) -> bool {
        self.display[x][y] != 0
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u8) {
        self.display[x as usize][y as usize] ^= color;
    }
    pub fn clear(&mut self) {
        self.display = [[0u8; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize];
    }
}

impl fmt::Display for Gfx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for b in 0..SCREEN_WIDTH {
            for c in 0..SCREEN_HEIGHT {
                writeln!(f, "({}, {}) => {:x?}", b, c, self.display[c as usize][b as usize])?;
            }
        }
        Ok(())
    }
}