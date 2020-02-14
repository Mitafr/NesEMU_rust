use crate::ppu::mem::PpuMem;
use crate::renderer::Renderer;
use crate::renderer::get_rgb;
use crate::ppu::register::PpuRegister;
use crate::ppu::register::Register;
use crate::ppu::palette::PaletteVram;

pub struct Background {
    tiles_data: [u8; 16],
    nametable_addr: usize,
    tile_addr: usize,
    tile_attr: u8,
    tile_low_byte: u8,
    tile_hi_byte: u8,
}

impl Background {
    pub fn new() -> Background {
        Background {
            tiles_data: [0u8; 16],
            nametable_addr: 0x2000,
            tile_addr: 0,
            tile_attr: 0,
            tile_low_byte: 0,
            tile_hi_byte: 0,
        }
    }
    pub fn fetch_attribute(&mut self, vram: &mut PpuMem, register: &mut PpuRegister) {
        let v = register.get_addr();
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let shift = ((v >> 4) & 4) | (v & 2);
        self.tile_attr = ((vram.peek(addr as usize) >> shift) & 3) << 2;
    }
    pub fn fetch_loworder_byte(&mut self, vram: &mut PpuMem, register: &mut PpuRegister) {
        let fine_y = (register.get_addr() >> 12) & 7;
        let background_table = 0x1000 * register.get_background_table() as u16;
        self.tile_addr = (background_table + (16 * vram.peek(self.nametable_addr) as u16) + fine_y) as usize;
        self.tile_low_byte = vram.peek(self.tile_addr);
    }
    pub fn fetch_highorder_byte(&mut self, vram: &mut PpuMem, register: &mut PpuRegister) {
        let fine_y = (register.get_addr() >> 12) & 7;
        let background_table = 0x1000 * register.get_background_table() as u16;
        self.tile_addr = (8 + background_table + (16 * vram.peek(self.nametable_addr) as u16) + fine_y) as usize;
        self.tile_hi_byte = vram.peek(self.tile_addr);
    }
    pub fn fetch_nametable(&mut self, register: &mut PpuRegister) {
        self.nametable_addr = register.get_nametable_address() | register.get_addr() as usize & 0x0FFF;
    }
    pub fn render(&mut self, dot: u32, line: u32, renderer: &mut Renderer, vram: &mut PpuMem, register: &mut PpuRegister) {
        if register.get_background_visibility() == 0 {
            return;
        }
        let rgb = get_rgb(vram.palette.peek_color_background(self.tiles_data[0]));
        let transparent = get_rgb(vram.palette.peek_color_background(0));
        if rgb != transparent {
            renderer.set_pixel_rgb(dot - 2, line, rgb);
        }
    }
    pub fn shit_tile_data(&mut self) {
        for i in 0..15 {
            self.tiles_data[i] = self.tiles_data[i+1];
        }
    }
    pub fn clear_data(&mut self) {
        self.tiles_data = [0u8; 16];
    }
    pub fn store_tile_data(&mut self) {
        for i in 7..15 {
            let low = (self.tile_low_byte & 0x80) >> 7;
            let hi = (self.tile_hi_byte & 0x80) >> 6;
            self.tile_low_byte <<= 1;
            self.tile_hi_byte <<= 1;
            self.tiles_data[i] = self.tile_attr | low | hi;
        }
    }
}