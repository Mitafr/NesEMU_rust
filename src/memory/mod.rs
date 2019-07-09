use std::fmt;
use std::io::prelude::*;

pub trait Memory {
    fn get_size(&self) -> usize;

    fn peek(&mut self, i: usize) -> u8;
    fn write(&mut self, i: usize, value: u8) -> u8;

    fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self;

    fn get_mem(&self) -> &[u8];
}

pub struct Ram {
    mem: [u8; 0x1FFF],
    mirror: [u8; 0x1FFF],
    size: usize,
    offset: usize,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            mem: [0; 0x1FFF],
            mirror: [0; 0x1FFF],
            size: 0,
            offset: 0,
        }
    }
}

impl Memory for Ram {
    fn get_size(&self) -> usize {
        self.size
    }
    fn peek(&mut self, i: usize) -> u8 {
        self.mem[i - self.offset]
    }
    fn write(&mut self, i: usize, value: u8) -> u8 {
        if i != 0xfe {
            println!("Writing in RAM => {:x?} at index {:x}", value, i);
        }
        self.mem[i - self.offset] = value;
        value
    }
    fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self {
        for (i, byte) in data.bytes().enumerate() {
            let bit: u8 = byte.unwrap();
            if bit != 0 {
                self.size += 1;
                self.mem[i] = bit;
                self.mirror[i] = bit;
            }
        }
        self
    }
    fn get_mem(&self) -> &[u8] {
        &self.mem[0..self.size]
    }
}