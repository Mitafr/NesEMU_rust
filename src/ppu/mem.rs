use crate::ppu::palette::PaletteVram;
use crate::ppu::palette::Palette;
use crate::ppu::sprite::SpriteMem;

use std::fmt;

pub struct PpuMem {
    vram: [u8; 0x4000],
    pub palette: Palette,
    pub nametable: [u8; 0x400],
    pub spr_mem: SpriteMem,
}

impl PpuMem {
    pub fn new() -> PpuMem {
        PpuMem {
            vram: [0; 0x4000],
            palette: Palette::new(),
            nametable: [0; 0x400],
            spr_mem: SpriteMem::new(),
        }
    }

    pub fn peek(&self, i: usize) -> u8 {
        match i {
            0x2000..=0x23FF => self.nametable[i - 0x2000],
            0x2400..=0x27FF => self.nametable[i - 0x2400],
            0x2800..=0x2BFF => self.nametable[i - 0x2800],
            0x2C00..=0x2FFF => self.nametable[i - 0x2C00],
            _ => self.vram[i]
        }
    }
    pub fn write(&mut self, i: usize, value: u8) -> u8 {
        match i {
            0x2000..=0x23FF => self.nametable[i - 0x2000] = value,
            0x2400..=0x27FF => self.nametable[i - 0x2400] = value,
            0x2800..=0x2BFF => self.nametable[i - 0x2800] = value,
            0x2C00..=0x2FFF => self.nametable[i - 0x2C00] = value,
            0x3F00..=0x3F0F => self.palette.write_background(value),
            0x3F10..=0x3F1F => self.palette.write_sprite(i, value),
            _ => self.vram[i] = value,
        };
        value
    }
    pub fn write_sprite_data(&mut self, i: usize, value: u8) {
        self.spr_mem.write_data(i, value);
    }
    pub fn set_cram(&mut self, value: Vec<u8>) {
        for (i, v) in value.iter().enumerate() {
            self.vram[i] = *v;
        }
    }
}

impl fmt::Display for PpuMem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "|----------PPU MEM--------------|")?;
        writeln!(f, "|--------------++---------------|")?;
        writeln!(f, "|\tadresse =>    value  |")?;
        writeln!(f, "|--------------++---------------|")?;
        for (i, b) in self.vram.iter().enumerate() {
            if i % 16 == 0 {    
                write!(f, "\n{:04x?}:", i)?;
            }
            if i % 4 == 0 {
                write!(f, "  ")?;
            }
            write!(f, "{:02x?} " , b)?;
        }
        writeln!(f, "\n|----------PPU NAMETABLE--------------|")?;
        for j in 0..256 / 8 {
            if j == 0 {
                write!(f, "   {:02x?} ", j)?;
                continue;
            }
            write!(f, " {:02x?} ", j)?;
        }
        for (i, b) in self.nametable.iter().enumerate() {
            if i % 32 == 0 {
                write!(f, "\n{:02x?}", i/32)?;
            }
            write!(f, " {:02x?} ", b)?;
        }
        writeln!(f, "\n|----------PPU SPRITE OAM--------------|")?;
        for (i, b) in self.spr_mem.get_oam().iter().enumerate() {
            if i % 16 == 0 {
                write!(f, "\n")?;
            }
            write!(f, " {:02x?} ", b)?;
        }
        writeln!(f, "\n|----------PPU SPRITE SECONDARY OAM--------------|")?;
        for (i, b) in self.spr_mem.get_secondary().iter().enumerate() {
            if i % 16 == 0 {
                write!(f, "\n")?;
            }
            write!(f, " {:02x?} ", b)?;
        }
        Ok(())
    }
}