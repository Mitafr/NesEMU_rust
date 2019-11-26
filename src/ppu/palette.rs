use crate::ppu::colors::COLORS;

use std::fmt;

pub trait PaletteVram {
    fn write_background(&mut self, v: u8);
    fn peek_color_background(&self, v: u8) -> u32;

    fn write_sprite(&mut self, v: u8);
    fn peek_color_sprite(&self, v: u8) -> u32;
}

pub struct Palette {
    pub background: Vec<u32>,
    pub sprite: Vec<u32>,
}

impl Palette {
    pub fn new() -> Palette {
        Palette {
            background: vec!(),
            sprite: vec!(),
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
    fn write_sprite(&mut self, v: u8) {
        self.sprite.push(COLORS[v as usize]);
    }
    fn peek_color_sprite(&self, v: u8) -> u32 {
        if self.sprite.len() >= v as usize {
            self.sprite[v as usize]
        } else {
            0
        }
    }
}

impl fmt::Display for Palette {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.background.len() > 0 {
            writeln!(f, "Images")?;
            for v in self.background.iter() {
                write!(f, "#{:06x?}   ", v)?;
            }
        }
        if self.sprite.len() > 0 {
            writeln!(f, "\nSprites")?;
            for v in self.sprite.iter() {
                write!(f, "#{:06x?}   ", v)?;
            }
        }
        Ok(())
    }
}