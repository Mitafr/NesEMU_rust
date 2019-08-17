use crate::ppu::mem::PpuMem;

use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Tile {
    pixels: Vec<Vec<u8>>,
    pub index: usize,
}

impl Tile {
    pub fn new() -> Tile {
        Tile {
            pixels: (0..8).into_iter().map(|_| vec![0; 8]).collect(),
            index: 0,
        }
    }
    pub fn build_tile(&mut self, slice: &[u8; 16], index: usize) {
        for j in 0..8 {
            let tilelow = slice[j];
            let tilehi = slice[j + 8];
            for k in 0..8 {
                let vv = (((0b1000_0000 >> k) & tilelow) / (0x80 >> k)) << 1;
                let vvv = ((0b1000_0000 >> k) & tilehi) / (0x80 >> k);
                self.pixels[j as usize][k as usize] = vv + vvv;
            }
            self.index = index;
        }
    }
    pub fn get_pixels(&self) -> &Vec<Vec<u8>> {
        &self.pixels
    }
    pub fn write_sprite(&mut self, v: u16) {
        //self.sprites.push(v);
    }
    pub fn set_index(&mut self, v: usize) {
        self.index = v;
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TILE")?;
        for (k,i) in self.pixels.iter().enumerate() {
            for (l,j) in i.iter().enumerate() {
                write!(f, "{} ", self.pixels[k][l])?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Deref for Tile {
    type Target = Vec<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.pixels
    }
}