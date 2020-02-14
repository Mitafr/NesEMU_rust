use crate::controller::Controller;
use crate::cpu::memory::Ram;
use crate::memory::Memory;
use crate::rom::Cartbridge;
use crate::ppu::Ppu;

use std::fmt;

pub struct CpuBus<'a> {
    ram: &'a mut Ram,
    rom: &'a mut Cartbridge,
    ppu: &'a mut Ppu,
    controller: &'a mut Controller,
}

pub trait Bus {
    fn peek(&mut self, i: u16) -> u8;
    fn write(&mut self, i: u16, v: u8) -> u8;
}

impl<'a> CpuBus<'a> {
    pub fn new(ram: &'a mut Ram, rom: &'a mut Cartbridge, ppu: &'a mut Ppu, controller: &'a mut Controller) -> CpuBus<'a> {
        Self {
            ram,
            rom,
            ppu,
            controller,
        }
    }
}

impl<'a> Bus for CpuBus<'a> {
    fn peek(&mut self, i: u16) -> u8 {
        match i & 0xFFFF {
            0..=0x1FFF => self.ram.peek(i),
            0x2000..=0x3FFF => self.ppu.peek(i),
            0x4000..=0x4014 => panic!("Not implemented yet !"),
            0x4015 => 0,
            0x4016 => self.controller.read(),
            0x4017 => self.controller.read(),
            0x6000..=0x7FFF => self.ram.peek(i - 0x6000),
            0x8000..=0xBFFF if self.rom.get_size() <= 0x4010 => self.rom.peek(i-0x8000),
            0x8000..=0xBFFF => self.rom.peek(i - 0x8000),
            0xC000..=0xFFFF => self.rom.peek(i - 0xC000),
            _ => panic!("Wrong index => {:x?}", i),
        }
    }
    fn write(&mut self, i: u16, v: u8) -> u8 {
        match i {
            0..=0x1FFF => self.ram.write(i, v),
            0x2000..=0x3FFF => self.ppu.write(i, v),
            0x4000..=0x4013 => 0,
            0x4014 => self.ppu.write_dma(v, &mut self.ram),
            0x4015 => 0,
            0x4016 => self.controller.write(v),
            0x4017 => self.controller.write(v),
            0x6000..=0x7FFF => self.ram.write(i - 0x6000, v),
            _ => panic!("Wrong index => {:x?}", i),
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