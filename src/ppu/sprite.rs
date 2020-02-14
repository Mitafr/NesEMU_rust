use std::fmt;

pub struct SpriteMem {
    oam: [u8; 0x100],
    secondary_oam: [u8; 0x20],
    // secondary_index: u8,
}

impl SpriteMem {
    pub fn new() -> SpriteMem {
        SpriteMem {
            oam: [0; 0x100],
            secondary_oam: [0; 0x20],
            // secondary_index: 0,
        }
    }

    /*pub fn peek(&mut self, i: usize) -> u8 {
        self.mem[i]
    }*/
    pub fn write_data(&mut self, i: usize, value: u8) -> u8 {
        self.oam[i] = value;
        value
    }
    pub fn get_oam(&self) -> &[u8] {
        &self.oam
    }
    pub fn get_secondary(&self) -> &[u8] {
        &self.secondary_oam
    }
    /*pub fn push_secondary(&mut self, v: u8) {
        self.secondary_oam[self.secondary_index as usize] = v;
        self.secondary_index += 1;
    }
    pub fn is_secondary_full(&self) -> bool {
        self.secondary_index == 9
    }
    pub fn clear_secondary(&mut self) {
        self.secondary_oam = [0;0x20];
        self.secondary_index = 0;
    }*/
}

impl fmt::Display for SpriteMem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "|----------PPU SPRITEMEM--------------|")?;
        writeln!(f, "|--------------++---------------|")?;
        writeln!(f, "|\tadresse =>    value  |")?;
        writeln!(f, "|--------------++---------------|")?;
        for (i, b) in self.oam.iter().enumerate() {
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
pub struct Sprite {
    pub coord_y: u8,
    pub coord_x: u8,
    #[allow(dead_code)]
    index: u8,
    pixels: Vec<Vec<u8>>,
    #[allow(dead_code)]
    flags: u8,
}

impl Sprite {
    #[allow(dead_code)]
    pub fn new(value: u8) -> Sprite {
        Sprite {
            coord_x: value & 0b0000_1000,
            coord_y: value & 0b0000_0001,
            index: value & 0b0000_0010,
            pixels: (0..8).into_iter().map(|_| vec![0; 8]).collect(),
            flags: value & 0b0000_0100,
        }
    }
    #[allow(dead_code)]
    pub fn build_sprite(&mut self, slice: &[u8; 16]) {
        for j in 0..8 {
            let tilelow = slice[j];
            let tilehi = slice[j + 8];
            for k in 0..8 {
                let vv = (((0b1000_0000 >> k) & tilelow) / (0x80 >> k)) << 1;
                let vvv = ((0b1000_0000 >> k) & tilehi) / (0x80 >> k);
                self.pixels[j as usize][k as usize] = vv + vvv;
            }
        }
    }
    #[allow(dead_code)]
    pub fn get_index(&self) -> u8 {
        self.index
    }
}
