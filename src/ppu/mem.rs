use crate::ppu::palette::PaletteVram;

use std::fmt;

pub struct PpuMem {
    vram: [u8; 0x4000],
    // cram: Vec<u8>,
}

impl PpuMem {
    pub fn new() -> PpuMem {
        PpuMem {
            vram: [0; 0x4000],
        }
    }

    pub fn peek(&mut self, i: usize) -> u8 {
        self.vram[i]
    }
    pub fn write_data<P: PaletteVram>(&mut self, i: usize, value: u8, palette: &mut P) -> u8 {
        match i {
            0x3F00...0x3F0F => {
                palette.write_background(value)
            }
            0x3F10...0x3F1F => {
                palette.write_sprite(value)
            }
            _ => {
                println!("Writing in VRAM at {:x?} -> {:x?} ({:08b})", i, value, value);
                self.vram[i] = value;
            }
        };
        value
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
        Ok(())
    }
}