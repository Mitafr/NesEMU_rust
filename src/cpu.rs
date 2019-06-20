use crate::memory::Memory;
extern crate rand;

use std::fmt;

pub enum Register {
    A,
    X,
    Y,
    SR,
    SP,
}

pub enum StatusFlags {
    CARRY,
    ZERO,
    INTERRUPT,
    DECIMAL,
    BREAK,
    OVERFLOW,
    NEGATIVE,
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
            pc: 0x600,
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
        println!("OPCODE at {:x?}: {:x?}", self.pc, opcode);
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
    fn get_flag(&mut self, flag: StatusFlags) -> bool {
        match flag {
            StatusFlags::CARRY => {
                self.r_sp & 0b00000001 == 0b00000001
            }
            StatusFlags::ZERO => {
                self.r_sp & 0b00000010 == 0b00000010
            }
            StatusFlags::INTERRUPT => {
                self.r_sp & 0b00000100 == 0b00000100
            }
            StatusFlags::DECIMAL => {
                self.r_sp & 0b00001000 == 0b00001000
            }
            StatusFlags::BREAK => {
                self.r_sp & 0b00010000 == 0b00010000
            }
            StatusFlags::OVERFLOW => {
                self.r_sp & 0b01000000 == 0b01000000
            }
            StatusFlags::NEGATIVE => {
                self.r_sp & 0b10000000 == 0b00000000
            }
            _ => {false}
        }
    }
    fn set_flag(&mut self, flag: StatusFlags, value: bool) {
        match flag {
            StatusFlags::CARRY => {
                if value {
                    self.r_sp |= 1 << 0;
                } else {
                    self.r_sp &= 0b11111110;
                }
            }
            StatusFlags::ZERO => {
                if value {
                    self.r_sp |= 1 << 1;
                } else {
                    self.r_sp &= 0b11111101;
                }
            }
            StatusFlags::INTERRUPT => {
                if value {
                    self.r_sp |= 1 << 2;
                } else {
                    self.r_sp &= 0b11111011;
                }
            }
            StatusFlags::DECIMAL => {
                if value {
                    self.r_sp |= 1 << 3;
                } else {
                    self.r_sp &= 0b11110111;
                }
            }
            StatusFlags::BREAK => {
                if value {
                    self.r_sp |= 1 << 4;
                } else {
                    self.r_sp &= 0b11101111;
                }
            }
            StatusFlags::OVERFLOW => {
                if value {
                    self.r_sp |= 1 << 6;
                } else {
                    self.r_sp &= 0b10111111;
                }
            }
            StatusFlags::NEGATIVE => {
                if value {
                    self.r_sp |= 1 << 7;
                } else {
                    self.r_sp &= 0b01111111;
                }
            }
            _ => {}
        }
    }
    fn relative_address(&mut self) -> u16 {
        let mut a = self.fetch_op() as u16;
        if a < 0x80 {
            a += self.pc as u16;
        } else {
            a += self.pc as u16 - 0x100;
        }
        a
    }
    fn execute_op(&mut self, opcode: u8) {
        match opcode {
             /* =====================
             * Register Instructions
             * =====================
             */
            0xaa => {
                // TAX (Transfer A reg to X reg)
                let value = self.r_a;
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x8a => {
                // TXA (Transfer X reg to A reg)
                let value = self.r_x;
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xca => {
                // DEX (Decremente X reg)
                let value = self.r_x.wrapping_sub(1);
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xe8 => {
                // INX (Incremente X reg)
                let value = self.r_x + 1;
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa8 => {
                // TAY (Tranfer A reg to Y reg)
                let value = self.r_a;
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x98 => {
                // TYA (Tranfer Y reg to A reg)
                let value = self.r_y;
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x88 => {
                // DEY (Decrement Y reg)
                let value = self.r_y - 1;
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x88 => {
                // INY (Increment Y reg)
                let value = self.r_y + 1;
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
             /* =====================
             * AD Instructions
             * =====================
             */
            0x65 => {
                // ADC zeropage
                let addr = self.fetch_op_and_increment();
                let value = self.mem.peek(addr as usize);
                let res: (u8, bool) = self.r_a.overflowing_add(value);
                self.r_a = res.0;
                self.set_flag(StatusFlags::CARRY, res.1);
                self.set_flag(StatusFlags::OVERFLOW, res.1);
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x69 => {
                // ADC imm
                let value = self.fetch_op_and_increment();
                if value >> 7 == 1 {
                    self.r_sp |= 1 << 0;
                }
                self.r_a = self.r_a.wrapping_add(value);
            }
             /* =====================
             * ST Instructions
             * =====================
             */
            0x85 => {
                // STA imm
                let hi = self.fetch_op_and_increment() as u16;
                self.mem.write(hi as usize, self.r_a);
            }
            0x8d => {
                // STA absolute
                let hi = self.fetch_op_and_increment() as u16;
                let low = (hi << 8) | self.fetch_op_and_increment() as u16;
                self.mem.write(low as usize, self.r_a)
            }
            0x8e => {
                // STX Absolute
                let hi = self.fetch_op_and_increment() as u16;
                let low = (hi << 8) | self.fetch_op_and_increment() as u16;
                self.mem.write(low as usize, self.r_x)
            }
             /* =====================
             * LD Instructions
             * =====================
             */
            0xa9 => {
                // LDA imm
                let value = self.fetch_op_and_increment();
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x1a => {
                // LDA Indirect,x
                println!("{:x?}", self.fetch_op());
                panic!(String::from("BRK"));
            }
            0xa2 => {
                // LDX imm
                let value = self.fetch_op_and_increment();
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x00 => {
                println!("{}", self);
                panic!(String::from("BRK"));
            }
             /* =====================
             * Logical Instructions
             * =====================
             */
            0x4e => {
                // LSR
                let hi = self.fetch_op_and_increment() as u16;
                let low = (hi << 8) | self.fetch_op_and_increment() as u16;
            }
            0xe0 => {
                // CPX imm
                let x = self.r_x;
                let m = self.fetch_op();
                self.set_flag(StatusFlags::CARRY, x >= m);
                self.set_flag(StatusFlags::ZERO, x == m);
            }
            0xc9 => {
                // CMP imm
                let a = self.r_a;
                let m = self.fetch_op();
                let value = a.wrapping_sub(m);
                self.set_flag(StatusFlags::CARRY, a >= m);
                self.set_flag(StatusFlags::ZERO, a == m);
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
            }
             /* =====================
             * Branching Instructions
             * =====================
             */
            0xd0 => {
                // BNE Branch if not equal
                println!("Check branch => {}", self.get_flag(StatusFlags::ZERO));
                if self.get_flag(StatusFlags::ZERO) {
                    self.pc += 1;
                } else {
                    self.pc = self.relative_address() as usize;
                }
            }
            0x6c => {
                // JMP ind
                let addr = self.fetch_op_and_increment();
                println!("{:x?}", addr);
                println!("{:x?}", self.mem.peek(addr as usize));
                panic!(String::from("Not implemented"));
            }
             /* =====================
             * Flag Instructions
             * =====================
             */
            0x18 => {
                // CLC
                self.set_flag(StatusFlags::CARRY, false);
            }
            0x38 => {
                // SEC
                self.set_flag(StatusFlags::CARRY, true);
            }
            0x58 => {
                // CLI
                self.set_flag(StatusFlags::INTERRUPT, false);
            }
            0x78 => {
                // SEI
                self.set_flag(StatusFlags::INTERRUPT, true);
            }
            0xB8 => {
                // CLV
                self.set_flag(StatusFlags::OVERFLOW, false);
            }
            0xD8 => {
                // CLD
                self.set_flag(StatusFlags::DECIMAL, false);
            }
            0xF8 => {
                // SED
                self.set_flag(StatusFlags::DECIMAL, true);
            }
            _ => {
                println!("{}", self);
                panic!(String::from(format!("Unrecognized Opcode: {:x?}", opcode)));
            }
        }
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "==========================");
        writeln!(f, "       NV-BDIZC");
        writeln!(f, "Flags: {:08b}", self.r_sp);
        writeln!(f, "Accumulator: {:x?}", self.r_a);
        writeln!(f, "X: {:x?}", self.r_x);
        writeln!(f, "Y: {:x?}", self.r_y);
        writeln!(f, "End PC: {:x?}", self.pc);
        writeln!(f, "==========================");
        Ok(())
    }
}