use crate::ppu::ppu_mem::PpuMem;
use crate::memory::Memory;
use crate::ppu::ppu_colors::COLORS;

pub struct Background {
    pub palette: Vec<u32>,
}

impl Background {
    pub fn new() -> Background {
        Background {
            palette: vec!(),
        }
    }
    pub fn build(&mut self, vram: &mut PpuMem) {
        // fetch palette
        // fetch nametable
        /*for i in 0x2000..0x23FF {
            let content = vram.peek(i);
            self.palette.push(COLORS[content as usize]);
            if content != 0u8 {
                println!("Palette : {:x?}", COLORS[content as usize]);
            }
        }*/
    }
    pub fn clear(&mut self) {
        self.palette = vec!();
    }
}