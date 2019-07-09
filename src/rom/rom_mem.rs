use crate::memory::Memory;
use std::io::prelude::*;

pub struct Rom {
    mem: [u8; 0x8000],
    size: usize,
    offset: usize,
}

impl Rom {
    pub fn new() -> Rom {
        Rom {
            mem: [0; 0x8000],
            size: 0,
            offset: 0x8000,
        }
    }
}

impl Memory for Rom {
    fn get_size(&self) -> usize {
        self.size   
    }
    fn peek(&mut self, i: usize) -> u8 {
        self.mem[i - self.offset]
    }
    fn write(&mut self, i: usize, value: u8) -> u8 {
        println!("Writing in ROM => {:x?} at index {:x}", value, i);
        self.mem[i - self.offset] = value;
        value
    }
    fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self {
        for (i, byte) in data.bytes().enumerate() {
            let bit: u8 = byte.unwrap();
            if bit != 0 {
                self.size += 1;
                self.mem[i] = bit;
            }
        }
        self
    }
    fn get_mem(&self) -> &[u8] {
        &self.mem[0..self.size]
    }
}
