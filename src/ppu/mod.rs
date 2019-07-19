pub mod ppu_background;
pub mod ppu_colors;
pub mod ppu_mem;
pub mod ppu_palette;
pub mod ppu_register;
pub mod ppu_renderer;
pub mod ppu_tile;

use crate::ppu::ppu_background::Background;
use crate::ppu::ppu_mem::PpuMem;
use crate::memory::Memory;
use crate::ppu::ppu_register::Register;
use crate::ppu::ppu_register::PpuRegister;
use crate::ppu::ppu_renderer::PpuRenderer;
use crate::ppu::ppu_palette::Palette;
use crate::ppu::ppu_palette::PaletteVram;
use crate::ppu::ppu_tile::Tile;
use crate::rom::Cartbridge;

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
    renderer: Option<PpuRenderer>,
    pub tileset: Vec<Tile>,
    pub palette: Palette,
    cycle: u16,
    line: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            background: Background::new(),
            renderer: None,
            register: PpuRegister::new(),
            tileset: vec!(),
            mem: PpuMem::new(),
            palette: Palette::new(),
            cycle: 0,
            line: 0,
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.register.peek(i)
    }
    pub fn write(&mut self, i: usize, v: u8) -> u8 {
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
            tile.build_tile(&v);
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
            self.background.build(&mut self.mem);
        }
        if self.line >= 262 {
            self.line = 0;
        }
        self.line += 1;
        match &mut self.renderer {
            Some(r) => {
                if !r.is_open() || r.is_close_key_pressed() {
                    println!("BREAK!");
                    return PpuStatus::BREAK;
                }
                r.draw_window();
                PpuStatus::PROCESSING
            }
            None => PpuStatus::RENDERER_NOT_INITIALIZED
        }
    }
}

impl fmt::Display for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.register)?;
        write!(f, "{}", self.mem)?;
        writeln!(f, "\n|----------PPU PALETTE--------------|")?;
        writeln!(f, "{}", self.palette)?;
        writeln!(f, "End ppu cycle : {}", self.cycle)?;
        write!(f, "Las line rendered : {}", self.line)?;
        Ok(())
    }
}