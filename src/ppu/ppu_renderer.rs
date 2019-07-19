extern crate minifb;

use minifb::{Key, WindowOptions, Window, Scale};

const SCALE: Scale = Scale::X1;
const SCREEN_HEIGHT: usize = 240;
const SCREEN_WIDTH: usize = 256;

pub struct PpuRenderer {
    renderer: Window,
    display: [u32; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
}

impl PpuRenderer {
    pub fn new(name: &str) -> PpuRenderer {
        let buffer = [0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        let window_options: WindowOptions = WindowOptions {
            borderless: false,
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