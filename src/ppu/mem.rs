use crate::memory::Memory;
use crate::ppu::palette::Palette;
use crate::ppu::palette::PaletteVram;

use std::fmt;

pub struct PpuMem {
    mem: [u8; 0x4000],
    size: usize,
}

impl PpuMem {
    pub fn new() -> PpuMem {
        PpuMem {
            mem: [0; 0x4000],
            size: 0,
        }
    }
    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn peek(&mut self, i: usize) -> u8 {
        self.mem[i]
    }
    pub fn write_data<P: PaletteVram>(&mut self, i: usize, value: u8, palette: &mut P) -> u8 {
        match i {
            0x3F00...0x3F0F => {
                palette.write_background(value)
            }
            _ => self.mem[i] = value,
        };
        value
    }
    pub fn write_addr(&mut self, i: usize, value: u8) -> u8 {
        println!("Writing in VRAM at {:x?} -> {:x?} ({:08b})", i, value, value);
        self.mem[i] = value;
        value
    }

    pub fn get_mem(&self) -> &[u8] {
        &self.mem
    }
}

impl fmt::Display for PpuMem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "|----------PPU MEM--------------|")?;
        writeln!(f, "|--------------++---------------|")?;
        writeln!(f, "|\tadresse =>    value  |")?;
        writeln!(f, "|--------------++---------------|")?;
        for (i, b) in self.mem.iter().enumerate() {
            if i % 16 == 0 {    
                write!(f, "\n{:04x?}:", i)?;
            }
            if i % 4 == 0 {
                write!(f, "  ")?;
            }
            write!(f, "{:02x?} " , b)?;
        }
        Ok(())
    }
}