pub mod background;
pub mod colors;
pub mod mem;
pub mod sprite;
pub mod palette;
pub mod register;
pub mod tile;

use crate::cpu::memory::Ram;
use crate::ppu::background::Background;
use crate::ppu::mem::PpuMem;
use crate::renderer::get_rgb;
#[allow(unused_imports)]
use crate::ppu::register::Register;
use crate::ppu::register::PpuRegister;
#[allow(unused_imports)]
use crate::ppu::palette::PaletteVram;
use crate::ppu::tile::Tile;
use crate::renderer::Renderer;
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
    pub background: Background,
    pub dot: i16,
    pub line: i16,
    pub tileset: Box<Vec<Tile>>,
    updated: bool,
    oam_addr: u8,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            background: Background::new(),
            register: PpuRegister::new(),
            mem: PpuMem::new(),
            tileset: Box::new(Vec::new()),
            dot: 0,
            line: 0,
            updated: false,
            oam_addr: 0,
        }
    }
    pub fn peek(&mut self, i: u16) -> u8 {
        self.register.peek(i, &mut self.mem)
    }
    pub fn write(&mut self, i: u16, v: u8) -> u8 {
        self.updated = true;
        self.register.write(i, v, &mut self.mem)
    }
    pub fn write_dma(&mut self, v: u8, ram: &mut Ram) -> u8 {
        self.updated = true;
        self.register.write_oam_data(v, &mut self.mem, ram);
        v
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
        self.register = PpuRegister::new();
        self.tileset = Box::new(Vec::new());
    }
    fn increment_y(&mut self) {
        let mut addr = self.register.get_addr();
        if addr & 0x7000 != 0x7000 {
            addr += 0x1000;
            self.register.set_addr_plain(addr);
        } else {
            addr = self.register.set_addr_plain(addr & 0x8FFF).get_addr();
            let mut y = (addr & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.register.set_addr_plain(addr ^ 0x0800);
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.register.set_addr_plain((addr & 0xFC1F) | (y << 5));
        }
    }
    fn increment_x(&mut self) {
        if (self.register.get_addr() & 0x001F) == 31 {
            let addr = self.register.set_addr_plain(self.register.get_addr() & 0xFFE0).get_addr();
            self.register.set_addr_plain(addr ^ 0x0400);
        } else {
            self.register.set_addr_plain(self.register.get_addr() + 1);
        }
    }
    fn copy_x(&mut self) {
        let addr = self.register.get_addr();
        let temp_addr = self.register.get_temp_addr();
        self.register.set_addr_plain((addr & 0xFBE0) | temp_addr & 0x041F);
    }
    fn copy_y(&mut self) {
        let addr = self.register.get_addr();
        let temp_addr = self.register.get_temp_addr();
        self.register.set_addr_plain((addr & 0x841F) | temp_addr & 0x7BE0);
    }
    pub fn run(&mut self, renderer: &mut Renderer) -> PpuStatus {
        let mut current_status = PpuStatus::PROCESSING;
        if self.line == -1 && self.dot == -1{
            self.background.clear_data();
        }
        self.dot += 1;
        if self.register.get_background_visibility() == 1 && self.register.get_sprite_visibility() == 1 {
            if self.line >= 1 && (self.dot >= 1 && self.dot <= 256 || self.dot >= 321 && self.dot <= 336) {
                match self.dot % 8 {
                    0 => self.background.store_tile_data(),
                    1 => self.background.fetch_nametable(&mut self.register),
                    3 => self.background.fetch_attribute(&mut self.mem, &mut self.register),
                    5 => self.background.fetch_loworder_byte(&mut self.mem, &mut self.register),
                    7 => self.background.fetch_highorder_byte(&mut self.mem, &mut self.register),
                    _ => {}
                }
            }
            if self.line == 261 && self.dot >= 280 && self.dot <= 304 {
                self.copy_y();
            }
            if self.line == 261 || self.line < 240 && self.register.get_background_visibility() == 1 {
                if self.dot % 8 == 0 && (self.dot >= 1 && self.dot <= 256 || self.dot >= 321 && self.dot <= 336) {
                    self.increment_x();
                }
                if self.dot == 256 {
                    self.increment_y();
                }
                if self.dot == 257 {
                    self.copy_x();
                }
            }
            // sprite logic
            if self.line >= 1 && self.line <= 239 && self.dot == 65 {
                //self.mem.spr_mem.clear_secondary();
                self.oam_addr = 0;
            }
            if self.line >= 1 && self.line <= 239 && self.dot >= 66 && self.dot <= 256 && self.oam_addr < 255 {
                // Read Y coordinate
                /*let oam_data = self.mem.spr_mem.get_oam()[self.oam_addr as usize];
                self.mem.spr_mem.push_secondary(oam_data);
                if y != 0 {
                    println!("Not empty sprite");
                }
                if !self.mem.spr_mem.is_secondary_full() {
                    if self.oam_addr >= 64 {
                        self.oam_addr += 4;
                    } else {
                        self.oam_addr += 1;
                    }
                }*/
            }
            if self.dot >= 1 && self.dot <= 256 && self.line < 240 {
                self.background.shit_tile_data();
                self.background.render(self.dot as u32, self.line as u32, renderer, &mut self.mem, &mut self.register);
            }
            if self.line >= 1 && self.line <= 239 && self.dot >= 257 && self.dot <= 320 {
                let spr_row = (self.dot - 257) / 8;
                let spr_line = self.line / 8;
                let spr_number = spr_row * 4 + spr_line as i16 * 4;
                let y = self.mem.spr_mem.get_oam()[spr_number as usize];
                let index = self.mem.spr_mem.get_oam()[(spr_number) as usize + 1];
                let attr = self.mem.spr_mem.get_oam()[(spr_number) as usize + 2];
                let hflip = attr & 64;
                let x = self.mem.spr_mem.get_oam()[(spr_number) as usize + 3];
                let xoffset = ((self.dot + 8 - 257) % ((spr_row + 1) * 8)) as u32;
                let yoffset = ((self.line + 8) % ((spr_line+1)*8)) as u32;
                let patternlow = self.mem.peek(((index as u32*16) + yoffset) as usize) << xoffset;
                let patternhi = self.mem.peek(((index as u32*16) + yoffset + 8) as usize) << xoffset;
                let low = (patternlow & 0x80) >> 7;
                let hi = (patternhi & 0x80) >> 6;
                let pixel = low | hi;
                let color = get_rgb(self.mem.palette.peek_color_sprite(attr & 3, pixel));
                renderer.set_pixel_rgb(x as u32 + xoffset, y as u32 + yoffset, color);
            }
        }
        if self.line == 241 && self.dot == 1 {
            self.register.set_vblank();
            if self.register.get_nmi_enable() == 0x1 {
                return PpuStatus::INTERRUPTNMI;
            }
        }
        if self.dot >= CYCLE_PER_LINE {
            self.dot = -1;
            self.line += 1;
            if self.line >= 262 {
                self.line = -1;
                self.register.clear_vblank();
                self.register.clear_spritehit();
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
        writeln!(f, "{}", self.register)?;
        writeln!(f, "End ppu cycle : {}", self.dot)?;
        writeln!(f, "Last line rendered : {}", self.line)?;
        Ok(())
    }
}
