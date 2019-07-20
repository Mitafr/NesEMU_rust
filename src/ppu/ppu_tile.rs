use crate::ppu::ppu_mem::PpuMem;

use std::fmt;
use std::ops::Deref;

pub struct Tile {
    tiles: Vec<Vec<u8>>,
}

impl Tile {
    pub fn new() -> Tile {
        Tile {
            tiles: (0..8).into_iter().map(|_| vec![0; 8]).collect(),
        }
    }
    pub fn build_tile(&mut self, slice: &[u8; 16]) {
        for j in 0..8 {
            let tilelow = slice[j];
            let tilehi = slice[j + 8];
            for k in 0..8 {
                let vv = (((0b1000_0000 >> k) & tilelow) / (0x80 >> k)) << 1;
                let vvv = ((0b1000_0000 >> k) & tilehi) / (0x80 >> k);
                self.tiles[j as usize][k as usize] = vv + vvv;
            }
        }
    }
    pub fn write_sprite(&mut self, v: u16) {
        //self.sprites.push(v);
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TILE")?;
        for (k,i) in self.tiles.iter().enumerate() {
            for (l,j) in i.iter().enumerate() {
                write!(f, "{} ", self.tiles[k][l])?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Deref for Tile {
    type Target = Vec<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.tiles
    }
}