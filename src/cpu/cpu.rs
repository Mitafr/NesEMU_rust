use crate::memory::*;
use crate::cartbridge::Cartbridge;
use crate::opcode::*;
use crate::cpu::cpu_registers::*;
extern crate rand;

use rand::Rng;

use std::fmt;

fn fetch_absolute_x<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let x = reg.get_x();
    let addr = fetch_word(reg, mem);
    addr.wrapping_add(x as u16) & 0xFFFF
}

fn fetch_absolute_y<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let y = reg.get_y();
    let addr = fetch_word(reg, mem);
    addr.wrapping_add(y as u16) & 0xFFFF
}

fn fetch_zeropage_x<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let value = fetch(reg, mem);
    let x = reg.get_x();
    ((value + x as u16) & 0xFF) as u16
}
fn fetch_zeropage_y<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let value = fetch(reg, mem);
    let y = reg.get_y();
    ((value + y as u16) & 0xFF) as u16
}

fn fetch_relative_address<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let a = fetch(reg, mem);
    if a < 0x80 {
        a + reg.get_pc() as u16
    } else {
        a + reg.get_pc() as u16 - 0x100
    }
}
fn fetch_indirect_absolute<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let hi = fetch(reg, mem);
    let low = hi + 1;
    let himem = mem.peek(hi as usize) as u16;
    let lowmem = mem.peek(low as usize) as u16;
    (lowmem << 8) | himem
}
fn fetch_indexed_indirect<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let x = reg.get_x() as u16;
    let addr = fetch(reg, mem).wrapping_add(x);
    let hi = mem.peek(addr as usize) as u16;
    let low = mem.peek((addr + x) as usize) as u16;
    ((low << 8) | hi) as u16
}
fn fetch_indirect_indexed<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let addr = fetch(reg, mem);
    let hi = mem.peek(addr as usize) as u16;
    let low = mem.peek((addr + 1) as usize) as u16;
    let memaddr: u16 = (low << 8) | hi;
    memaddr.wrapping_add(reg.get_y() as u16)
}
fn fetch_operand<T: CpuRegisters, M: Memory>(code: &Opcode, reg: &mut T, mem: &mut M) -> u16 {
    match code.mode {
        Addressing::Accumulator => 0x0000,
        Addressing::Implied => 0x0000,
        Addressing::Immediate => fetch(reg, mem),
        Addressing::ZeroPage => fetch(reg, mem),
        Addressing::ZeroPageX => fetch_zeropage_x(reg, mem),
        Addressing::ZeroPageY => fetch_zeropage_y(reg, mem),
        Addressing::Absolute => fetch_word(reg, mem),
        Addressing::AbsoluteX => fetch_absolute_x(reg, mem),
        Addressing::AbsoluteY => fetch_absolute_y(reg, mem),
        Addressing::Relative => fetch_relative_address(reg, mem),
        Addressing::IndirectAbsolute => fetch_indirect_absolute(reg, mem),
        Addressing::IndexedIndirect => fetch_indexed_indirect(reg, mem),
        Addressing::IndirectIndexed => fetch_indirect_indexed(reg, mem),
    }
}

fn fetch_opcode<'a, T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> &'a Opcode {
    let value = mem.peek(reg.get_pc() as usize);
    let opcode: &Opcode = OPCODES.get(&value).unwrap();
    reg.incr_pc();
    opcode
}
fn fetch_word<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let hi = mem.peek(reg.get_pc() as usize) as u16;
    reg.incr_pc();
    let low = mem.peek(reg.get_pc() as usize) as u16;
    reg.incr_pc();
    ((low << 8) | hi) as u16
}
fn fetch<T: CpuRegisters, M: Memory>(reg: &mut T, mem: &mut M) -> u16 {
    let value = mem.peek(reg.get_pc() as usize);
    reg.incr_pc();
    value as u16
}

#[derive(PartialEq)]
pub enum EmulationStatus {
    PROCESSING,
    ERROR,
    BREAK,
}

pub struct Cpu {
    register: Registers,
    ram: Ram,
    rom: Cartbridge,
    status: EmulationStatus,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            register: Registers::new(),
            ram: Ram::new(),
            rom: Cartbridge::new(),
            status: EmulationStatus::PROCESSING,
        }
    }

    pub fn init_mem(&mut self) {
        self.ram.load_program(&mut self.rom);
        println!("{}", self.ram);
    }
    pub fn init_rom(&mut self, rom: String) {
        self.rom.load_from_file(rom);
    }

    fn set_random_number(&mut self) {
        let mut rng = rand::thread_rng();
        let value: u8 = rng.gen_range(0, 255);
        self.ram.write(0x00FE, value);
    }

    pub fn run(&mut self) -> EmulationStatus {
        let pc = self.register.get_pc();
        self.set_random_number();
        let opcode = fetch_opcode(&mut self.register, &mut self.ram);
        let value = fetch_operand(opcode, &mut self.register, &mut self.ram);
        println!("OPCODE at {:x?}: {:x?}, value: {:x?}", pc, opcode, value);
        self.execute_op(opcode, value)
    }
    fn run_instructions(&mut self, n: usize) {
        for _i in 0..n {
            self.set_random_number();
            let opcode = fetch_opcode(&mut self.register, &mut self.ram);
            let value = fetch_operand(opcode, &mut self.register, &mut self.ram);
            println!("OPCODE at {:x?}: {:x?}, value: {:x?}", self.register.get_pc(), opcode, value);
            self.execute_op(opcode, value);
        }
    }
    fn execute_op(&mut self, opcode: &Opcode, mut value: u16) -> EmulationStatus {
        match opcode.name {
            Instruction::ADC => {
                let res: (u8, bool);
                if self.register.get_flag(StatusFlags::CARRY) {
                    value += 1;
                }
                if opcode.mode == Addressing::Immediate {
                    res = self.register.get_a().overflowing_add(value as u8);
                } else {
                    res = self.register.get_a().overflowing_add(self.ram.peek(value as usize));
                }
                self.register
                    .set_a(res.0)
                    .set_flag(StatusFlags::CARRY, res.1)
                    .set_flag(StatusFlags::OVERFLOW, res.1)
                    .set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res.0 == 0);
            }
            Instruction::AND => {
                let a = self.register.get_a();
                if opcode.mode == Addressing::Immediate {
                    self.register.set_a(a & value as u8);
                } else {
                    self.register.set_a(a & self.ram.peek(value as usize) as u8);
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
                    old = self.ram.peek(value as usize);
                    let m = self.ram.write(value as usize, old << 1);
                    self.register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                self.register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
            }
            Instruction::BCC => {
                if !self.register.get_flag(StatusFlags::CARRY) {
                    self.register.set_pc(value);
                }
            }
            Instruction::BCS => {
                if self.register.get_flag(StatusFlags::CARRY) {
                    self.register.set_pc(value);
                }
            }
            Instruction::BEQ => {
                if self.register.get_flag(StatusFlags::ZERO) {
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
                    self.register.set_pc(value);
                }
            }
            Instruction::BNE => {
                if !self.register.get_flag(StatusFlags::ZERO) {
                    self.register.set_pc(value);
                }
            }
            Instruction::BPL => {
                if !self.register.get_flag(StatusFlags::NEGATIVE) {
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
                    self.register.set_pc(value);
                }
            }
            Instruction::BVS => {
                if self.register.get_flag(StatusFlags::OVERFLOW) {
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
                    m = self.ram.peek(value as usize) as u16;
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
                    m = self.ram.peek(value as usize) as u16;
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
                    m = self.ram.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, y < m as u8)
                    .set_flag(StatusFlags::CARRY, y >= m as u8)
                    .set_flag(StatusFlags::ZERO, y == m as u8);
            }
            Instruction::DEC => {
                let res = self.ram.peek(value as usize).wrapping_sub(1);
                self.ram.write(value as usize, res);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::DEX => {
                let res = self.register.get_x();
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_x(res.wrapping_sub(1));
            }
            Instruction::DEY => {
                let res = self.register.get_y();
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_y(res.wrapping_sub(1));
            }
            Instruction::EOR => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = self.ram.peek(value as usize) as u16;
                }
                let res = a ^ m as u8;
                self.register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INC => {
                let res = self.ram.peek(value as usize).wrapping_add(1);
                self.ram.write(value as usize, res);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INX => {
                let res = self.register.get_x();
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_x(res.wrapping_add(1));
            }
            Instruction::INY => {
                let res = self.register.get_y();
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_y(res.wrapping_add(1));
            }
            Instruction::JMP => {
                self.register.set_pc(value);
            }
            Instruction::JSR => {
                let pc = self.register.get_pc() - 1;
                self.register.push_stack((pc >> 8) as u8, &mut self.ram);
                self.register.push_stack(pc as u8, &mut self.ram);
                self.register.set_pc(value);
            }
            Instruction::LDA => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = self.ram.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_a(res as u8);
            }
            Instruction::LDX => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = self.ram.peek(value as usize) as u16;
                }
                self.register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_x(res as u8);
            }
            Instruction::LDY => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = self.ram.peek(value as usize) as u16;
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
                    old = self.ram.peek(value as usize);
                    let m = self.ram.write(value as usize, old >> 1);
                    self.register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                self.register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
            }
            Instruction::NOP => {
                return EmulationStatus::PROCESSING;
            }
            Instruction::ORA => {
                let a = self.register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = self.ram.peek(value as usize) as u16;
                }
                let res = a | m as u8;
                self.register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);

            }
            Instruction::PHA => {
                let a = self.register.get_a();
                self.register.push_stack(a, &mut self.ram);
            }
            Instruction::PHP => {
                let status = self.register.get_sp();
                self.register.push_stack(status, &mut self.ram);
            }
            Instruction::PLA => {
                let res = self.register.pop_stack(&mut self.ram);
                self.register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_a(res as u8);
            }
            Instruction::PLP => {
                let res = self.register.pop_stack(&mut self.ram);
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
                    let old = self.ram.peek(value as usize);
                    let mut m = old << 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        m += 1;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
                    self.ram.write(value as usize, m);
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
                    let old = self.ram.peek(value as usize);
                    let mut m = old >> 1;
                    if self.register.get_flag(StatusFlags::CARRY) {
                        m += 128;
                    }
                    self.register
                        .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0);
                    self.ram.write(value as usize, m);
                }
            }
            Instruction::RTI => {
                let sp = self.register.pop_stack(&mut self.ram);
                self.register.set_sp(sp as u8);
                let hi = self.register.pop_stack(&mut self.ram) as u16;
                let low = self.register.pop_stack(&mut self.ram) as u16;
                self.register.set_pc((hi << 8) | low);
            }
            Instruction::RTS => {
                let low = self.register.pop_stack(&mut self.ram) as u16;
                let hi = self.register.pop_stack(&mut self.ram) as u16;
                self.register.set_pc((hi << 8) | low);
                self.register.incr_pc();
            }
            Instruction::SBC => {
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = self.ram.peek(value as usize) as u16;
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
                self.ram.write(value as usize, self.register.get_a());
            }
            Instruction::STX => {
                self.ram.write(value as usize, self.register.get_x());
            }
            Instruction::STY => {
                self.ram.write(value as usize, self.register.get_y());
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

            }
            _ => { 
                println!("Unrecognized Opcode: {:x?}", opcode);
                return EmulationStatus::ERROR;
            }
        }
        return EmulationStatus::PROCESSING;
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "==========================")?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "Flags: {:08b}", self.register.get_sr())?;
        writeln!(f, "Accumulator: ${:x?}", self.register.get_a())?;
        writeln!(f, "X: ${:x?}", self.register.get_x())?;
        writeln!(f, "Y: ${:x?}", self.register.get_y())?;
        writeln!(f, "Stack: ${:x?}", self.register.get_sp())?;
        writeln!(f, "End PC: ${:x?}", self.register.get_pc())?;
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
    #[test]
    fn test_adc_immediate() {
        let program:Vec<u8> = vec!(0x69, 0xa1); // ADC #$a1
        let mut cpu = create_test_context(&program);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0xa1);
    }
    #[test]
    fn test_adc_zeropage() {
        let program:Vec<u8> = vec!(0x65, 0xa1); // ADC $a1
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a1, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_zeropagex() {
        let program:Vec<u8> = vec!(0x75, 0xa1); // ADC $a1,X
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a2, 0x08);
        cpu.register.set_x(0x01);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolute() {
        let program:Vec<u8> = vec!(0x6D, 0xa1, 0x00); // ADC $00a1
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a1, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolutex() {
        let program:Vec<u8> = vec!(0x7D, 0xa1, 0x00); // ADC $00a1
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a2, 0x08);
        cpu.register.set_x(0x01);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_asl_accumulator() {
        let program:Vec<u8> = vec!(0x0A); // ASL A
        let mut cpu = create_test_context(&program);
        cpu.register.set_a(0x02);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x04);
    }
    #[test]
    fn test_asl() {
        let program:Vec<u8> = vec!(0x06, 0x04); // ASL $04
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x0004, 0x02);
        cpu.run_instructions(1);
        assert_eq!(cpu.ram.peek(0x0004), 0x04);
    }
    #[test]
    fn test_lda_immediate() {
        let program:Vec<u8> = vec!(0xa9, 0xff); // LDA #$ff
        let mut cpu = create_test_context(&program);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0xFF);
    }
    #[test]
    fn test_lda_zeroepage() {
        let program:Vec<u8> = vec!(0xa5, 0xa0); // LDA $a0
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a0, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_zeroepagex() {
        let program:Vec<u8> = vec!(0xb5, 0xa0); // LDA $a0,X
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a5, 0x08);
        cpu.register.set_x(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_absolute() {
        let program:Vec<u8> = vec!(0xad, 0x00, 0xF0);  // LDA $f000
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0xf000, 0x0F);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutex() {
        let program:Vec<u8> = vec!(0xbd, 0xa5, 0x00);  // LDA $00a5,X
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00aa, 0x0F);
        cpu.register.set_x(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutey() {
        let program:Vec<u8> = vec!(0xb9, 0xa5, 0x00);  // LDA $00a5,Y
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00aa, 0x0F);
        cpu.register.set_y(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_indirectx() {
        let program:Vec<u8> = vec!(0xa1, 0xa0);  // LDA ($a0,X)
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a1, 0xA3);
        cpu.ram.write(0x00a3, 0xA0);
        cpu.register.set_x(0x01);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0xA0);
    }
    #[test]
    fn test_lda_indirecty() {
        let program:Vec<u8> = vec!(0xb1, 0xa5);  // LDA ($a5),Y
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x00a4, 0xA0);
        cpu.ram.write(0x00a5, 0xA3);
        cpu.register.set_y(0x01);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0xA0);
    }
    #[test]
    fn test_rol_accumulator() {
        let program: Vec<u8> = vec!(0xA9, 146, 0x2A, 0x2A);
        let mut cpu = create_test_context(&program);
        cpu.run_instructions(2);
        assert_eq!(cpu.register.get_a(), 36);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 73);
        assert!(!cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_rol() {
        let program: Vec<u8> = vec!(0x26, 0x22, 0x26, 0x22);
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x0022, 146);
        cpu.run_instructions(1);
        assert_eq!(cpu.ram.peek(0x0022), 36);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
        cpu.run_instructions(1);
        assert_eq!(cpu.ram.peek(0x0022), 73);
        assert!(!cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_ror_accumulator() {
        let program: Vec<u8> = vec!(0xA9, 147, 0x6A, 0x6A);
        let mut cpu = create_test_context(&program);
        cpu.run_instructions(2);
        assert_eq!(cpu.register.get_a(), 73);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 164);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_ror() {
        let program: Vec<u8> = vec!(0x66, 0x22, 0x66, 0x22);
        let mut cpu = create_test_context(&program);
        cpu.ram.write(0x0022, 147);
        cpu.run_instructions(1);
        assert_eq!(cpu.ram.peek(0x0022), 73);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
        cpu.run_instructions(1);
        assert_eq!(cpu.ram.peek(0x0022), 164);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_sta() {
        let program:Vec<u8> = vec!(0x8d, 0xa5, 0x00);  // STA $00a5
        let mut cpu = create_test_context(&program);
        cpu.register.set_a(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.ram.peek(0xa5u8 as usize), 0x05);
    }
}