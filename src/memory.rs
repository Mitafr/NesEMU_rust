use crate::cartbridge::Cartbridge;

use std::fs::File;
use std::fmt;
use std::io::prelude::*;

pub struct Memory {
    pub mem: [u8; 0xFFFF + 1],
    size: usize,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            mem: [0; 0xFFFF + 1],
            size: 0,
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.mem[i]
    }
    pub fn write(&mut self, i: usize, value: u8) {
        println!("Writing => {:x?} at index {:x}", value, i);
        self.mem[i] = value;
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


impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Size: {}", self.size)?;
        for (i, b) in self.mem.iter().enumerate() {
            if i >= 0x0600 && i <= 0x06FF {
                let value: u8 = b.clone();
                writeln!(f, "{:04x?} => {:x?} ", i, b)?;
            }
        }
        Ok(())
    }
}