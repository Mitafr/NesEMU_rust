use crate::ppu::colors::COLORS;

use std::fmt;

pub trait PaletteVram {
    fn write_background(&mut self, v: u8);
    fn peek_color_background(&self, v: u8) -> u32;
}

pub struct Palette {
    pub background: Vec<u32>,
}

impl Palette {
    pub fn new() -> Palette {
        Palette {
            background: vec!(),
        }
    }
}

impl PaletteVram for Palette {
    fn write_background(&mut self, v: u8) {
        self.background.push(COLORS[v as usize]);
    }
    fn peek_color_background(&self, v: u8) -> u32 {
        if self.background.len() > v as usize {
            self.background[v as usize]
        } else {
            0
        }
    }
}

fn get_rgb(value: u32) -> (u8, u8, u8) {
    let r = ((value & 0xFF0000) >> 16) as u8;
    let g = ((value & 0x00FF00) >> 8) as u8;
    let b = ((value & 0x0000FF) >> 0) as u8;
    (r, g, b)
}


impl fmt::Display for Palette {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, v) in self.background.iter().enumerate() {
            write!(f, "{:06x?}   ", v)?;
        }
        writeln!(f, "")?;
        for (i, v) in self.background.iter().enumerate() {
            write!(f, "  {:02x?}     ", i)?;
        }
        Ok(())
    }
}