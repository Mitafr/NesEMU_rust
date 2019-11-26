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
    pub register: PpuRegister,
    pub mem: PpuMem,
    pub spr_mem: SpriteMem,
    pub sprites: SpriteSet,
    pub background: Background,
    pub tileset: Box<Vec<Tile>>,
    pub palette: Palette,
    dot: u16,
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
            dot: 0,
            line: 0,
            updated: false,
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
    }
    pub fn run(&mut self, cycle: u16) -> PpuStatus {
        let current_cycle = self.dot;
        if current_cycle == 0 {
            self.dot = 1;
            return PpuStatus::WAITING;
        }
        if self.line == 0 {
            self.background.clear();
        }
        /*println!("Dot : {}", self.dot);
        println!("Line : {}", self.line);*/
        if self.line < 240 {
            match current_cycle % 8 {
                1 => self.background.fetch_nametable(self.dot % 256, self.line % 240, &mut self.mem, &mut self.register),
                3 => self.background.fetch_attribute(self.dot % 256, self.line % 240, &mut self.mem, &mut self.register),
                5 => self.background.fetch_loworder_byte(self.dot % 256, self.line % 240, &mut self.mem, &mut self.register),
                7 => self.background.fetch_highorder_byte(self.dot % 256, self.line % 240, &mut self.mem, &mut self.register),
                _ => {}
            }
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
        self.dot += 1;
        if current_cycle > 340 {
            self.dot = 0;
            self.line += 1;
            if self.line > 261 {
                self.line = 0;
            }
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
        writeln!(f, "End ppu cycle : {}", self.dot)?;
        writeln!(f, "Last line rendered : {}", self.line)?;
        writeln!(f, "SpriteSet size: {}", self.sprites.len())?;
        write!(f, "TileSet size : {}", self.tileset.len())?;
        Ok(())
    }
}
