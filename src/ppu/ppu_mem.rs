use crate::memory::Memory;

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
}

impl Memory for PpuMem {
    fn get_size(&self) -> usize {
        self.size
    }

    fn peek(&mut self, i: usize) -> u8 {
        println!("Reading in VRAM at {:x?}", i);
        self.mem[i]
    }
    fn write(&mut self, i: usize, value: u8) -> u8 {
        println!("Writing in VRAM at {:x?} -> {:x?} ({:08b})", i, value, value);
        self.mem[i] = value;
        value
    }

    fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self {
        self
    }

    fn get_mem(&self) -> &[u8] {
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
            if *b != 0 {
                writeln!(f, "|\t{:04x?}\t => \t{:x?}\t|", i, b)?;
            }
        }
        writeln!(f, "|_______________________________|")?;
        Ok(())
    }
}