use crate::ppu::ppu_mem::PpuMem;
use crate::memory::Memory;

pub struct Background {
    content: Vec<u8>,
}

impl Background {
    pub fn new() -> Background {
        Background {
            content: vec!(),
        }
    }
    pub fn build(&mut self, vram: &mut PpuMem) {
        for i in 0x3F00..0x3F0F {
            let content = vram.peek(i);
            self.content.push(content);
        }
    }
    pub fn clear(&mut self) {
        self.content = vec!();
    }
}