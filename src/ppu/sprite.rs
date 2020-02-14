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