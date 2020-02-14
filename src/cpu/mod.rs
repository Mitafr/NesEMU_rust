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

#[derive(PartialEq)]
pub enum CPUInterrupts {
    INTERRUPTNMI,
    // INTERRUPTIRQ,
    INTERRUPTNONE,
}

pub struct Cpu {
    register: Register,
    extra_cycle: Cycle,
    interrupt: CPUInterrupts,
    page_crossed: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        let register = Register::new();
        Cpu {
            register: register,
            extra_cycle: 0,
            interrupt: CPUInterrupts::INTERRUPTNONE,
            page_crossed: false,
        }
    }
    pub fn trigger_nmi(&mut self) {
        self.interrupt = CPUInterrupts::INTERRUPTNMI;
    }
    pub fn reset<B: Bus>(&mut self, bus: &mut B) -> Cycle {
        let hi = bus.peek(0xFFFC);
        let low = bus.peek(0xFFFC + 1);
        self.register.push_stack(hi, bus);
        self.register.push_stack(low, bus);
        let pc = ((low as u16) << 8) | hi as u16;
        self.register.set_pc(pc);
        println!("CPU: first PC : {:x?}", self.register.get_pc());
        7
    }
    fn write_random_number<B: Bus>(&mut self, bus: &mut B) {
        let mut rng = rand::thread_rng();
        let value: u8 = rng.gen_range(0, 255);
        bus.write(0x00FE, value);
    }
    pub fn run<B: Bus>(&mut self, bus: &mut B) -> (Cycle, EmulationStatus) {
        self.page_crossed = false;
        self.extra_cycle = 0;
        self.write_random_number(bus);
        let pc = self.register.get_pc();
        let value = bus.peek(pc);
        self.register.incr_pc();
        let opcode = opcode::OPCODES.get(&value).unwrap();
        let operand = self.fetch_operand(bus, opcode);
        let res = self.execute_op(operand, bus, opcode);
        match self.interrupt {
            //CPUInterrupts::INTERRUPTIRQ => {}
            CPUInterrupts::INTERRUPTNMI => {
                self.register.set_flag(StatusFlags::BREAK, false);
                self.register.push_stack((self.register.get_pc() >> 8) as u8, bus);
                self.register.push_stack((self.register.get_pc() & 0xFF) as u8, bus);
                self.register.push_stack(self.register.get_sr() as u8, bus);
                self.register.set_flag(StatusFlags::INTERRUPT, true);

                let hi = bus.peek(0xFFFB) as u16;
                let low = bus.peek(0xFFFA) as u16;
                self.register.set_pc((hi << 8) | low);
                self.extra_cycle = 7;
            }
            CPUInterrupts::INTERRUPTNONE => {}
        }
        self.interrupt = CPUInterrupts::INTERRUPTNONE;
        (opcode.cycle as Cycle + self.extra_cycle, res)
    }
    #[allow(dead_code)]
    fn run_instructions<B: Bus>(&mut self, n: usize, bus: &mut B) -> (Cycle, EmulationStatus) {
        let mut cycle: Cycle = 0;
        for _i in 0..n {
            self.extra_cycle = 0;
            self.write_random_number(bus);
            let value = bus.peek(self.register.get_pc());
            self.register.incr_pc();
            let opcode = opcode::OPCODES.get(&value).unwrap();
            let value = self.fetch_operand(bus, opcode);
            self.execute_op(value, bus, opcode);
            cycle += opcode.cycle as Cycle + self.extra_cycle;
        }
        (cycle, EmulationStatus::BREAK)
    }

    fn fetch_absolute_x<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let hi = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let low = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let addr = (low << 8) | hi;
        let res = addr.wrapping_add(self.register.get_x() as u16);
        self.page_crossed = addr & 0xFF00 != res & 0xFF00;
        res
    }

    fn fetch_absolute_y<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let hi = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let low = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let addr = (low << 8) | hi;
        let res = addr.wrapping_add(self.register.get_y() as u16);
        self.page_crossed = addr & 0xFF00 != res & 0xFF00;
        res
    }

    fn fetch_zeropage_x<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = bus.peek(self.register.get_pc());
        self.register.incr_pc();
        value.wrapping_add(self.register.get_x()) as u16
    }
    fn fetch_zeropage_y<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = bus.peek(self.register.get_pc());
        self.register.incr_pc();
        value.wrapping_add(self.register.get_y()) as u16
    }

    fn fetch_relative_address<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        if value < 0x80 {
            return value + self.register.get_pc();
        }
        value + self.register.get_pc() - 0x100
    }
    fn fetch_indirect_absolute<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let hi = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let low = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let addr = (low << 8) | hi;
        let upper = bus.peek((addr & 0xFF00) | ((((addr & 0xFF) + 1) & 0xFF))) as u16;
        (bus.peek(addr) as u16) + (upper << 8) as u16
    }
    fn fetch_indexed_indirect<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = bus.peek(self.register.get_pc());
        self.register.incr_pc();
        let addr = value.wrapping_add(self.register.get_x()) as u16;
        bus.peek(addr) as u16 + ((bus.peek((addr.wrapping_add(1)) & 0xFF) as u16) << 8) & 0xFFFF
    }
    fn fetch_indirect_indexed<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let addr = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let base_addr = bus.peek(addr) as u16 + ((bus.peek((addr + 1) & 0x00FF) as u16) * 0x100);
        let result = (base_addr.wrapping_add(self.register.get_y() as u16)) & 0xFFFF;
        self.page_crossed = base_addr & 0xFF00 != result & 0xFF00;
        result
    }
    fn fetch_word<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let hi = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        let low = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        (low << 8) | hi
    }
    fn fetch<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let value = bus.peek(self.register.get_pc()) as u16;
        self.register.incr_pc();
        value
    }
    fn fetch_operand<B: Bus>(&mut self, bus: &mut B, opcode: &Opcode) -> u16 {
        match opcode.mode {
            Addressing::Accumulator => return 0x0000,
            Addressing::Implied => return 0x0000,
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
    fn execute_op<B: Bus>(&mut self, value: u16, bus: &mut B, opcode: &Opcode) -> EmulationStatus {
        match opcode.name {
            Instruction::ADC => {
                let a = self.register.get_a() as u16;
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value) as u16;
                }
                let c: u16 = match self.register.get_flag(StatusFlags::CARRY) {
                    true => 1,
                    false => 0,
                };
                let res = a + m + c;
                self.register
                    .set_flag(StatusFlags::CARRY, res > 0xff)
                    .set_flag(StatusFlags::ZERO, res as u8 == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::OVERFLOW, (res ^ m) & ((res ^ a)) & 0x80 != 0)
                    .set_a(res as u8);
            }
            Instruction::AND => {
                let a = self.register.get_a();
                let mut m = value as u8;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value) as u8;
                }
                let res = a & m;
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_a(res);
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
                    old = bus.peek(value);
                    let m = bus.write(value, old << 1);
                    self.register
                        .set_flag(StatusFlags::ZERO, m == 0)
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
                let a = self.register.get_a();
                let m = bus.peek(value);
                let res =  a & m;
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::OVERFLOW, m & (1 << 6) != 0)
                    .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
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
                let mut m = value as u8;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value);
                }
                let res = a.wrapping_sub(m);
                if opcode.mode == Addressing::AbsoluteX || opcode.mode == Addressing::AbsoluteY || opcode.mode == Addressing::IndirectIndexed {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, a >= m)
                    .set_flag(StatusFlags::ZERO, a == m);
            }
            Instruction::CPX => {
                let x = self.register.get_x();
                let mut m = value as u8;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value);
                }
                let res = x.wrapping_sub(m);
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, x >= m)
                    .set_flag(StatusFlags::ZERO, x == m);
            }
            Instruction::CPY => {
                let y = self.register.get_y();
                let mut m = value as u8;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value);
                }
                let res = y.wrapping_sub(m);
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, y >= m)
                    .set_flag(StatusFlags::ZERO, y == m);
            }
            Instruction::DEC => {
                let old_value = bus.peek(value);
                let res = old_value.wrapping_sub(1);
                bus.write(value, res);
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
            Instruction::DCP => {
                let old_value = bus.peek(value);
                let res = old_value.wrapping_sub(1);
                bus.write(value, res);
                let a = self.register.get_a();
                if opcode.mode == Addressing::AbsoluteX || opcode.mode == Addressing::AbsoluteY || opcode.mode == Addressing::IndirectIndexed {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
                }
                self.register
                    .set_flag(StatusFlags::ZERO, a.wrapping_sub(res) == 0)
                    .set_flag(StatusFlags::NEGATIVE, a.wrapping_sub(res) & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, a >= res);
            }
            Instruction::EOR => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value) as u16;
                }
                let res = a ^ m as u8;
                self.register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INC => {
                let res = bus.peek(value).wrapping_add(1);
                bus.write(value, res);
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
            Instruction::ISB => {
                let res = bus.peek(value).wrapping_add(1);
                bus.write(value, res);
                let a = self.register.get_a();
                let c = match self.register.get_flag(StatusFlags::CARRY) {
                    true => 0,
                    false => 1,
                };
                let res: i16 = a as i16 - res as i16 - c as i16;
                self.register
                    .set_flag(StatusFlags::CARRY, res >= 0)
                    .set_flag(StatusFlags::ZERO, res as u8 == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::OVERFLOW, (a as u8 ^ value as u8) & ((res as u8 ^ a)) & 0x80 != 0)
                    .set_a(res as u8);
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
            Instruction::LAX => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value) as u16;
                }
                if opcode.mode == Addressing::AbsoluteX || opcode.mode == Addressing::AbsoluteY || opcode.mode == Addressing::IndirectIndexed {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_a(res as u8)
                    .set_x(res as u8);
            }
            Instruction::LDA => {
                let res = if opcode.mode == Addressing::Immediate { value } else { bus.peek(value) as u16 };
                if opcode.mode == Addressing::AbsoluteX || opcode.mode == Addressing::AbsoluteY || opcode.mode == Addressing::IndirectIndexed {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_a(res as u8);
            }
            Instruction::LDX => {   
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value) as u16;
                }
                if opcode.mode == Addressing::AbsoluteY {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_x(res as u8);
            }
            Instruction::LDY => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value) as u16;
                }
                if opcode.mode == Addressing::AbsoluteX {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
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
                    let a = self.register.set_a(old>>1).get_a();
                    self.register
                        .set_flag(StatusFlags::ZERO, a == 0)
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0);
                } else {
                    old = bus.peek(value);
                    let m = bus.write(value, old >> 1);
                    self.register
                        .set_flag(StatusFlags::ZERO, m == 0)
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                self.register.set_flag(StatusFlags::CARRY, old & 0x1 != 0);
            }
            Instruction::NOP => return EmulationStatus::PROCESSING,
            Instruction::ORA => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value) as u16;
                }
                let res = a | m as u8;
                if opcode.mode == Addressing::AbsoluteX || opcode.mode == Addressing::AbsoluteY || opcode.mode == Addressing::IndirectIndexed {
                    self.extra_cycle = if self.page_crossed { 1 } else { 0 };
                }
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
                self.register.set_flag(StatusFlags::BREAK, true);
                self.register.push_stack(self.register.get_sr(), bus);
                self.register.set_flag(StatusFlags::BREAK, false);
            }
            Instruction::PLA => {
                let res = self.register.pop_stack(bus);
                self.register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::PLP => {
                let res = self.register.pop_stack(bus);
                self.register
                    .set_sr(res as u8)
                    .set_flag(StatusFlags::UNUSED, true)
                    .set_flag(StatusFlags::BREAK, false);
            }
            Instruction::RLA => {
                let old = bus.peek(value);
                let mut m = old << 1;
                if self.register.get_flag(StatusFlags::CARRY) {
                    m += 1;
                }
                bus.write(value, m);
                let result = self.register.get_a() & m;
                self.register
                    .set_flag(StatusFlags::ZERO, result == 0)
                    .set_flag(StatusFlags::NEGATIVE, result & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0)
                    .set_a(result);

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
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0)
                        .set_a(a);
                } else {
                    let old = bus.peek(value);
                    let mut m = old << 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        m += 1;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0)
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                        bus.write(value, m);
                }
                self.register.set_flag(StatusFlags::ZERO, self.register.get_a() == 0);
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
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0)
                        .set_a(a);
                } else {
                    let old = bus.peek(value);
                    let mut m = old >> 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        m += 128;
                    }
                    self.register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0)
                        .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0);
                    bus.write(value, m);
                }
            }
            Instruction::RRA => {
                let a = self.register.get_a();
                let old = bus.peek(value);
                let mut m = old >> 1;
                if self.register.get_flag(StatusFlags::CARRY) {
                    m += 128;
                }
                self.register
                    .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0);
                bus.write(value, m);
                let c = match self.register.get_flag(StatusFlags::CARRY) {
                    true => 1,
                    false => 0,
                };
                let result = a as u16 + m as u16 + c as u16;
                self.register
                    .set_flag(StatusFlags::ZERO, result == 0)
                    .set_flag(StatusFlags::NEGATIVE, result & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, result > 0xFF)
                    .set_flag(StatusFlags::OVERFLOW, (a as u8 ^ result as u8) & ((value as u8 ^ result as u8)) & 0x80 != 0)
                    .set_a(result as u8);
            }
            Instruction::RTI => {
                let sr = self.register.pop_stack(bus);
                let low = self.register.pop_stack(bus) as u16;
                let hi = self.register.pop_stack(bus) as u16;
                self.register
                    .set_sr(sr)
                    .set_pc(low | (hi << 8))
                    .set_flag(StatusFlags::UNUSED, true);
            }
            Instruction::RTS => {
                let low = self.register.pop_stack(bus) as u16;
                let hi = self.register.pop_stack(bus) as u16;
                self.register.set_pc(low | (hi << 8));
                self.register.incr_pc();
            }
            Instruction::SAX => {
                let res = self.register.get_a() & self.register.get_x();
                bus.write(value, res);
            }
            Instruction::SBC => {
                let a = self.register.get_a();
                let mut m = value as u8;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value);
                }
                let c = match self.register.get_flag(StatusFlags::CARRY) {
                    true => 0,
                    false => 1,
                };
                let res: i16 = a as i16 - m as i16 - c as i16;
                self.register
                    .set_flag(StatusFlags::CARRY, res >= 0)
                    .set_flag(StatusFlags::ZERO, res as u8 == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::OVERFLOW, (a as u8 ^ m) & ((res as u8 ^ a)) & 0x80 != 0)
                    .set_a(res as u8);
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
            Instruction::SLO => {
                let old = bus.peek(value);
                let m = bus.write(value, old << 1);
                self.register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
                let result = self.register.get_a() | m;
                self.register
                    .set_flag(StatusFlags::ZERO, result == 0)
                    .set_flag(StatusFlags::NEGATIVE, result & (1 << 7) != 0)
                    .set_a(result);
            }
            Instruction::SRE => {
                let old = bus.peek(value);
                let m = bus.write(value, old >> 1);
                let result = self.register.get_a() ^ m;
                self.register
                    .set_flag(StatusFlags::ZERO, result == 0)
                    .set_flag(StatusFlags::NEGATIVE, result & (1 << 7) != 0)
                    .set_flag(StatusFlags::CARRY, old & 0x1 != 0)
                    .set_a(result);
            }
            Instruction::STA => {
                bus.write(value, self.register.get_a());
            }
            Instruction::STX => {
                bus.write(value, self.register.get_x());
            }
            Instruction::STY => {
                bus.write(value, self.register.get_y());
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
                let sp = self.register.get_sp();
                self.register
                    .set_flag(StatusFlags::ZERO, sp == 0)
                    .set_flag(StatusFlags::NEGATIVE, sp & (1 << 7) != 0)
                    .set_x(sp);
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Cartbridge;
    use crate::ppu::Ppu;
    use crate::CpuBus;
    use crate::controller::Controller;
    use crate::cpu::memory::Ram;
    #[allow(unused_imports)]
    use crate::cpu::register::Register;
    use crate::memory::Memory;


    struct TestContext {
        cpu: Cpu,
        ram: Ram,
        ppu: Ppu,
        rom: Cartbridge,
        controller: Controller
    }

    fn create_test_context(program: &Vec<u8>) -> TestContext {
        let mut cartbridge = Cartbridge::new();
        let cpu = Cpu::new();
        let cpu_ram = Ram::new();
        let ppu = Ppu::new();
        let controller = Controller::new();
        cartbridge.load_from_vec(program);
        TestContext {
            cpu: cpu,
            ram: cpu_ram,
            ppu: ppu,
            rom: cartbridge,
            controller
        }
    }
    #[test]
    fn reset_cpu() {
        let mut ctx = create_test_context(&Vec::new());
        ctx.rom.write(0xFFFC - 0xC000, 0x01);
        ctx.rom.write(0xFFFD - 0xC000, 0x80);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.reset(&mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_x(), 0);
        assert_eq!(ctx.cpu.register.get_a(), 0);
        assert_eq!(ctx.cpu.register.get_y(), 0);
        assert_eq!(ctx.cpu.register.get_pc(), 0x8001);
    }
    #[test]
    fn adc_immediate_should_add_to_acc() {
        let program:Vec<u8> = vec!(0x69, 0xfe, 0x69, 0x01); // ADC #$a1
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.register.set_a(0x01);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xff);
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::OVERFLOW));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x00);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::OVERFLOW));
        assert!(ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_adc_zeropage() {
        let program:Vec<u8> = vec!(0x65, 0xa1); // ADC $a1
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a1, 0x08);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_zeropagex() {
        let program:Vec<u8> = vec!(0x75, 0xa1); // ADC $a1,X
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a2, 0x08);
        ctx.cpu.register.set_x(0x01);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolute() {
        let program:Vec<u8> = vec!(0x6D, 0xa1, 0x00); // ADC $00a1
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a1, 0x08);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolutex() {
        let program:Vec<u8> = vec!(0x7D, 0xa1, 0x00); // ADC $00a1
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a2, 0x08);
        ctx.cpu.register.set_x(0x01);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_asl_accumulator() {
        let program:Vec<u8> = vec!(0x0A); // ASL A
        let mut ctx = create_test_context(&program);
        ctx.cpu.register.set_a(0x02);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x04);
    }
    #[test]
    fn test_asl() {
        let program:Vec<u8> = vec!(0x06, 0x04); // ASL $04
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0004, 0x02);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.ram.peek(0x0004), 0x04);
    }
    #[test]
    fn test_bit() {
        let program:Vec<u8> = vec!(0x24, 0x04); // BIT $04
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0004, 0xaa);
        ctx.cpu.register.set_a(0x40);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert!(ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        let program:Vec<u8> = vec!(0x24, 0x04); // BIT $04
        ctx = create_test_context(&program);
        ctx.cpu.register.set_a(0x40);
        ctx.ram.write(0x0004, 0x40);
        cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert!(ctx.cpu.register.get_flag(StatusFlags::OVERFLOW));
    }
    #[test]
    fn test_lsr_accumulator() {
        let program:Vec<u8> = vec!(0x4a); // LSR
        let mut ctx = create_test_context(&program);
        ctx.cpu.register.set_a(0x41);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x20);
        assert_eq!(ctx.cpu.register.get_flag(StatusFlags::CARRY), true);
    }
    #[test]
    fn test_plp() {
        let program:Vec<u8> = vec!(0x28); // PLP
        let mut ctx = create_test_context(&program);
        ctx.cpu.register.set_flag(StatusFlags::CARRY, true);
        ctx.cpu.register.set_flag(StatusFlags::NEGATIVE, true);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.register.push_stack(ctx.cpu.register.get_sr(), &mut cpu_bus);
        ctx.cpu.register.set_flag(StatusFlags::CARRY, false);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::ZERO));
    }
    #[test]
    fn test_lda_immediate() {
        let program:Vec<u8> = vec!(0xa9, 0xff); // LDA #$ff
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xFF);
    }
    #[test]
    fn test_bne() {
        let program:Vec<u8> = vec!(0xD0, 0x02, 0xa9, 0xff, 0x00); // BNE $02
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x0c);
        ctx.cpu.register.set_flag(StatusFlags::ZERO, true);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xff);
    }
    #[test]
    fn test_bmi() {
        let program:Vec<u8> = vec!(0x30, 0x02, 0xa9, 0xff, 0x00); // BMI $02
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x0c);
        ctx.cpu.register.set_flag(StatusFlags::NEGATIVE, false);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xff);
    }
    #[test]
    fn test_bpl() {
        let program:Vec<u8> = vec!(0x10, 0x02, 0xa9, 0xff, 0x00); // BPL $02
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x0c);
        ctx.cpu.register.set_flag(StatusFlags::NEGATIVE, true);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xff);
    }
    #[test]
    fn test_bvc() {
        let program:Vec<u8> = vec!(0x50, 0x02, 0xa9, 0xff, 0x00); // BVC $02
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x0c);
        ctx.cpu.register.set_flag(StatusFlags::OVERFLOW, true);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(3, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xff);
        assert_eq!(res.0, 2 + 2 + 7);
        ctx = create_test_context(&program);
        ctx.cpu.register.set_flag(StatusFlags::OVERFLOW, false);
        cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x00);
        assert_eq!(res.0, 2 + 1 + 7);
    }
    #[test]
    fn test_bvs() {
        let program:Vec<u8> = vec!(0x70, 0x02, 0xa9, 0xff, 0x00); // BVS $02
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x0c);
        ctx.cpu.register.set_flag(StatusFlags::OVERFLOW, false);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(3, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xff);
        assert_eq!(res.0, 2 + 2 + 7);
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x0c);
        ctx.cpu.register.set_flag(StatusFlags::OVERFLOW, true);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x00);
        assert_eq!(res.0, 2 + 1 + 7);
    }
    #[test]
    fn test_dec_zeropage() {
        let program:Vec<u8> = vec!(0xC6, 0x10, 0xC6, 0x10, 0xD6, 0x0F); // DEC #$10
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x10, 0x02);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x10), 0x01);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert_eq!(res.0, 5);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x10), 0x00);
        assert!(ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert_eq!(res.0, 5);
        ctx.cpu.register.set_x(0x01);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x10), 0xff);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert_eq!(res.0, 6);
    }
    #[test]
    fn test_dec_absolute() {
        let program:Vec<u8> = vec!(0xCE, 0x01, 0x10, 0xCE, 0x01, 0x10, 0xDE, 0x00, 0x10); // DEC #$10
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x1001, 0x02);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x1001), 0x01);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert_eq!(res.0, 6);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x1001), 0x00);
        assert!(ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert_eq!(res.0, 6);
        ctx.cpu.register.set_x(0x1);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x1001), 0xff);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::ZERO));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        assert_eq!(res.0, 7);
    }
    #[test]
    fn test_lda_zeroepage() {
        let program:Vec<u8> = vec!(0xa5, 0xa0); // LDA $a0
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a0, 0x08);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_zeroepagex() {
        let program:Vec<u8> = vec!(0xb5, 0xa0); // LDA $a0,X
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a5, 0x08);
        ctx.cpu.register.set_x(0x05);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_absolute() {
        let program:Vec<u8> = vec!(0xad, 0x00, 0x1F);  // LDA $f000
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x1f00, 0x0F);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutex() {
        let program:Vec<u8> = vec!(0xbd, 0xa5, 0x00);  // LDA $00a5,X
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00aa, 0x0F);
        ctx.cpu.register.set_x(0x05);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutey() {
        let program:Vec<u8> = vec!(0xb9, 0xa5, 0x00);  // LDA $00a5,Y
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00aa, 0x0F);
        ctx.cpu.register.set_y(0x05);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_indirectx() {
        let program:Vec<u8> = vec!(0xa1, 0xa0);  // LDA ($a0,X)
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a1, 0xA3);
        ctx.ram.write(0x00a3, 0xA0);
        ctx.cpu.register.set_x(0x01);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xA0);
    }
    #[test]
    fn test_lda_indirecty() {
        let program:Vec<u8> = vec!(0xb1, 0xa5);  // LDA ($a5),Y
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a4, 0xA0);
        ctx.ram.write(0x00a5, 0xA3);
        ctx.cpu.register.set_y(0x01);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xA0);
    }
    #[test]
    fn test_rol_accumulator() {
        let program: Vec<u8> = vec!(0xA9, 146, 0x2A, 0x2A);
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 36);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 73);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_rol() {
        let program: Vec<u8> = vec!(0x26, 0x22, 0x26, 0x22);
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0022, 146);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.ram.peek(0x0022), 36);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.ram.peek(0x0022), 73);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_ror_accumulator() {
        let program: Vec<u8> = vec!(0xA9, 147, 0x6A, 0x6A);
        let mut ctx = create_test_context(&program);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(2, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 73);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 164);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_ror() {
        let program: Vec<u8> = vec!(0x66, 0x22, 0x66, 0x22);
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x0022, 147);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.ram.peek(0x0022), 73);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.ram.peek(0x0022), 164);
        assert!(ctx.cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_sbc() {
        let program:Vec<u8> = vec!(0xe5, 0x05, 0xe9, 0x0b);  // SBC $a5 SBC #$0b
        let mut ctx = create_test_context(&program);
        ctx.cpu.register.set_a(0x00);
        ctx.ram.write(0x05, 0x01);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xfe);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::OVERFLOW));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::CARRY));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
        ctx.cpu.register.set_a(0x09);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.cpu.register.get_a(), 0xfd);
        assert!(!ctx.cpu.register.get_flag(StatusFlags::CARRY));
        assert!(!ctx.cpu.register.get_flag(StatusFlags::OVERFLOW));
        assert!(ctx.cpu.register.get_flag(StatusFlags::NEGATIVE));
    }
    #[test]
    fn test_sta() {
        let program:Vec<u8> = vec!(0x8d, 0xa5, 0x00);  // STA $00a5
        let mut ctx = create_test_context(&program);
        ctx.cpu.register.set_a(0x05);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(ctx.ram.peek(0xa5u16), 0x05);
    }
    #[test]
    fn test_stx() {
        let program:Vec<u8> = vec!(0x96, 0xa5, 0x8e, 0xa7, 0x00, 0x86, 0xaa);  // STX $05,Y
        let mut ctx = create_test_context(&program);
        ctx.ram.write(0x00a7, 0x00);
        ctx.cpu.register.set_y(0x02);
        ctx.cpu.register.set_x(0xaa);
        let mut cpu_bus = CpuBus::new(&mut ctx.ram, &mut ctx.rom, &mut ctx.ppu, &mut ctx.controller);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x00a7), 0xaa);
        assert_eq!(res.0, 4);
        ctx.cpu.register.set_x(0xff);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x00a7), 0xff);
        assert_eq!(res.0, 4);
        ctx.cpu.register.set_x(0x01);
        let res = ctx.cpu.run_instructions(1, &mut cpu_bus);
        assert_eq!(cpu_bus.peek(0x00aa), 0x01);
        assert_eq!(res.0, 3);
    }
}