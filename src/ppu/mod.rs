pub mod background;
pub mod colors;
pub mod mem;
pub mod sprite;
pub mod palette;
pub mod register;
pub mod tile;

use crate::ppu::background::Background;
use crate::ppu::mem::PpuMem;
use crate::ppu::sprite::SpriteMem;
#[allow(unused_imports)]
use crate::ppu::sprite::Sprite;
use crate::ppu::sprite::SpriteSet;
use crate::ppu::register::Register;
use crate::ppu::register::PpuRegister;
use crate::ppu::palette::Palette;
#[allow(unused_imports)]
use crate::ppu::palette::PaletteVram;
use crate::ppu::tile::Tile;
use crate::rom::Cartbridge;

use std::boxed::Box;
use std::fmt;

#[derive(PartialEq)]
pub enum PpuStatus {
    PROCESSING,
    RENDERING,
    INTERRUPTNMI,
}

const CYCLE_PER_LINE: i16 = 341;

pub struct Ppu {
    pub register: PpuRegister,
    pub mem: PpuMem,
    pub spr_mem: SpriteMem,
    pub sprites: SpriteSet,
    pub background: Background,
    pub palette: Palette,
    pub dot: i16,
    pub line: i16,
    temp_tile_addr: usize,
    updated: bool,
    pub tileset: Box<Vec<Tile>>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            background: Background::new(),
            register: PpuRegister::new(),
            mem: PpuMem::new(),
            spr_mem: SpriteMem::new(),
            sprites: SpriteSet::new(),
            palette: Palette::new(),
            dot: 0,
            line: 0,
            temp_tile_addr: 0x0000,
            updated: false,
            tileset: Box::new(Vec::new()),
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.register.peek(i)
    }
    pub fn write(&mut self, i: usize, v: u8) -> u8 {
        self.updated = true;
        self.register.write(i, v, &mut self.mem, &mut self.palette, &mut self.spr_mem)
    }
    pub fn init(&mut self, rom: &mut Cartbridge) {
        self.mem.set_cram(rom.get_character().to_vec());
        println!("PPU: CRAM OK");
        let mut i = 0;
        while i < 0x1000 {
            let mut v = [0; 16];
            for j in 0..16 {
                v[j] = self.mem.peek(i as usize);
                i += 1;
            }
            let mut tile = Tile::new();
            tile.build_tile(&v, i);
            self.tileset.push(tile);
        }
        println!("PPU: Tileset OK");
    }
    pub fn reset(&mut self) {
        self.mem = PpuMem::new();
        self.palette = Palette::new();
        self.register = PpuRegister::new();
        self.spr_mem = SpriteMem::new();
        self.sprites = SpriteSet::new();
        self.tileset = Box::new(Vec::new());
    }
    pub fn run(&mut self) -> PpuStatus {
        let mut current_status = PpuStatus::PROCESSING;
        if self.line == 0 {
            self.background.clear();
        }
        self.dot += 1;
        if self.dot <= 256 {
            match self.dot % 8 {
                1 => self.temp_tile_addr = self.register.get_addr() as usize,
                2 => self.background.fetch_nametable(self.temp_tile_addr, &mut self.mem, &mut self.register),
                3 => self.temp_tile_addr = self.register.get_addr() as usize,
                4 => self.background.fetch_attribute(self.temp_tile_addr, &mut self.mem),
                6 => self.background.fetch_loworder_byte(&mut self.mem, &mut self.register),
                0 => self.background.fetch_highorder_byte(&mut self.mem),
                _ => {}
            }
        }
        if self.dot >= 321 && self.dot <= 336 {
            match self.dot % 8 {
                1 => self.temp_tile_addr = self.register.get_addr() as usize,
                2 => self.background.fetch_nametable(self.temp_tile_addr, &mut self.mem, &mut self.register),
                4 => self.background.fetch_attribute(self.temp_tile_addr, &mut self.mem),
                6 => self.background.fetch_loworder_byte(&mut self.mem, &mut self.register),
                0 => self.background.fetch_highorder_byte(&mut self.mem),
                _ => {}
            }
        }
        if self.line == 241 && self.dot == 1 {
            self.register.set_vblank();
            if self.register.get_nmi_enable() == 0x1 {
                current_status = PpuStatus::INTERRUPTNMI;
            }
        }
        if self.dot >= CYCLE_PER_LINE {
            self.dot = 0;
            self.line += 1;
            if self.line >= 261 {
                self.line = -1;
                self.register.clear_vblank();
                self.register.clear_spritehit();
                //self.sprites.build(&mut self.spr_mem, &mut self.register, &mut self.mem);
                current_status = PpuStatus::RENDERING;
            }
        }
        current_status
    }
    pub fn has_been_updated(&mut self) -> bool {
        self.updated
    }
    pub fn clear_updated(&mut self) {
        self.updated = false;
    }
    pub fn force_update(&mut self) {
        self.updated = true;
    }
}

impl fmt::Display for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.mem)?;
        writeln!(f, "{}", self.spr_mem)?;
        writeln!(f, "{}", self.register)?;
        writeln!(f, "\n|----------PPU PALETTE--------------|")?;
        writeln!(f, "{}", self.palette)?;
        writeln!(f, "End ppu cycle : {}", self.dot)?;
        writeln!(f, "Last line rendered : {}", self.line)?;
        writeln!(f, "SpriteSet size: {}", self.sprites.len())?;
        write!(f, "TileSet size : {}", self.background.tiles.len())?;
        Ok(())
    }
}
