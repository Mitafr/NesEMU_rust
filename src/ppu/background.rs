use crate::ppu::mem::PpuMem;
use crate::ppu::renderer::PpuRenderer;
use crate::ppu::register::PpuRegister;
use crate::ppu::palette::Palette;
use crate::ppu::tile::Tile;

pub struct Background {
    pub tiles: Vec<Tile>,
}

impl Background {
    pub fn new() -> Background {
        Background {
            tiles: vec!(),
        }
    }
    pub fn build(&mut self, vram: &mut PpuMem, register: &mut PpuRegister, tileset: &mut Vec<Tile>) {
        // fetch nametable
        for i in 0x2000..0x23FF {
            let content = vram.peek(i);
            if content != 0u8 {
                let mut tile = tileset[content as usize].clone();
                tile.set_index(i - 0x2000);
                self.tiles.push(tile);
            }
        }
    }
    pub fn clear(&mut self) {
        self.tiles.clear();
    }
    pub fn draw(&mut self, renderer: &mut PpuRenderer, palette: &mut Palette) {
        for tile in self.tiles.iter_mut() {
            renderer.set_tile(tile, palette);
        }
    }
}