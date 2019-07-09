use crate::cpu::cpu_mem::CpuMem;
use crate::memory::Memory;
use crate::memory::Ram;
use crate::rom::Cartbridge;
use crate::ppu::Ppu;
use crate::ppu::ppu_register::PpuRegister;

use std::fmt;

pub struct CpuBus<'a> {
    ram: &'a mut Ram,
    rom: &'a mut Cartbridge,
    ppu: &'a mut Ppu,
}

pub trait Bus {
    fn peek(&mut self, i: usize) -> u8;
    fn write(&mut self, i: usize, v: u8) -> u8;
}

impl<'a> CpuBus<'a> {
    pub fn new(ram: &'a mut Ram, rom: &'a mut Cartbridge, ppu: &'a mut Ppu) -> CpuBus<'a> {
        Self {
            ram,
            rom,
            ppu,
        }
    }
}

impl<'a> Bus for CpuBus<'a> {
    fn peek(&mut self, i: usize) -> u8 {
        match i {
            0...0x1FFF => self.ram.peek(i),
            0x2000...0x3FFF => self.ppu.peek(i),
            0x8000...0xFFFF => self.rom.peek(i),
            _ => {
                println!("Wrong index => {:x?}", i);
                0
            }
        }
    }
    fn write(&mut self, i: usize, v: u8) -> u8 {
        match i {
            0...0x1FFF => self.ram.write(i, v),
            0x2000...0x3FFF => self.ppu.write(i, v),
            0x8000...0xFFFF => self.rom.write(i, v),
            _ => {
                println!("Wrong index => {:x?}", i);
                v
            }
        }
    }
}

impl<'a> fmt::Display for CpuBus<'a> {
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
        writeln!(f, "=======PPU=======")?;
        writeln!(f, "{}", self.ppu)?;
        writeln!(f, "=================")?;
        Ok(())
    }
}