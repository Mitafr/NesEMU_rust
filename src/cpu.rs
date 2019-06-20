use crate::memory::Memory;
extern crate rand;

use rand::Rng;

use std::thread;
use std::time::Duration;

pub enum Register {
    A,
    X,
    Y,
    SR,
    SP,
}

pub enum StatusFlags {
    CARRY((u8, bool)),
    ZERO(u8),
    INTERRUPT,
    DECIMAL,
    BREAK,
    OVERFLOW((u8, bool)),
    NEGATIVE(u8),
}

pub struct Cpu {
    delay_timer: u8,
    draw: bool,
    index_register: usize,
    mem: Memory,
    pc: usize,
    r_a: u8,
    r_x: u8,
    r_y: u8,
    r_sr: u8,
    r_sp: u8,
    rom: String,
    sound_timer: u8,
    stack: Vec<u16>,
}

impl Cpu {
    pub fn new(rom: String) -> Cpu {
        Cpu {
            delay_timer: 0,
            draw: false,
            index_register: 0,
            mem: Memory::new(),
            pc: 0x100,
            r_a: 0x0,
            r_x: 0x0,
            r_y: 0x0,
            r_sr: 0x0,
            r_sp: 0b00110000,
            rom: rom,
            sound_timer: 0,
            stack: Vec::new(),
        }
    }
    pub fn init(&mut self) -> Result<(), String> {
        self.mem.load_rom(&self.rom)?;
        println!("{}", self.mem);
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let opcode = self.fetch_op();
        println!("OPCODE: {:x?}", opcode);
        let value = self.execute_op(opcode);
        Ok(())
    }
    fn fetch_op(&mut self) -> u8 {
        let value = self.mem.peek(self.pc);
        self.pc += 1;
        value
    }
    fn fetch_op_and_increment(&mut self) -> u8 {
        let value = self.mem.peek(self.pc);
        self.pc += 1;
        value
    }
    fn set_flag(&mut self, flag: StatusFlags) {
        match flag {
            StatusFlags::CARRY(x) => {
                if x.1 {
                    self.r_sp |= 1 << 0;
                } else {
                    self.r_sp &= 0b11111110;
                }
            }
            StatusFlags::ZERO(x) => {
                if x == 0 {
                    self.r_sp |= 1 << 1;
                } else {
                    self.r_sp |= 0 << 1;
                }
            }
            StatusFlags::NEGATIVE(x) => {
                if x >> 7 == 1 {
                    self.r_sp |= 1 << 7;
                } else {
                    self.r_sp &= 0b01111111;
                }
            }
            StatusFlags::OVERFLOW(x) => {
                println!("Overflow: {:x?}", x);
                if x.1 {
                    self.r_sp |= 1 << 6;
                } else {
                    self.r_sp &= 0b10111111;
                }
            }
            _ => {}
        }
    }
    fn execute_op(&mut self, opcode: u8) {
        match opcode {
            0xaa => {
                self.r_x = self.r_a;
            }
            0xe8 => {
                self.r_x += 1;
            }
            0x65 => {
                // ADC zeropage
                let addr = self.fetch_op_and_increment();
                let value = self.mem.peek(addr as usize);
                let res: (u8, bool) = self.r_a.overflowing_add(value);
                self.r_a = res.0;
                self.set_flag(StatusFlags::CARRY(res));
                self.set_flag(StatusFlags::OVERFLOW(res));
                self.set_flag(StatusFlags::ZERO(self.r_a));
                self.set_flag(StatusFlags::NEGATIVE(self.r_a));
            }
            0x85 => {
                // STA imm
                let hi = self.fetch_op_and_increment() as u16;
                self.mem.write(hi as usize, self.r_a);
            }   
            0x69 => {
                // ADC imm
                let value = self.fetch_op_and_increment();
                if value >> 7 == 1 {
                    self.r_sp |= 1 << 0;
                }
                self.r_a = self.r_a.wrapping_add(value);
            }
            0x4e => {
                let hi = self.fetch_op_and_increment() as u16;
                let low = (hi << 8) | self.fetch_op_and_increment() as u16;
            }
            0xa9 => {
                // LDA imm
                let value = self.fetch_op_and_increment();
                self.set_flag(StatusFlags::NEGATIVE(value));
                self.r_a = value;
            }
            0x8d => {
                // STA absolute
                let hi = self.fetch_op_and_increment() as u16;
                let low = (hi << 8) | self.fetch_op_and_increment() as u16;
                self.mem.write(low as usize, self.r_a)
            }
            0x00 => {
                println!("==========================");
                println!("       NV-BDIZC");
                println!("Flags: {:08b}", self.r_sp);
                println!("Accumulator: {:x?}", self.r_a);
                println!("X: {:x?}", self.r_x);
                println!("Y: {:x?}", self.r_y);
                println!("End PC: {:x?}", self.pc);
                println!("==========================");
                panic!(String::from("BRK"));
            }
            _ => {
                panic!(String::from(format!("Unrecognized Opcode: {:x?}", opcode)));
            }
        }
    }
}