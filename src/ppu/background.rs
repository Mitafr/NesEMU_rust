use crate::ppu::mem::PpuMem;
use crate::renderer::Renderer;
use crate::ppu::register::PpuRegister;
use crate::ppu::register::Register;
use crate::ppu::palette::Palette;
use crate::ppu::tile::Tile;

const TILE_PER_LINE_X: usize = 32;
const TILE_PER_LINE_Y: usize = 30;

pub struct Background {
    pub tiles: Vec<Tile>,
    current_tile: Tile,
    current_pattern: [u8; 16],
    nametable_index: Box<[[u8; TILE_PER_LINE_Y * 8]; TILE_PER_LINE_X * 8]>,
    attribute_tiles: Box<[[u8; TILE_PER_LINE_Y * 8]; TILE_PER_LINE_X * 8]>,
}

impl Background {
    pub fn new() -> Background {
        Background {
            tiles: vec!(),
            current_tile: Tile::new(),
            current_pattern: [0u8; 16],
            nametable_index: Box::new([[0u8; TILE_PER_LINE_Y * 8]; TILE_PER_LINE_X * 8]),
            attribute_tiles: Box::new([[0u8; TILE_PER_LINE_Y * 8]; TILE_PER_LINE_X * 8]),
        }
    }
    pub fn fetch_attribute(&mut self, x: u16, y: u16, vram: &mut PpuMem, register: &mut PpuRegister) {
        let v = register.get_addr() as usize;
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let value = vram.peek(addr);
        self.attribute_tiles[x as usize][y as usize] = value;
    }
    pub fn fetch_loworder_byte(&mut self, x: u16, y: u16, vram: &mut PpuMem, register: &mut PpuRegister) {
        self.current_pattern = [0u8; 16];
        let addr = self.nametable_index[x as usize][y as usize];
        for i in addr..addr + 8 {
            self.current_pattern[(i - addr) as usize] = vram.peek(i as usize);
        }
    }
    pub fn fetch_highorder_byte(&mut self, x: u16, y: u16, vram: &mut PpuMem, register: &mut PpuRegister) {
        let addr = self.nametable_index[x as usize][y as usize] + 8;
        for i in addr..addr + 8 {
            self.current_pattern[(i - addr) as usize] = vram.peek(i as usize);
        }
        self.current_tile.build_tile(&self.current_pattern, addr as usize);
        self.tiles.push(self.current_tile.clone());
        self.current_tile = Tile::new();
    }
    pub fn fetch_nametable(&mut self, x: u16, y: u16, vram: &mut PpuMem, register: &mut PpuRegister) {
        let v = register.get_addr() as usize;
        // let nametable_select = v & 0b0001_1000_0000_0000;
        let nametable_address = register.get_nametable_address() | v & 0x0FFF;
        let index = vram.peek(nametable_address);
        if index != 0 {
            println!("({}, {})", y, x);
            self.nametable_index[x as usize][y as usize] = index;
        }
    }
    #[allow(unused_variables)]
    pub fn build(&mut self, vram: &mut PpuMem, register: &mut PpuRegister, tileset: &mut Vec<Tile>) {
        // fetch nametable
        for i in 0x2000..0x23BF {
            //let content = vram.peek(i);
            //let mut tile = tileset[content as usize].clone();
            //tile.set_index(i - 0x2000);
            //self.tiles.push(tile);
        }
    }
    pub fn clear(&mut self) {
        self.tiles = vec!();
    }
    pub fn draw(&mut self, renderer: &mut Renderer, palette: &mut Palette) {
        for tile in self.tiles.iter_mut() {
            renderer.set_tile(tile, palette);
        }
    }
}