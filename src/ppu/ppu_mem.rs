use crate::memory::Memory;

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
        self.mem[i]
    }
    fn write(&mut self, i: usize, value: u8) -> u8 {
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