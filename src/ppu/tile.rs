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
                let low = (((0x80 >> k) & tilelow) / (0x80 >> k)) << 1;
                let hi = ((0x80 >> k) & tilehi) / (0x80 >> k);
                self.pixels[j as usize][k as usize] = low + hi;
            }
            self.index = index;
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TILE")?;
        for (k,i) in self.pixels.iter().enumerate() {
            for (l,_j) in i.iter().enumerate() {
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
