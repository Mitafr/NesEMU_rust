use crate::ppu::mem::PpuMem;
use crate::renderer::Renderer;
use crate::ppu::register::PpuRegister;
use crate::ppu::register::Register;
use crate::ppu::palette::Palette;
use crate::ppu::tile::Tile;

pub struct Background {
    pub tiles: Vec<Tile>,
    current_tile: Tile,
    current_pattern: [u8; 16],
    tile_index: u8,
    nametable_addr: usize,
    tile_addr: usize,
    tile_attr: u8,
    empty_pattern: bool,
}

impl Background {
    pub fn new() -> Background {
        Background {
            tiles: vec!(),
            current_tile: Tile::new(),
            current_pattern: [0u8; 16],
            tile_index: 0,
            nametable_addr: 0x2000,
            tile_addr: 0,
            tile_attr: 0,
            empty_pattern: true,
        }
    }
    pub fn fetch_attribute(&mut self, addr: usize, vram: &mut PpuMem) {
        let addr = 0x23C0 | (addr & 0x0C00) | ((addr >> 4) & 0x38) | ((addr >> 2) & 0x07);
        let value = vram.peek(addr);
        self.tile_attr = value;
    }
    pub fn fetch_loworder_byte(&mut self, vram: &mut PpuMem, register: &mut PpuRegister) {
        self.current_pattern = [0u8; 16];
        self.tile_addr = ((self.tile_index as usize) << 4)
                        | register.get_background_table() as usize;
        for i in self.tile_addr..self.tile_addr + 8 {
            if vram.peek(i as usize) != 0 {
                self.empty_pattern = false;
            }
            self.current_pattern[(i - self.tile_addr) as usize] = vram.peek(i as usize);
        }
    }
    pub fn fetch_highorder_byte(&mut self, vram: &mut PpuMem) {
        self.tile_addr += 8;
        for i in self.tile_addr..self.tile_addr + 8 {
            if vram.peek(i as usize) != 0 {
                self.empty_pattern = false;
            }
            self.current_pattern[(i - self.tile_addr + 8) as usize] = vram.peek(i as usize);
        }
        self.current_tile.build_tile(&self.current_pattern, (self.nametable_addr - 0x2000) as usize);
        self.tiles.push(self.current_tile.clone());
        self.current_tile = Tile::new();
        self.empty_pattern = true;
    }
    pub fn fetch_nametable(&mut self, addr: usize, vram: &mut PpuMem, register: &mut PpuRegister) {
        self.nametable_addr = register.get_nametable_address() | addr & 0x0FFF;
        self.tile_index = vram.peek(self.nametable_addr);
    }
    pub fn clear(&mut self) {
        self.tiles = Vec::new();
    }
    pub fn draw(&mut self, renderer: &mut Renderer, palette: &mut Palette) {
        for tile in self.tiles.iter_mut() {
            if tile.index != 0 {
                renderer.set_tile(tile, palette);
            }
        }
    }
}