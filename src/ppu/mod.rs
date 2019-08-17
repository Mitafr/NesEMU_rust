pub mod background;
// pub mod colors;
pub mod colors;
pub mod mem;
pub mod palette;
pub mod register;
pub mod renderer;
pub mod tile;

use crate::ppu::background::Background;
use crate::ppu::mem::PpuMem;
use crate::ppu::register::Register;
use crate::ppu::register::PpuRegister;
use crate::ppu::renderer::PpuRenderer;
use crate::ppu::palette::Palette;
use crate::ppu::palette::PaletteVram;
use crate::ppu::tile::Tile;
use crate::rom::Cartbridge;

use std::boxed::Box;
use std::fmt;
use std::option::Option;

#[derive(PartialEq)]
pub enum PpuStatus {
    BREAK,
    ERROR,
    PROCESSING,
    WAITING,
    RENDERER_NOT_INITIALIZED,
}

const CYCLE_PER_LINE: u16 = 341;

pub struct Ppu {
    register: PpuRegister,
    pub mem: PpuMem,
    background: Background,
    pub renderer: Option<PpuRenderer>,
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
            renderer: None,
            register: PpuRegister::new(),
            tileset: Box::new(Vec::new()),
            mem: PpuMem::new(),
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
        self.register.write(i, v, &mut self.mem, &mut self.palette)
    }
    pub fn init(&mut self, rom: &mut Cartbridge) {
        let renderer = PpuRenderer::new("NesEMU");
        for (i, v) in rom.get_character().iter().enumerate() {
            self.mem.write_data(i, *v, &mut self.palette);
        }
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
        self.renderer = Some(renderer);
    }
    pub fn run(&mut self, cycle: u16) -> PpuStatus {
        let cycle = self.cycle + cycle;
        if self.cycle < CYCLE_PER_LINE {
            self.cycle = cycle;
            return PpuStatus::WAITING;
        }
        self.cycle = cycle - CYCLE_PER_LINE;
        if self.line == 0 {
            self.background.clear();
        }
        if self.line < 240 {
            self.background.build(&mut self.mem, &mut self.register, &mut self.tileset);
        }
        if self.line >= 262 {
            self.line = 0;
        }
        self.line += 1;
        match &mut self.renderer {
            Some(r) => {
                if !r.is_open() || r.is_close_key_pressed() {
                    return PpuStatus::BREAK;
                }
                self.background.draw(r, &mut self.palette);
                r.draw_window();
                self.updated = false;
                PpuStatus::PROCESSING
            }
            None => PpuStatus::RENDERER_NOT_INITIALIZED
        }
    }
    pub fn has_been_updated(&mut self) -> bool {
        self.updated
    }
}

impl fmt::Display for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.register)?;
        //writeln!(f, "{}", self.mem)?;
        writeln!(f, "\n|----------PPU PALETTE--------------|")?;
        //writeln!(f, "{}", self.palette)?;
        writeln!(f, "End ppu cycle : {}", self.cycle)?;
        write!(f, "Las line rendered : {}", self.line)?;
        Ok(())
    }
}
