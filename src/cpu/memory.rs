use crate::memory::Memory;
use std::io::prelude::*;

pub struct Ram {
    mem: [u8; 0x1FFF],
    size: usize,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            mem: [0; 0x1FFF],
            size: 0,
        }
    }
    #[allow(dead_code)]
    pub fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self {
        for (i, byte) in data.bytes().enumerate() {
            let bit: u8 = byte.unwrap();
            if bit != 0 {
                self.size += 1;
                self.mem[i] = bit;
            }
        }
        self
    }
}

impl Memory for Ram {
    fn get_size(&self) -> usize {
        self.size
    }
    fn peek(&self, i: u16) -> u8 {
        self.mem[i as usize]
    }
    fn write(&mut self, i: u16, value: u8) -> u8 {
        self.mem[i as usize] = value;
        value
    }
    fn get_mem(&self) -> &[u8] {
        &self.mem[0..self.size]
    }
}