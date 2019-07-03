use crate::cartbridge::Cartbridge;

use std::fmt;
use std::io::prelude::*;


pub trait Memory {
    fn get_size(&self) -> usize;

    fn peek(&mut self, i: usize) -> u8;
    fn write(&mut self, i: usize, value: u8) -> u8;
}

pub struct Ram {
    mem: [u8; 0xFFFF + 1],
    size: usize,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            mem: [0; 0xFFFF + 1],
            size: 0,
        }
    }
    pub fn load_program(&mut self, cartbridge: &mut Cartbridge) {
        for (i, byte) in cartbridge.get_program().bytes().enumerate() {
            let bit: u8 = byte.unwrap();
            if bit != 0 {
                self.size += 1;
                self.mem[i + 0x0600] = bit;
            }
        }
    }
}

impl Memory for Ram {
    fn peek(&mut self, i: usize) -> u8 {
        self.mem[i]
    }
    fn write(&mut self, i: usize, value: u8) -> u8 {
        println!("Writing => {:x?} at index {:x}", value, i);
        self.mem[i] = value;
        value
    }
    fn get_size(&self) -> usize {
        self.size
    }
}

impl fmt::Display for Ram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Size: {}", self.size)?;
        for (i, b) in self.mem.iter().enumerate() {
            if i >= 0x0600 && i <= 0x0602 {
                writeln!(f, "{:04x?} => {:x?} ", i, b)?;
            }
        }
        Ok(())
    }
}