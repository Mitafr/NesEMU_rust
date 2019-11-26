pub mod register;
pub mod opcode;
pub mod memory;
pub mod bus;

use crate::cpu::bus::Bus;
use crate::cpu::register::*;
use crate::cpu::opcode::*;
use crate::Cycle;
extern crate rand;

use rand::Rng;
use std::fmt;

#[derive(PartialEq)]
pub enum EmulationStatus {
    PROCESSING,
    ERROR,
    BREAK,
    RESET,
}

pub struct Cpu {
    pub opcode_counter: u32,
    register: Register,
    extra_cycle: Cycle,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut register = Register::new();
        register.set_flag(StatusFlags::INTERRUPT, true);
        Cpu {
            opcode_counter: 0,
            register: register,
            extra_cycle: 0,
        }
    }
    pub fn reset<B: Bus>(&mut self, bus: &mut B) -> Cycle {
        let hi = bus.peek(0xFFFC) as u16;
        let low = bus.peek(0xFFFC + 1) as u16;
        self.register.push_stack(hi as u8, bus);
        self.register.push_stack(low as u8, bus);
        let pc = ((low << 8) | hi) as u16;
        self.register.set_pc(pc);
        7
    }
    fn set_random_number<B: Bus>(&mut self, bus: &mut B) {
        let mut rng = rand::thread_rng();
        let value: u8 = rng.gen_range(0, 255);
        bus.write(0x00FE, value);
    }
    pub fn run<B: Bus>(&mut self, bus: &mut B) -> (Cycle, EmulationStatus) {
        self.extra_cycle = 0;
        self.set_random_number(bus);
        let pc = self.register.get_pc();
        let value = bus.peek(pc as usize);
        self.register.incr_pc();
        let opcode = opcode::OPCODES.get(&value).unwrap();
        let value = self.fetch_operand(bus, opcode);
        println!("OPCODE at {:x?}: {:x?}, value: {:x?}", pc, opcode, value);
        println!("A: {:x?},X: {:x?},Y: {:x?},P: {:x?},SP: {:x?},PC: {:x?},",
            self.register.get_a(),
            self.register.get_x(),
            self.register.get_y(),
            self.register.get_sr(),
            self.register.get_sp(),
            pc
        );
        println!("N : {}, V: {}, R: {}, B: {}, D: {}, I: {}, Z: {}, C: {}",
            self.register.get_flag(StatusFlags::NEGATIVE),
            self.register.get_flag(StatusFlags::OVERFLOW),
            self.register.get_flag(StatusFlags::UNUSED),
            self.register.get_flag(StatusFlags::BREAK),
            self.register.get_flag(StatusFlags::DECIMAL),
            self.register.get_flag(StatusFlags::INTERRUPT),
            self.register.get_flag(StatusFlags::ZERO),
            self.register.get_flag(StatusFlags::CARRY),
        );
        self.opcode_counter += 1;
        let res = self.execute_op(value, bus, opcode);
        println!("Extra Cycle: {:x?}", self.extra_cycle);
        (opcode.cycle as Cycle + self.extra_cycle, res)
    }
    #[allow(dead_code)]
    fn run_instructions<B: Bus>(&mut self, n: usize, bus: &mut B) {
        for _i in 0..n {
            self.set_random_number(bus);
            let value = bus.peek(self.register.get_pc() as usize);
            self.register.incr_pc();
            let opcode = opcode::OPCODES.get(&value).unwrap();
            let value = self.fetch_operand(bus, opcode);
            println!("OPCODE at {:x?}: {:x?}, value: {:x?}", self.register.get_pc(), opcode, value);
            println!("A: {:x?},X: {:x?},Y: {:x?},P: {:x?},SP: {:x?},PC: {:x?},",
                self.register.get_a(),
                self.register.get_x(),
                self.register.get_y(),
                self.register.get_sr(),
                self.register.get_sp(),
                self.register.get_pc()
            );
            /*println!("N : {}, V: {}, R: {}, B: {}, D: {}, I: {}, Z: {}, C: {}",
                self.register.get_flag(StatusFlags::NEGATIVE),
                self.register.get_flag(StatusFlags::OVERFLOW),
                self.register.get_flag(StatusFlags::UNUSED),
                self.register.get_flag(StatusFlags::BREAK),
                self.register.get_flag(StatusFlags::DECIMAL),
                self.register.get_flag(StatusFlags::INTERRUPT),
                self.register.get_flag(StatusFlags::ZERO),
                self.register.get_flag(StatusFlags::CARRY),
            );*/
            self.execute_op(value, bus, opcode);
        }
    }

    fn fetch_absolute_x<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let x = self.register.get_x();
        let addr = self.fetch_word(bus);
        addr.wrapping_add(x as u16)
    }

    fn fetch_absolute_y<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let y = self.register.get_y();
        let addr = self.fetch_word(bus);
        addr.wrapping_add(y as u16)
    }

    fn fetch_zeropage_x<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = self.fetch(bus);
        let x = self.register.get_x();
        value.wrapping_add(x as u16)
    }
    fn fetch_zeropage_y<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = self.fetch(bus);
        let y = self.register.get_y();
        value.wrapping_add(y as u16)
    }

    fn fetch_relative_address<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let a = self.fetch(bus);
        if a < 0x80 {
            a + self.register.get_pc() as u16
        } else {
            a + self.register.get_pc() as u16 - 0x100
        }
    }
    fn fetch_indirect_absolute<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let addr = self.fetch_word(bus);
        let upper = bus.peek((addr as usize) | ((addr) + 1) as usize) as u16;
        let low = bus.peek(addr as usize) as u16;
        low + (upper << 8) as u16
    }
    fn fetch_indexed_indirect<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let x = self.register.get_x() as u16;
        let addr = self.fetch(bus).wrapping_add(x);
        let hi = bus.peek(addr as usize) as u16;
        let low = bus.peek((addr + x) as usize) as u16;
        ((low << 8) | hi) as u16
    }
    fn fetch_indirect_indexed<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let addr = self.fetch(bus);
        let base_addr = (bus.peek(addr as usize) as usize) + ((bus.peek(((addr + 1) & 0x00FF) as usize) as usize) * 0x100);
        ((base_addr + (self.register.get_y() as usize)) & 0xFFFF) as u16
    }
    fn fetch_operand<B: Bus>(&mut self, bus: &mut B, opcode: &Opcode) -> u16 {
        match opcode.mode {
            Addressing::Accumulator => 0x0000,
            Addressing::Implied => 0x0000,
            Addressing::Immediate => self.fetch(bus),
            Addressing::ZeroPage => self.fetch(bus),
            Addressing::ZeroPageX => self.fetch_zeropage_x(bus),
            Addressing::ZeroPageY => self.fetch_zeropage_y(bus),
            Addressing::Absolute => self.fetch_word(bus),
            Addressing::AbsoluteX => self.fetch_absolute_x(bus),
            Addressing::AbsoluteY => self.fetch_absolute_y(bus),
            Addressing::Relative => self.fetch_relative_address(bus),
            Addressing::IndirectAbsolute => self.fetch_indirect_absolute(bus),
            Addressing::IndexedIndirect => self.fetch_indexed_indirect(bus),
            Addressing::IndirectIndexed => self.fetch_indirect_indexed(bus),
        }
    }
    fn fetch_word<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let hi = bus.peek(self.register.get_pc() as usize) as u16;
        self.register.incr_pc();
        let low = bus.peek(self.register.get_pc() as usize) as u16;
        self.register.incr_pc();
        ((low << 8) | hi) as u16
    }
    fn fetch<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = bus.peek(self.register.get_pc() as usize) as u16;
        self.register.incr_pc();
        value
    }

    fn execute_op<B: Bus>(&mut self, value: u16, bus: &mut B, opcode: &Opcode) -> EmulationStatus {
        match opcode.name {
            Instruction::ADC => {
                let mut res: (u8, bool);
                if opcode.mode == Addressing::Immediate {
                    res = self.register.get_a().overflowing_add(value as u8);
                } else {
                    res = self.register.get_a().overflowing_add(bus.peek(value as usize));
                }
                self.register.set_a(res.0)
                    .set_flag(StatusFlags::CARRY, res.1);
                if self.register.get_flag(StatusFlags::CARRY) {
                    res = self.register.get_a().overflowing_add(1);
                }
                self.register
                    .set_a(res.0)
                    .set_flag(StatusFlags::OVERFLOW, res.1)
                    .set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res.0 == 0);
            }
            Instruction::AND => {
                let a = self.register.get_a();
                if opcode.mode == Addressing::Immediate {
                    self.register.set_a(a & value as u8);
                } else {
                    self.register.set_a(a & bus.peek(value as usize) as u8);
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, value == 0);
            }
            Instruction::ASL => {
                let old: u8;
                if opcode.mode == Addressing::Accumulator {
                    old = self.register.get_a();
                    let a = self.register.set_a(old << 1).get_a();
                    self.register
                        .set_flag(StatusFlags::ZERO, a == 0)
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0);
                } else {
                    old = bus.peek(value as usize);
                    let m = bus.write(value as usize, old << 1);
                    self.register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                self.register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
            }
            Instruction::BCC => {
                if !self.register.get_flag(StatusFlags::CARRY) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BCS => {
                if self.register.get_flag(StatusFlags::CARRY) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BEQ => {
                if self.register.get_flag(StatusFlags::ZERO) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BIT => {
                let value = self.register.get_a() & value as u8;
                self.register
                    .set_flag(StatusFlags::ZERO, value == 0)
                    .set_flag(StatusFlags::OVERFLOW, value & (1 << 6) != 0)
                    .set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
            }
            Instruction::BMI => {
                if self.register.get_flag(StatusFlags::NEGATIVE) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BNE => {
                if !self.register.get_flag(StatusFlags::ZERO) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BPL => {
                if !self.register.get_flag(StatusFlags::NEGATIVE) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BRK => {
                self.register
                    .set_flag(StatusFlags::BREAK, true)
                    .set_flag(StatusFlags::INTERRUPT, true);
                return EmulationStatus::BREAK;
            }
            Instruction::BVC => {
                if !self.register.get_flag(StatusFlags::OVERFLOW) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::BVS => {
                if self.register.get_flag(StatusFlags::OVERFLOW) {
                    self.extra_cycle = if self.register.get_pc() & 0xFF00 != value & 0xFF00 { 2 } else { 1 };
                    self.register.set_pc(value);
                }
            }
            Instruction::CLC => {
                self.register.set_flag(StatusFlags::CARRY, false);
            }
            Instruction::CLD => {
                self.register.set_flag(StatusFlags::DECIMAL, false);
            }
            Instruction::CLI => {
                self.register.set_flag(StatusFlags::INTERRUPT, false);
            }
            Instruction::CLV => {
                self.register.set_flag(StatusFlags::OVERFLOW, false);
            }
            Instruction::CMP => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, a < m as u8)
                    .set_flag(StatusFlags::CARRY, a >= m as u8)
                    .set_flag(StatusFlags::ZERO, a == m as u8);
            }
            Instruction::CPX => {
                let x = self.register.get_x();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, x < m as u8)
                    .set_flag(StatusFlags::CARRY, x >= m as u8)
                    .set_flag(StatusFlags::ZERO, x == m as u8);
            }
            Instruction::CPY => {
                let y = self.register.get_y();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, y < m as u8)
                    .set_flag(StatusFlags::CARRY, y >= m as u8)
                    .set_flag(StatusFlags::ZERO, y == m as u8);
            }
            Instruction::DEC => {
                let res = bus.peek(value as usize).wrapping_sub(1);
                bus.write(value as usize, res);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::DEX => {
                let res = self.register.get_x().wrapping_sub(1);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_x(res);
            }
            Instruction::DEY => {
                let res = self.register.get_y().wrapping_sub(1);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_y(res);
            }
            Instruction::EOR => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                let res = a ^ m as u8;
                self.register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INC => {
                let res = bus.peek(value as usize).wrapping_add(1);
                bus.write(value as usize, res);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INX => {
                let res = self.register.get_x().wrapping_add(1);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_x(res);
            }
            Instruction::INY => {
                let res = self.register.get_y().wrapping_add(1);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_y(res);
            }
            Instruction::JMP => {
                self.register.set_pc(value);
            }
            Instruction::JSR => {
                let pc = self.register.get_pc() - 1;
                self.register.push_stack((pc >> 8) as u8, bus);
                self.register.push_stack(pc as u8, bus);
                self.register.set_pc(value);
            }
            Instruction::LDA => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_a(res as u8);
            }
            Instruction::LDX => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_x(res as u8);
            }
            Instruction::LDY => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_y(res as u8);
            }
            Instruction::LSR => {
                let old: u8;
                if opcode.mode == Addressing::Accumulator {
                    old = self.register.get_a();
                    let a = self.register.set_a(old >> 1).get_a();
                    self.register
                        .set_flag(StatusFlags::ZERO, a == 0)
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0);
                } else {
                    old = bus.peek(value as usize);
                    let m = bus.write(value as usize, old >> 1);
                    self.register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                self.register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
            }
            Instruction::NOP => return EmulationStatus::PROCESSING,
            Instruction::ORA => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                let res = a | m as u8;
                self.register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);

            }
            Instruction::PHA => {
                let a = self.register.get_a();
                self.register.push_stack(a, bus);
            }
            Instruction::PHP => {
                let status = self.register.get_sp();
                self.register.push_stack(status, bus);
            }
            Instruction::PLA => {
                let res = self.register.pop_stack(bus);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_a(res as u8);
            }
            Instruction::PLP => {
                let res = self.register.pop_stack(bus);
                self.register.set_sp(res as u8);
            }
            Instruction::ROL => {
                if opcode.mode == Addressing::Accumulator {
                    let old = self.register.get_a();
                    let mut a = old << 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        a += 1;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0)
                        .set_a(a);
                } else {
                    let old = bus.peek(value as usize);
                    let mut m = old << 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        m += 1;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
                    bus.write(value as usize, m);
                }
            }
            Instruction::ROR => {
                if opcode.mode == Addressing::Accumulator {
                    let old = self.register.get_a();
                    let mut a = old >> 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        a += 128;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0)
                        .set_a(a);
                } else {
                    let old = bus.peek(value as usize);
                    let mut m = old >> 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        m += 128;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0);
                    bus.write(value as usize, m);
                }
            }
            Instruction::RTI => {
                let sp = self.register.pop_stack(bus);
                self.register.set_sp(sp as u8);
                let hi = self.register.pop_stack(bus) as u16;
                let low = self.register.pop_stack(bus) as u16;
                self.register.set_pc((hi << 8) | low);
            }
            Instruction::RTS => {
                let low = self.register.pop_stack(bus) as u16;
                let hi = self.register.pop_stack(bus) as u16;
                self.register.set_pc((hi << 8) | low);
                self.register.incr_pc();
            }
            Instruction::SBC => {
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                if self.register.get_flag(StatusFlags::CARRY) {
                    m += 1;
                }
                let res: (u8, bool) = self.register.get_a().overflowing_sub(m as u8);
                self.register
                    .set_flag(StatusFlags::CARRY, res.1)
                    .set_flag(StatusFlags::OVERFLOW, res.1)
                    .set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res.0 == 0)
                    .set_a(res.0);
            }
            Instruction::SEC => {
                self.register.set_flag(StatusFlags::CARRY, true);
            }
            Instruction::SED => {
                self.register.set_flag(StatusFlags::DECIMAL, true);
            }
            Instruction::SEI => {
                self.register.set_flag(StatusFlags::INTERRUPT, true);
            }
            Instruction::STA => {
                bus.write(value as usize, self.register.get_a());
            }
            Instruction::STX => {
                bus.write(value as usize, self.register.get_x());
            }
            Instruction::STY => {
                bus.write(value as usize, self.register.get_y());
            }
            Instruction::TAX => {
                let a = self.register.get_a();
                self.register
                    .set_flag(StatusFlags::ZERO, a == 0)
                    .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0)
                    .set_x(a);
            }
            Instruction::TAY => {
                let a = self.register.get_a();
                self.register
                    .set_flag(StatusFlags::ZERO, a == 0)
                    .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0)
                    .set_y(a);
            }
            Instruction::TSX => {
                let st = self.register.get_sp();
                self.register
                    .set_flag(StatusFlags::ZERO, st == 0)
                    .set_flag(StatusFlags::NEGATIVE, st & (1 << 7) != 0)
                    .set_x(st as u8);
            }
            Instruction::TXA => {
                let x = self.register.get_x();
                self.register
                    .set_flag(StatusFlags::ZERO, x == 0)
                    .set_flag(StatusFlags::NEGATIVE, x & (1 << 7) != 0)
                    .set_a(x);
            }
            Instruction::TXS => {
                let x = self.register.get_x();
                self.register.set_sp(x);
                let sp = self.register.get_sp();
                self.register
                    .set_flag(StatusFlags::ZERO, sp == 0)
                    .set_flag(StatusFlags::NEGATIVE, sp & (1 << 7) != 0)
                    .set_x(sp);
            }
            Instruction::TYA => {
                let y = self.register.get_y();
                self.register
                    .set_flag(StatusFlags::ZERO, y == 0)
                    .set_flag(StatusFlags::NEGATIVE, y & (1 << 7) != 0)
                    .set_a(y);
            }
        }
        EmulationStatus::PROCESSING
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "===========CPU===========")?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "Flags: {:08b}", self.register.get_sr())?;
        writeln!(f, "Accumulator: ${:x?}", self.register.get_a())?;
        writeln!(f, "X: ${:x?}", self.register.get_x())?;
        writeln!(f, "Y: ${:x?}", self.register.get_y())?;
        writeln!(f, "Stack: ${:x?}", self.register.get_sp())?;
        writeln!(f, "End PC: ${:x?}", self.register.get_pc())?;
        writeln!(f, "Opcode counter: {}", self.opcode_counter)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Cartbridge;
    use crate::ppu::Ppu;
    use crate::memory::Memory;

    struct TestContext {
        cpu: Cpu,
        ram: cpu_mem::Ram,
        register: Register,
        ppu: Ppu,
        rom: Cartbridge
    }

    fn create_test_context(program: &Vec<u8>) -> TestContext {
        let mut cartbridge = Cartbridge::new();
        let cpu = Cpu::new();
        let cpu_ram = cpu_mem::Ram::new();
        let cpu_register = cpu_register::Register::new();
        let ppu = Ppu::new();
        cartbridge.load_from_vec(program);
        TestContext {
            cpu: cpu,
            ram: cpu_ram,
            register: cpu_register,
            ppu: ppu,
            rom: cartbridge
        }
    }
    #[test]
    fn test_adc_immediate() {
        let program:Vec<u8> = vec!(0x69, 0xa1); // ADC #$a1
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0xa1);
    }
    #[test]
    fn test_adc_zeropage() {
        let program:Vec<u8> = vec!(0x65, 0xa1); // ADC $a1
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a1, 0x08);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_zeropagex() {
        let program:Vec<u8> = vec!(0x75, 0xa1); // ADC $a1,X
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a2, 0x08);
        ctx.register.set_x(0x01);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolute() {
        let program:Vec<u8> = vec!(0x6D, 0xa1, 0x00); // ADC $00a1
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a1, 0x08);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolutex() {
        let program:Vec<u8> = vec!(0x7D, 0xa1, 0x00); // ADC $00a1
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a2, 0x08);
        ctx.register.set_x(0x01);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x08);
    }
    #[test]
    fn test_asl_accumulator() {
        let program:Vec<u8> = vec!(0x0A); // ASL A
        let mut ctx = create_test_context(&program);
        ctx.register.set_a(0x02);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x04);
    }
    #[test]
    fn test_asl() {
        let program:Vec<u8> = vec!(0x06, 0x04); // ASL $04
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0004, 0x02);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.ram.peek(0x0004), 0x04);
    }
    #[test]
    fn test_lda_immediate() {
        let program:Vec<u8> = vec!(0xa9, 0xff); // LDA #$ff
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0xFF);
    }
    #[test]
    fn test_lda_zeroepage() {
        let program:Vec<u8> = vec!(0xa5, 0xa0); // LDA $a0
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a0, 0x08);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_zeroepagex() {
        let program:Vec<u8> = vec!(0xb5, 0xa0); // LDA $a0,X
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a5, 0x08);
        ctx.register.set_x(0x05);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_absolute() {
        let program:Vec<u8> = vec!(0xad, 0x00, 0x1F);  // LDA $f000
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x1f00, 0x0F);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutex() {
        let program:Vec<u8> = vec!(0xbd, 0xa5, 0x00);  // LDA $00a5,X
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00aa, 0x0F);
        ctx.register.set_x(0x05);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutey() {
        let program:Vec<u8> = vec!(0xb9, 0xa5, 0x00);  // LDA $00a5,Y
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00aa, 0x0F);
        ctx.register.set_y(0x05);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_indirectx() {
        let program:Vec<u8> = vec!(0xa1, 0xa0);  // LDA ($a0,X)
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a1, 0xA3);
        ctx.ram.write(0x00a3, 0xA0);
        ctx.register.set_x(0x01);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0xA0);
    }
    #[test]
    fn test_lda_indirecty() {
        let program:Vec<u8> = vec!(0xb1, 0xa5);  // LDA ($a5),Y
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a4, 0xA0);
        ctx.ram.write(0x00a5, 0xA3);
        ctx.register.set_y(0x01);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 0xA0);
    }
    #[test]
    fn test_rol_accumulator() {
        let program: Vec<u8> = vec!(0xA9, 146, 0x2A, 0x2A);
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(2, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 36);
        assert!(ctx.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 73);
        assert!(!ctx.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_rol() {
        let program: Vec<u8> = vec!(0x26, 0x22, 0x26, 0x22);
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0022, 146);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.ram.peek(0x0022), 36);
        assert!(ctx.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.ram.peek(0x0022), 73);
        assert!(!ctx.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_ror_accumulator() {
        let program: Vec<u8> = vec!(0xA9, 147, 0x6A, 0x6A);
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(2, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 73);
        assert!(ctx.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.register.get_a(), 164);
        assert!(ctx.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_ror() {
        let program: Vec<u8> = vec!(0x66, 0x22, 0x66, 0x22);
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0022, 147);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.ram.peek(0x0022), 73);
        assert!(ctx.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.ram.peek(0x0022), 164);
        assert!(ctx.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_sta() {
        let program:Vec<u8> = vec!(0x8d, 0xa5, 0x00);  // STA $00a5
        let mut ctx = create_test_context(&program);
        ctx.register.set_a(0x05);
        let mut cpu_bus = cpu_bus::CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu);
        ctx.cpu.run_instructions(1, &mut cpu_bus, &mut ctx.register);
        assert_eq!(ctx.ram.peek(0xa5u8 as usize), 0x05);
    }
}