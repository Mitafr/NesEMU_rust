use crate::memory::*;
use crate::rom::rom_mem::Rom;

use std::fmt;

pub struct CpuMem {
    ram: Ram,
    rom: Rom,
}

impl CpuMem {
    pub fn new() -> CpuMem {
        CpuMem {
            ram: Ram::new(),
            rom: Rom::new(),
        }
    }
}

impl Memory for CpuMem {
    fn get_size(&self) -> usize {
        self.ram.get_size() + self.rom.get_size()
    }
    fn peek(&mut self, i: usize) -> u8 {
        match i {
            0...0x1FFF => self.ram.peek(i),
            0x8000...0xFFFF => self.rom.peek(i),
            _ => {
                println!("Wrong index => {:x?}", i);
                0
            }
        }
    }
    fn write(&mut self, i: usize, value: u8) -> u8 {
        match i {
            0...0x1FFF => self.ram.write(i, value),
            0x8000...0xFFFF => self.rom.write(i, value),
            _ => {
                println!("Wrong index => {:x?}", i);
                value
            }
        }
    }
    fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self {
        self.rom.load_program(data);
        self
    }
    fn get_mem(&self) -> &[u8] {
        self.rom.get_mem()
    }
}

impl fmt::Display for CpuMem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "=======RAM=======\nSize: {}", self.ram.get_size())?;
        for (i, b) in self.ram.get_mem().iter().enumerate() {
            if *b != 0 {
                writeln!(f, "{:04x?} => {:x?} ", i, b)?;
            }
        }
        writeln!(f, "=======ROM=======\nSize: {}", self.rom.get_size())?;
        for (i, b) in self.rom.get_mem().iter().enumerate() {
            if *b != 0 {
                writeln!(f, "{:04x?} => {:x?} ", i, b)?;
            }
        }
        Ok(())
    }
}