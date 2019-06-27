use crate::memory::Memory;
use crate::cartbridge::Cartbridge;
use crate::opcode::*;
use crate::cpu_registers::*;
extern crate rand;

use rand::Rng;

use std::collections::HashMap;
use std::fmt;

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
    register: Registers,
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
    rom: Cartbridge,
    stack: Vec<u16>,
    pause: bool,
    opcodes: HashMap<u8, Opcode>
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            register: Registers::new(),
            delay_timer: 0,
            draw: false,
            index_register: 0,
            mem: Memory::new(),
            pc: 0x0600,
            r_a: 0x0,
            r_x: 0x0,
            r_y: 0x0,
            r_sr: 0x0,
            r_sp: 0b00110000,
            rom: Cartbridge::new(),
            stack: Vec::new(),
            pause: false,
            opcodes: init_opcodes(),
        }
    }

    pub fn init_mem(&mut self) {
        self.mem.load_program(&mut self.rom);
        println!("{}", self.mem);
    }
    pub fn init_rom(&mut self, rom: String) {
        self.rom.load_from_file(rom);
    }

    fn set_random_number(&mut self) {
        let mut rng = rand::thread_rng();
        let value: u8 = rng.gen_range(0, 255);
        self.mem.write(0x00FE, value);
    }

    pub fn run(&mut self) -> Result<(bool), String> {
        if !self.pause {
            self.set_random_number();
            let opcode = self.fetch_op_and_increment();
            println!("OPCODE at {:x?}: {:x?}", (self.pc - 1) as u8, opcode);
            let value = self.execute_op(opcode);
            println!("{}", self);
            self.pause = !value;
        }
        Ok(self.pause)
    }
    fn fetch_op(&mut self) -> u8 {
        self.mem.peek(self.pc)
    }
    fn fetch_op_and_increment(&mut self) -> u8 {
        let value = self.mem.peek(self.pc);
        self.pc += 1;
        value
    }
    fn read_16(&mut self) -> u16 {
        let hi = self.fetch_op_and_increment() as u16;
        let low = self.fetch_op_and_increment() as u16;
        (low << 8) | hi
    }
    fn read_zeropage(&mut self) -> u8 {
        let addr = self.fetch_op_and_increment();
        self.mem.peek(addr as usize)
    }
    fn get_flag(&mut self, flag: StatusFlags) -> bool {
        match flag {
            StatusFlags::CARRY => self.r_sp & 0b00000001 == 0b00000001,
            StatusFlags::ZERO => self.r_sp & 0b00000010 == 0b00000010,
            StatusFlags::INTERRUPT => self.r_sp & 0b00000100 == 0b00000100,
            StatusFlags::DECIMAL => self.r_sp & 0b00001000 == 0b00001000,
            StatusFlags::BREAK => self.r_sp & 0b00010000 == 0b00010000,
            StatusFlags::OVERFLOW => self.r_sp & 0b01000000 == 0b01000000,
            StatusFlags::NEGATIVE => self.r_sp & 0b10000000 == 0b10000000,
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
        }
    }
    fn fetch_relative_address(&mut self) -> u16 {
        let mut a = self.fetch_op_and_increment() as u16;
        if a < 0x80 {
            a += self.pc as u16
        } else {
            a += self.pc as u16 - 0x100
        }
        a
    }
    fn fetch_zeropage_x(&mut self) -> u16 {
        let value = self.mem.peek(self.pc) as u16;
        self.pc += 1;
        ((value + self.r_x as u16) & 0xFF) as u16
    }
    fn fetch_zeropage_y(&mut self) -> u16 {
        let value = self.mem.peek(self.pc) as u16;
        self.pc += 1;
        ((value + self.r_y as u16) & 0xFF) as u16
    }
    fn fetch_operand(&mut self, code: &Opcode) -> u16 {
        match code.mode {
            Addressing::Accumulator => 0x0000,
            Addressing::Implied => 0x0000,
            Addressing::Immediate => {
                let value = self.mem.peek(self.pc);
                self.pc += 1;
                value as u16
            },
            Addressing::Relative => self.fetch_relative_address(),
            Addressing::ZeroPage => {
                let value = self.mem.peek(self.pc);
                self.pc += 1;
                value as u16
            }
            Addressing::ZeroPageX => self.fetch_zeropage_x(),
            Addressing::ZeroPageY => self.fetch_zeropage_y(),
            _ => {0}
            /*Addressing::Absolute => fetch_word(registers, bus),     
            Addressing::AbsoluteX => fetch_absolute_x(registers, bus),
            Addressing::AbsoluteY => fetch_absolute_y(registers, bus),
            Addressing::PreIndexedIndirect => fetch_pre_indexed_indirect(registers, bus),
            Addressing::PostIndexedIndirect => fetch_post_indexed_indirect(registers, bus),
            Addressing::IndirectAbsolute => fetch_indirect_absolute(registers, bus),*/
        }
    }
    fn run_instructions(&mut self, n: usize) {
        for _i in 0..n {
            self.set_random_number();
            let opcode = self.fetch_op_and_increment();
            println!("OPCODE at {:x?}: {:x?}", self.pc as u8, opcode);
            let value = self.execute_op(opcode);
            // self.execute_op(opcode);
        }
    }
    fn lda_imm(&mut self, operand: u16) {
        // LDA imm
        self.r_a = operand as u8;
        self.set_flag(StatusFlags::NEGATIVE, operand & (1 << 7) != 0);
        self.set_flag(StatusFlags::ZERO, operand == 0);
    }
    fn lda(&mut self) {
        // LDA imm
        let value = self.fetch_op_and_increment();
        self.r_a = value;
        self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
        self.set_flag(StatusFlags::ZERO, value == 0);
    }
    fn execute_op(&mut self, opcode: u8) -> bool {
        match opcode {
             /* =====================
             * Register Instructions
             * =====================
             */
            0xaa => {
                // TAX (Transfer A reg to X reg)
                let value = self.r_a;
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x8a => {
                // TXA (Transfer X reg to A reg)
                let value = self.r_x;
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xca => {
                // DEX (Decremente X reg)
                let value = self.r_x.wrapping_sub(1);
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xe8 => {
                // INX (Incremente X reg)
                let value = self.r_x + 1;
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa8 => {
                // TAY (Tranfer A reg to Y reg)
                let value = self.r_a;
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x98 => {
                // TYA (Tranfer Y reg to A reg)
                let value = self.r_y;
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x88 => {
                // DEY (Decrement Y reg)
                let value = self.r_y - 1;
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xc8 => {
                // INY (Increment Y reg)
                let value = self.r_y + 1;
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xc6 => {
                // DEC Decrement memory zeropage
                let addr = self.fetch_op_and_increment();
                let value = self.mem.peek(addr as usize) - 1;
                self.mem.write(addr as usize, value);
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);

            }
             /* =====================
             * STACK Instructions
             * =====================
             */
            0x68 => {
                // PLA
                let value = self.stack.pop().unwrap() as u8;
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x48 => {
                // PHA
                let value = self.r_a as u16;
                self.stack.push(value);
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
             /* =====================
             * AD Instructions
             * =====================
             */
            0x69 => {
                // ADC imm
                let mut value = self.fetch_op_and_increment();
                if self.get_flag(StatusFlags::CARRY) {
                    value += 1;
                }
                let res: (u8, bool) = self.r_a.overflowing_add(value);
                self.r_a = res.0;
                self.set_flag(StatusFlags::CARRY, res.1);
                self.set_flag(StatusFlags::OVERFLOW, res.1);
                self.set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, res.0 == 0);
            }
            0x65 => {
                // ADC zeropage
                let mut value = self.read_zeropage();
                if self.get_flag(StatusFlags::CARRY) {
                    value += 1;
                }
                let res: (u8, bool) = self.r_a.overflowing_add(value);
                self.r_a = res.0;
                self.set_flag(StatusFlags::CARRY, res.1);
                self.set_flag(StatusFlags::OVERFLOW, res.1);
                self.set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, res.0 == 0);
            }
             /* =====================
             * SB Instructions
             * =====================
             */
            0xe9 => {
                // SBC imm
                let mut value = self.fetch_op_and_increment();
                if self.get_flag(StatusFlags::CARRY) {
                    value += 1;
                }
                let res: (u8, bool) = self.r_a.overflowing_sub(value);
                self.r_a = res.0;
                self.set_flag(StatusFlags::CARRY, res.1);
                self.set_flag(StatusFlags::OVERFLOW, res.1);
                self.set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, res.0 == 0);
            }
             /* =====================
             * ST Instructions
             * =====================
             */
            0x85 => {
                // STA zero page
                let addr = self.fetch_op_and_increment() as u16;
                self.mem.write(addr as usize, self.r_a);
            }
            0x95 => {
                // STA zeropage, X
                let addr = self.fetch_op_and_increment();
                let x = self.r_x;
                let res: (u8, bool) = x.overflowing_add(addr);
                let value = self.r_a;
                self.mem.write(res.0 as usize, value);
            }
            0x8d => {
                // STA absolute
                let value = self.read_16();
                self.mem.write(value as usize, self.r_a);
            }
            0x9d => {
                // STA Absolute,X
                let x = self.r_x as u16;
                let value = self.read_16();
                let addr = value | x;
                self.mem.write(addr as usize, self.r_a);
            }
            0x99 => {
                // STA Absolute,Y
                let y = self.r_y as u16;
                let value = self.read_16();
                let addr = value | y;
                self.mem.write(addr as usize, self.r_a);
            }
            0x81 => {
                // STA (Indirect,x)
                let xaddr = self.mem.peek(self.r_x as usize) as u16;
                let xaddr2 = self.mem.peek((self.r_x + self.r_x) as usize) as u16;
                let memaddr = (xaddr2 << 8) | xaddr;
                let value = self.r_a;
                self.pc += 1;
                self.mem.write(memaddr as usize, value);
            }
            0x91 => {
                // STA (Indirect),y
                let yaddr = self.mem.peek(self.r_y as usize) as u16;
                let yaddr2 = self.mem.peek((self.r_y + self.r_y) as usize) as u16;
                let memaddr = (yaddr2 << 8) | yaddr;
                let value = self.r_a;
                self.pc += 1;
                self.mem.write(memaddr as usize, value);
            }
            0x8e => {
                // STX Absolute
                let value = self.read_16();
                self.mem.write(value as usize, self.r_x);
            }
            0x8c => {
                // STY Absolute
                let value = self.read_16();
                self.mem.write(value as usize, self.r_y);
            }
             /* =====================
             * LD Instructions
             * =====================
             */
            0xa9 => {
                // LDA imm
                let value = self.fetch_op_and_increment();
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa5 => {
                // LDA zero page
                let addr = self.fetch_op_and_increment();
                let value = self.mem.peek(addr as usize);
                println!("{:x?}", addr);
                println!("{:x?}", value);
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value >> 7 == 1);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xb5 => {
                // LDA zero page,X
                let addr = self.fetch_op_and_increment();
                let x = self.r_x;
                let res: (u8, bool) = x.overflowing_add(addr);
                self.r_a = self.mem.peek(res.0 as usize);
                self.set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, res.0 == 0);
            }
            0xad => {
                // LDA absolute
                let addr = self.read_16();
                let value = self.mem.peek(addr as usize);
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xbd => {
                // LDA absolute,X
                let addr = self.read_16();
                let x = self.r_x;
                let res: u16 = addr.wrapping_add(x as u16);
                let value = self.mem.peek(res as usize);
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xb9 => {
                // LDA absolute,Y
                let addr = self.read_16();
                let y = self.r_y;
                let res: u16 = addr.wrapping_add(y as u16);
                let value = self.mem.peek(res as usize);
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa1 => {
                // LDA (Indirect,x)
                let addr = (self.fetch_op_and_increment() as u16).wrapping_add(self.r_x as u16);
                let hi = self.mem.peek(addr as usize) as u16;
                let low = self.mem.peek((addr + self.r_x as u16) as usize) as u16;
                let memaddr: u16 = (low << 8) | hi;
                let value = self.mem.peek(memaddr as usize);
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xb1 => {
                // LDA (Indirect),y
                let addr = self.fetch_op_and_increment() as u16;
                let hi = self.mem.peek(addr as usize) as u16;
                let low = self.mem.peek((addr + 1) as usize) as u16;
                let memaddr: u16 = (low << 8) | hi;
                let indirect: usize = memaddr.wrapping_add(self.r_y as u16) as usize;
                let value = self.mem.peek(indirect);
                self.r_a = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa2 => {
                // LDX imm
                let value = self.fetch_op_and_increment();
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa6 => {
                // LDX zero page
                let value = self.read_zeropage();
                self.r_x = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0xa0 => {
                // LDY imm
                let value = self.fetch_op_and_increment();
                self.r_y = value;
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
                self.set_flag(StatusFlags::ZERO, value == 0);
            }
            0x00 => {
                println!("BRK");
                return false;
            }
             /* =====================
             * Logical Instructions
             * =====================
             */
            0x4a => {
                // LSR Accumulator
                self.set_flag(StatusFlags::CARRY, self.r_a << 0 == 1);
                let value = self.r_a >> 1;
                self.r_a = value;
                self.set_flag(StatusFlags::ZERO, value == 0);
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
            }
            0x4e => {
                // LSR Absolute
                let value = self.read_16();
                return false;
            }
            0x29 => {
                // AND imm
                let value = self.fetch_op_and_increment();
                self.r_a &= value;
                self.set_flag(StatusFlags::ZERO, value == 0);
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
            }
            0xe0 => {
                // CPX imm
                let x = self.r_x;
                let m = self.fetch_op_and_increment();
                self.set_flag(StatusFlags::CARRY, x >= m);
                self.set_flag(StatusFlags::ZERO, x == m);
            }
            0xe4 => {
                // CPX zeropage
                let m = self.read_zeropage();
                let x = self.r_x;
                self.set_flag(StatusFlags::CARRY, x >= m);
                self.set_flag(StatusFlags::ZERO, x == m);
            }
            0xc0 => {
                // CPY imm
                let y = self.r_y;
                let m = self.fetch_op_and_increment();
                self.set_flag(StatusFlags::CARRY, y >= m);
                self.set_flag(StatusFlags::ZERO, y as i8 == m as i8);
            }
            0xc4 => {
                // CPY zeropage
                let m = self.read_zeropage();
                let y = self.r_y;
                self.set_flag(StatusFlags::CARRY, y >= m);
                self.set_flag(StatusFlags::ZERO, y as i8 == m as i8);
            }
            0xc9 => {
                // CMP imm
                let m = self.fetch_op_and_increment();
                let a = self.r_a;
                let value = a.wrapping_sub(m);
                self.set_flag(StatusFlags::CARRY, a >= m);
                self.set_flag(StatusFlags::ZERO, a == m);
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
            }
            0xc5 => {
                // CMP zeropage
                let m = self.read_zeropage();
                let a = self.r_a;
                let value = a.wrapping_sub(m);
                self.set_flag(StatusFlags::CARRY, a >= m);
                self.set_flag(StatusFlags::ZERO, a == m);
                self.set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
            }
            0x09 => {
                // ORA imm
                let a = self.r_a;
                let m = self.fetch_op_and_increment();
                panic!(String::from("Not yet finished"));
            }
            0x24 => {
                // BIT zeropage
                let m = self.read_zeropage();
                let a = self.r_a;
                let value = a & m;
                self.set_flag(StatusFlags::ZERO, value == 0);
                self.set_flag(StatusFlags::OVERFLOW, m & (1 << 6) != 0);
                self.set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
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
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0xf0 => {
                // BEQ Branch if equal
                println!("Check branch => {}", self.get_flag(StatusFlags::ZERO));
                if !self.get_flag(StatusFlags::ZERO) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0x10 => {
                // BPL Branch if Positive
                println!("Check branch => {}", self.get_flag(StatusFlags::NEGATIVE));
                if !self.get_flag(StatusFlags::NEGATIVE) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0x30 => {
                // BPL Branch if Minus
                println!("Check branch => {}", self.get_flag(StatusFlags::NEGATIVE));
                if self.get_flag(StatusFlags::NEGATIVE) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0x90 => {
                // BCC Branch if Carry clear
                println!("Check branch => {}", self.get_flag(StatusFlags::CARRY));
                if !self.get_flag(StatusFlags::CARRY) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0xb0 => {
                // BCS Branch if Carry set
                println!("Check branch => {}", self.get_flag(StatusFlags::CARRY));
                if self.get_flag(StatusFlags::CARRY) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0x50 => {
                // BVC Branch if Overflow clear
                println!("Check branch => {}", self.get_flag(StatusFlags::OVERFLOW));
                if !self.get_flag(StatusFlags::OVERFLOW) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0x70 => {
                // BCS Branch if Overflow set
                println!("Check branch => {}", self.get_flag(StatusFlags::OVERFLOW));
                if self.get_flag(StatusFlags::OVERFLOW) {
                    self.pc += 1;
                } else {
                    self.pc = self.fetch_relative_address() as usize;
                }
            }
            0x4c => {
                // JMP absolute
                let value = self.read_16();
                self.pc = value as usize;
            }
            0x6c => {
                // JMP ind
                let hi = self.fetch_op_and_increment() as u16;
                let low = hi + 1;
                let himem = self.mem.peek(hi as usize) as u16;
                let lowmem = self.mem.peek(low as usize) as u16;
                let value = (lowmem << 8) | himem;
                self.pc = value as usize;
            }
            0x20 => {
                // JSR Jump subroutine
                self.stack.push((self.pc + 1) as u16);
                let value = self.read_16();
                self.pc = value as usize;
            }
            0x60 => {
                // RTS return from subroutine
                let value = self.stack.pop().unwrap() + 1;
                self.pc = value as usize;
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
            0xea => {
                // NOP
                return true;
            }
            _ => {
                println!("Unrecognized Opcode: {:x?}", opcode);
                return false;
            }
        }
        return true;
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "==========================")?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "Flags: {:08b}", self.r_sp)?;
        writeln!(f, "Accumulator: ${:x?}", self.r_a)?;
        writeln!(f, "X: ${:x?}", self.r_x)?;
        writeln!(f, "Y: ${:x?}", self.r_y)?;
        writeln!(f, "Stack: ${}", self.stack.len())?;
        writeln!(f, "End PC: ${:x?}", self.pc)?;
        writeln!(f, "==========================")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_test_context(program: &Vec<u8>) -> Cpu {
        let mut cpu: Cpu = Cpu::new();
        cpu.rom.load_from_vec(program);
        cpu.init_mem();
        cpu
    }
    /*#[test]
    fn test_bit() {
        let program:Vec<u8> = vec!(0xa9, 0xff, // LDA #255
                                   0x85, 0x01, // STA $01
                                   0x24, 0x01, // BIT $01
                                   0xa9, 0x00, // LDA #00
                                   0x85, 0x02, // STA $02
                                   0x24, 0x02, // BIT $01
        );
        let mut cpu = create_test_context(&program);
        cpu.run_instructions(3);
        assert_eq!(cpu.get_flag(StatusFlags::ZERO), false);
        assert_eq!(cpu.get_flag(StatusFlags::NEGATIVE), true);
        assert_eq!(cpu.get_flag(StatusFlags::OVERFLOW), true);
        cpu.run_instructions(3);
        assert_eq!(cpu.get_flag(StatusFlags::ZERO), true);
        assert_eq!(cpu.get_flag(StatusFlags::NEGATIVE), false);
        assert_eq!(cpu.get_flag(StatusFlags::OVERFLOW), false);
    }*/
    #[test]
    fn test_lda_immediate() {
        let program:Vec<u8> = vec!(0xa9, 0xff); // LDA #$ff
        let mut cpu = create_test_context(&program);
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0xFF);
    }
    #[test]
    fn test_lda_zeroepage() {
        let program:Vec<u8> = vec!(0xa5, 0xa0); // LDA $a0
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0x00a0, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0x08);
    }
    #[test]
    fn test_lda_zeroepagex() {
        let program:Vec<u8> = vec!(0xb5, 0xa0); // LDA $a0,X
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0x00a5, 0x08);
        cpu.r_x = 0x05;
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0x08);
    }
    #[test]
    fn test_lda_absolute() {
        let program:Vec<u8> = vec!(0xad, 0x00, 0xF0);  // LDA $f000
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0xf000, 0x0F);
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0x0F);
    }
    #[test]
    fn test_lda_absolutex() {
        let program:Vec<u8> = vec!(0xbd, 0xa5, 0x00);  // LDA $00a5,X
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0x00aa, 0x0F);
        cpu.r_x = 0x05;
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0x0F);
    }
    #[test]
    fn test_lda_absolutey() {
        let program:Vec<u8> = vec!(0xb9, 0xa5, 0x00);  // LDA $00a5,Y
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0x00aa, 0x0F);
        cpu.r_y = 0x05;
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0x0F);
    }
    #[test]
    fn test_lda_indirectx() {
        let program:Vec<u8> = vec!(0xa1, 0xa0);  // LDA ($a0,X)
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0x00a1, 0xA3);
        cpu.mem.write(0x00a3, 0xA0);
        cpu.r_x = 0x01;
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0xA0);
    }
    #[test]
    fn test_lda_indirecty() {
        let program:Vec<u8> = vec!(0xb1, 0xa5);  // LDA ($a5),Y
        let mut cpu = create_test_context(&program);
        cpu.mem.write(0x00a4, 0xA0);
        cpu.mem.write(0x00a5, 0xA3);
        cpu.r_y = 0x01;
        cpu.run_instructions(1);
        assert_eq!(cpu.r_a, 0xA0);
    }
}