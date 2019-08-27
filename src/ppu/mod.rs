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
    BREAK,
    ERROR,
    PROCESSING,
    WAITING,
    RENDERING,
    RENDERERNOTINITIALIZED,
}

const CYCLE_PER_LINE: u16 = 341;

pub struct Ppu {
    register: PpuRegister,
    pub mem: PpuMem,
    pub spr_mem: SpriteMem,
    pub sprites: SpriteSet,
    pub background: Background,
    pub tileset: Box<Vec<Tile>>,
    pub palette: Palette,
    cycle: u16,
    line: u16,
    updated: bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            background: Background::new(),
            register: PpuRegister::new(),
            tileset: Box::new(Vec::new()),
            mem: PpuMem::new(),
            spr_mem: SpriteMem::new(),
            sprites: SpriteSet::new(),
            palette: Palette::new(),
            cycle: 0,
            line: 0,
            updated: false,
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.register.peek(i)
    }
    pub fn write(&mut self, i: usize, v: u8) -> u8 {
        self.updated = true;
        println!("Updated");
        self.register.write(i, v, &mut self.mem, &mut self.palette, &mut self.spr_mem)
    }
    pub fn init(&mut self, rom: &mut Cartbridge) {
        self.mem.set_cram(rom.get_character().to_vec());
        self.build_tiles();
    }
    fn build_tiles(&mut self) {
        let mut i = 0;
        let offset = 0x1000;
        if self.register.get_background_table() == 1 {
            i += 0x1000;
            // offset = 0x1000;
        }
        while i < offset + 0x1000 {
            let mut v = [0; 16];
            for j in 0..16 {
                v[j] = self.mem.peek(i as usize);
                i += 1;
            }
            let mut tile = Tile::new();
            tile.build_tile(&v, i);
            self.tileset.push(tile);
        }
    }
    pub fn run(&mut self, cycle: u16) -> PpuStatus {
        let cycle = self.cycle + cycle;
        if self.cycle < CYCLE_PER_LINE {
            self.cycle = cycle;
            self.updated = false;
            return PpuStatus::WAITING;
        }
        self.cycle = cycle - CYCLE_PER_LINE;
        self.line += 1;
        if self.line == 0 {
            self.background.clear();
        }
        // TODO:
        // build all background at one time (should be line by line)
        // if self.line < 240 {
        //      self.background.build_line(&mut self.mem, &mut self.register, &mut self.tileset);
        // }
        if self.line < 240 {
            println!("Rendering line {}", self.line);
        }
        if self.line == 240 {
            self.background.build(&mut self.mem, &mut self.register, &mut self.tileset);
        }
        if self.line == 241 {
            self.register.set_vblank();
            self.register.clear_spritehit();
        }
        if self.line >= 262 {
            self.line = 0;
            self.register.clear_vblank();
            self.register.clear_spritehit();
            self.sprites.build(&mut self.spr_mem, &mut self.register, &mut self.mem);
            return PpuStatus::RENDERING;
        }
        PpuStatus::PROCESSING
    }
    pub fn has_been_updated(&mut self) -> bool {
        self.updated
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
        writeln!(f, "End ppu cycle : {}", self.cycle)?;
        writeln!(f, "Last line rendered : {}", self.line)?;
        writeln!(f, "SpriteSet size: {}", self.sprites.len())?;
        write!(f, "TileSet size : {}", self.tileset.len())?;
        Ok(())
    }
}
