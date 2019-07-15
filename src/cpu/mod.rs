pub mod cpu_register;
pub mod opcode;
pub mod cpu_mem;
pub mod cpu_bus;

use crate::cpu::cpu_bus::Bus;
use crate::cpu::cpu_register::*;
use crate::cpu::opcode::*;
extern crate rand;

use rand::Rng;

fn fetch_absolute_x<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let x = reg.get_x();
    let addr = fetch_word(reg, bus);
    addr.wrapping_add(x as u16)
}

fn fetch_absolute_y<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let y = reg.get_y();
    let addr = fetch_word(reg, bus);
    addr.wrapping_add(y as u16)
}

fn fetch_zeropage_x<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let value = fetch(reg, bus);
    let x = reg.get_x();
    value.wrapping_add(x as u16)
}
fn fetch_zeropage_y<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let value = fetch(reg, bus);
    let y = reg.get_y();
    value.wrapping_add(y as u16)
}

fn fetch_relative_address<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let a = fetch(reg, bus);
    if a < 0x80 {
        a + reg.get_pc() as u16
    } else {
        a + reg.get_pc() as u16 - 0x100
    }
}
fn fetch_indirect_absolute<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let addr = fetch_word(reg, bus);
    let upper = bus.peek((addr as usize) | ((addr) + 1) as usize) as u16;
    let low = bus.peek(addr as usize) as u16;
    low + (upper << 8) as u16
}
fn fetch_indexed_indirect<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let x = reg.get_x() as u16;
    let addr = fetch(reg, bus).wrapping_add(x);
    let hi = bus.peek(addr as usize) as u16;
    let low = bus.peek((addr + x) as usize) as u16;
    ((low << 8) | hi) as u16
}
fn fetch_indirect_indexed<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let addr = fetch(reg, bus);
    let base_addr = (bus.peek(addr as usize) as usize) + ((bus.peek(((addr + 1) & 0x00FF) as usize) as usize) * 0x100);
    ((base_addr + (reg.get_y() as usize)) & 0xFFFF) as u16
}
fn fetch_operand<T: CpuRegister, B: Bus>(code: &Opcode, reg: &mut T, bus: &mut B) -> u16 {
    match code.mode {
        Addressing::Accumulator => 0x0000,
        Addressing::Implied => 0x0000,
        Addressing::Immediate => fetch(reg, bus),
        Addressing::ZeroPage => fetch(reg, bus),
        Addressing::ZeroPageX => fetch_zeropage_x(reg, bus),
        Addressing::ZeroPageY => fetch_zeropage_y(reg, bus),
        Addressing::Absolute => fetch_word(reg, bus),
        Addressing::AbsoluteX => fetch_absolute_x(reg, bus),
        Addressing::AbsoluteY => fetch_absolute_y(reg, bus),
        Addressing::Relative => fetch_relative_address(reg, bus),
        Addressing::IndirectAbsolute => fetch_indirect_absolute(reg, bus),
        Addressing::IndexedIndirect => fetch_indexed_indirect(reg, bus),
        Addressing::IndirectIndexed => fetch_indirect_indexed(reg, bus),
    }
}

fn fetch_opcode<'a, T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> &'a Opcode {
    let value = bus.peek(reg.get_pc() as usize);
    reg.incr_pc();
    OPCODES.get(&value).unwrap()
}
fn fetch_word<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let hi = bus.peek(reg.get_pc() as usize) as u16;
    reg.incr_pc();
    let low = bus.peek(reg.get_pc() as usize) as u16;
    reg.incr_pc();
    ((low << 8) | hi) as u16
}
fn fetch<T: CpuRegister, B: Bus>(reg: &mut T, bus: &mut B) -> u16 {
    let value = bus.peek(reg.get_pc() as usize) as u16;
    reg.incr_pc();
    value
}

#[derive(PartialEq)]
pub enum EmulationStatus {
    PROCESSING,
    ERROR,
    BREAK,
    INFINITE_LOOP,
}

pub struct Cpu {
    pub opcode_counter: u32,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            opcode_counter: 0,
        }
    }
    pub fn reset<B: Bus, R: CpuRegister>(&mut self, bus: &mut B, register: &mut R) {
        let hi = bus.peek(0xFFFC) as u16;
        let low = bus.peek(0xFFFC + 1) as u16;
        let pc = ((low << 8) | hi) as u16;
        register.set_pc(pc);
    }
    fn set_random_number<B: Bus>(&mut self, bus: &mut B) {
        let mut rng = rand::thread_rng();
        let value: u8 = rng.gen_range(0, 255);
        bus.write(0x00FE, value);
    }

    pub fn run<B: Bus, R: CpuRegister>(&mut self, bus: &mut B, register: &mut R) -> (u16, EmulationStatus) {
        self.set_random_number(bus);
        let opcode = fetch_opcode(register, bus);
        let value = fetch_operand(opcode, register, bus);
        println!("OPCODE at {:x?}: {:x?}, value: {:x?}", register.get_pc(), opcode, value);
        (opcode.cycle, self.execute_op(opcode, value, bus, register))
    }
    fn run_instructions<B: Bus, R: CpuRegister>(&mut self, n: usize, bus: &mut B, register: &mut R) {
        for _i in 0..n {
            self.set_random_number(bus);
            let opcode = fetch_opcode(register, bus);
            let value = fetch_operand(opcode, register, bus);
            println!("OPCODE at {:x?}: {:x?}, value: {:x?}", register.get_pc(), opcode, value);
            self.execute_op(opcode, value, bus, register);
        }
    }
    fn execute_op<B: Bus, R: CpuRegister>(&mut self, opcode: &Opcode, value: u16, bus: &mut B, register: &mut R) -> EmulationStatus {
        self.opcode_counter += 1;
        match opcode.name {
            Instruction::ADC => {
                let mut res: (u8, bool);
                if opcode.mode == Addressing::Immediate {
                    res = register.get_a().overflowing_add(value as u8);
                } else {
                    res = register.get_a().overflowing_add(bus.peek(value as usize));
                }
                if register.get_flag(StatusFlags::CARRY) {
                    res.0 += 1;
                }
                register
                    .set_a(res.0)
                    .set_flag(StatusFlags::CARRY, res.1)
                    .set_flag(StatusFlags::OVERFLOW, res.1)
                    .set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res.0 == 0);
            }
            Instruction::AND => {
                let a = register.get_a();
                if opcode.mode == Addressing::Immediate {
                    register.set_a(a & value as u8);
                } else {
                    register.set_a(a & bus.peek(value as usize) as u8);
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, value == 0);
            }
            Instruction::ASL => {
                let old: u8;
                if opcode.mode == Addressing::Accumulator {
                    old = register.get_a();
                    let a = register.set_a(old << 1).get_a();
                    register
                        .set_flag(StatusFlags::ZERO, a == 0)
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0);
                } else {
                    old = bus.peek(value as usize);
                    let m = bus.write(value as usize, old << 1);
                    register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
            }
            Instruction::BCC => {
                if !register.get_flag(StatusFlags::CARRY) {
                    register.set_pc(value);
                }
            }
            Instruction::BCS => {
                if register.get_flag(StatusFlags::CARRY) {
                    register.set_pc(value);
                }
            }
            Instruction::BEQ => {
                if register.get_flag(StatusFlags::ZERO) {
                    register.set_pc(value);
                }
            }
            Instruction::BIT => {
                let value = register.get_a() & value as u8;
                register
                    .set_flag(StatusFlags::ZERO, value == 0)
                    .set_flag(StatusFlags::OVERFLOW, value & (1 << 6) != 0)
                    .set_flag(StatusFlags::NEGATIVE, value & (1 << 7) != 0);
            }
            Instruction::BMI => {
                if register.get_flag(StatusFlags::NEGATIVE) {
                    register.set_pc(value);
                }
            }
            Instruction::BNE => {
                if !register.get_flag(StatusFlags::ZERO) {
                    register.set_pc(value);
                }
            }
            Instruction::BPL => {
                if !register.get_flag(StatusFlags::NEGATIVE) {
                    register.set_pc(value);
                }
            }
            Instruction::BRK => {
                register
                    .set_flag(StatusFlags::BREAK, true)
                    .set_flag(StatusFlags::INTERRUPT, true);
                return EmulationStatus::BREAK;
            }
            Instruction::BVC => {
                if !register.get_flag(StatusFlags::OVERFLOW) {
                    register.set_pc(value);
                }
            }
            Instruction::BVS => {
                if register.get_flag(StatusFlags::OVERFLOW) {
                    register.set_pc(value);
                }
            }
            Instruction::CLC => {
                register.set_flag(StatusFlags::CARRY, false);
            }
            Instruction::CLD => {
                register.set_flag(StatusFlags::DECIMAL, false);
            }
            Instruction::CLI => {
                register.set_flag(StatusFlags::INTERRUPT, false);
            }
            Instruction::CLV => {
                register.set_flag(StatusFlags::OVERFLOW, false);
            }
            Instruction::CMP => {
                let a = register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, a < m as u8)
                    .set_flag(StatusFlags::CARRY, a >= m as u8)
                    .set_flag(StatusFlags::ZERO, a == m as u8);
            }
            Instruction::CPX => {
                let x = register.get_x();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, x < m as u8)
                    .set_flag(StatusFlags::CARRY, x >= m as u8)
                    .set_flag(StatusFlags::ZERO, x == m as u8);
            }
            Instruction::CPY => {
                let y = register.get_y();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, y < m as u8)
                    .set_flag(StatusFlags::CARRY, y >= m as u8)
                    .set_flag(StatusFlags::ZERO, y == m as u8);
            }
            Instruction::DEC => {
                let res = bus.peek(value as usize).wrapping_sub(1);
                bus.write(value as usize, res);
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::DEX => {
                let res = register.get_x();
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_x(res.wrapping_sub(1));
            }
            Instruction::DEY => {
                let res = register.get_y();
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_y(res.wrapping_sub(1));
            }
            Instruction::EOR => {
                let a = register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                let res = a ^ m as u8;
                register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INC => {
                let res = bus.peek(value as usize).wrapping_add(1);
                bus.write(value as usize, res);
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);
            }
            Instruction::INX => {
                let res = register.get_x();
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_x(res.wrapping_add(1));
            }
            Instruction::INY => {
                let res = register.get_y();
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_y(res.wrapping_add(1));
            }
            Instruction::JMP => {
                register.set_pc(value);
            }
            Instruction::JSR => {
                let pc = register.get_pc() - 1;
                register.push_stack((pc >> 8) as u8, bus);
                register.push_stack(pc as u8, bus);
                register.set_pc(value);
            }
            Instruction::LDA => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value as usize) as u16;
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_a(res as u8);
            }
            Instruction::LDX => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value as usize) as u16;
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_x(res as u8);
            }
            Instruction::LDY => {
                let mut res = value;
                if opcode.mode != Addressing::Immediate {
                    res = bus.peek(value as usize) as u16;
                }
                register
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_y(res as u8);
            }
            Instruction::LSR => {
                let old: u8;
                if opcode.mode == Addressing::Accumulator {
                    old = register.get_a();
                    let a = register.set_a(old >> 1).get_a();
                    register
                        .set_flag(StatusFlags::ZERO, a == 0)
                        .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0);
                } else {
                    old = bus.peek(value as usize);
                    let m = bus.write(value as usize, old >> 1);
                    register
                        .set_flag(StatusFlags::NEGATIVE, m & (1 << 7) != 0);
                }
                register.set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
            }
            Instruction::NOP => {
                return EmulationStatus::PROCESSING;
            }
            Instruction::ORA => {
                let a = register.get_a();
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                let res = a | m as u8;
                register
                    .set_a(res)
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0);

            }
            Instruction::PHA => {
                let a = register.get_a();
                register.push_stack(a, bus);
            }
            Instruction::PHP => {
                let status = register.get_sp();
                register.push_stack(status, bus);
            }
            Instruction::PLA => {
                let res = register.pop_stack(bus);
                register
                    .set_flag(StatusFlags::ZERO, res == 0)
                    .set_flag(StatusFlags::NEGATIVE, res & (1 << 7) != 0)
                    .set_a(res as u8);
            }
            Instruction::PLP => {
                let res = register.pop_stack(bus);
                register.set_sp(res as u8);
            }
            Instruction::ROL => {
                if opcode.mode == Addressing::Accumulator {
                    let old = register.get_a();
                    let mut a = old << 1;
                    if register.get_flag(StatusFlags::CARRY) {
                        a += 1;
                    }
                    register
                        .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0)
                        .set_a(a);
                } else {
                    let old = bus.peek(value as usize);
                    let mut m = old << 1;
                    if register.get_flag(StatusFlags::CARRY) {
                        m += 1;
                    }
                    register
                        .set_flag(StatusFlags::CARRY, old & (1 << 7) != 0);
                    bus.write(value as usize, m);
                }
            }
            Instruction::ROR => {
                if opcode.mode == Addressing::Accumulator {
                    let old = register.get_a();
                    let mut a = old >> 1;
                    if register.get_flag(StatusFlags::CARRY) {
                        a += 128;
                    }
                    register
                        .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0)
                        .set_a(a);
                } else {
                    let old = bus.peek(value as usize);
                    let mut m = old >> 1;
                    if register.get_flag(StatusFlags::CARRY) {
                        m += 128;
                    }
                    register
                        .set_flag(StatusFlags::CARRY, old & (1 << 0) != 0);
                    bus.write(value as usize, m);
                }
            }
            Instruction::RTI => {
                let sp = register.pop_stack(bus);
                register.set_sp(sp as u8);
                let hi = register.pop_stack(bus) as u16;
                let low = register.pop_stack(bus) as u16;
                register.set_pc((hi << 8) | low);
            }
            Instruction::RTS => {
                let low = register.pop_stack(bus) as u16;
                let hi = register.pop_stack(bus) as u16;
                register.set_pc((hi << 8) | low);
                register.incr_pc();
            }
            Instruction::SBC => {
                let mut m = value;
                if opcode.mode != Addressing::Immediate {
                    m = bus.peek(value as usize) as u16;
                }
                if register.get_flag(StatusFlags::CARRY) {
                    m += 1;
                }
                let res: (u8, bool) = register.get_a().overflowing_sub(m as u8);
                register
                    .set_flag(StatusFlags::CARRY, res.1)
                    .set_flag(StatusFlags::OVERFLOW, res.1)
                    .set_flag(StatusFlags::NEGATIVE, res.0 & (1 << 7) != 0)
                    .set_flag(StatusFlags::ZERO, res.0 == 0)
                    .set_a(res.0);
            }
            Instruction::SEC => {
                register.set_flag(StatusFlags::CARRY, true);
            }
            Instruction::SED => {
                register.set_flag(StatusFlags::DECIMAL, true);
            }
            Instruction::SEI => {
                register.set_flag(StatusFlags::INTERRUPT, true);
            }
            Instruction::STA => {
                bus.write(value as usize, register.get_a());
            }
            Instruction::STX => {
                bus.write(value as usize, register.get_x());
            }
            Instruction::STY => {
                bus.write(value as usize, register.get_y());
            }
            Instruction::TAX => {
                let a = register.get_a();
                register
                    .set_flag(StatusFlags::ZERO, a == 0)
                    .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0)
                    .set_x(a);
            }
            Instruction::TAY => {
                let a = register.get_a();
                register
                    .set_flag(StatusFlags::ZERO, a == 0)
                    .set_flag(StatusFlags::NEGATIVE, a & (1 << 7) != 0)
                    .set_y(a);
            }
            Instruction::TSX => {
                let st = register.get_sp();
                register
                    .set_flag(StatusFlags::ZERO, st == 0)
                    .set_flag(StatusFlags::NEGATIVE, st & (1 << 7) != 0)
                    .set_x(st as u8);
            }
            Instruction::TXA => {
                let x = register.get_x();
                register
                    .set_flag(StatusFlags::ZERO, x == 0)
                    .set_flag(StatusFlags::NEGATIVE, x & (1 << 7) != 0)
                    .set_a(x);
            }
            Instruction::TXS => {
                let sp = register.get_sp();
                register
                    .set_flag(StatusFlags::ZERO, sp == 0)
                    .set_flag(StatusFlags::NEGATIVE, sp & (1 << 7) != 0)
                    .set_x(sp);
            }
            Instruction::TYA => {
                let y = register.get_y();
                register
                    .set_flag(StatusFlags::ZERO, y == 0)
                    .set_flag(StatusFlags::NEGATIVE, y & (1 << 7) != 0)
                    .set_a(y);
            }
        }
        return EmulationStatus::PROCESSING;
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    fn create_test_context(program: &Vec<u8>) -> Cpu {
        let mut cpu = Cpu::new();
        let mut cpu_ram = memory::Ram::new();
        let cpu_register = cpu::cpu_register::Register::new();
        let ppu = ppu::Ppu::new();
        let mut cartbridge = Cartbridge::new();
        let mut buffer = cartbridge.read_file(String::from("roms/hello.nes"));
        cartbridge.load_program(&mut buffer);
        cpu.rom.load_from_vec(program);
        cpu.init_mem();
        let mut cpu_bus = cpu::cpu_bus::CpuBus::new(&mut cpu_ram, &mut cpu_rom, &mut ppu);
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
        cpu.memory.write(0x00a1, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_zeropagex() {
        let program:Vec<u8> = vec!(0x75, 0xa1); // ADC $a1,X
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00a2, 0x08);
        cpu.register.set_x(0x01);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolute() {
        let program:Vec<u8> = vec!(0x6D, 0xa1, 0x00); // ADC $00a1
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00a1, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_adc_absolutex() {
        let program:Vec<u8> = vec!(0x7D, 0xa1, 0x00); // ADC $00a1
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00a2, 0x08);
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
        cpu.memory.write(0x0004, 0x02);
        cpu.run_instructions(1);
        assert_eq!(cpu.memory.peek(0x0004), 0x04);
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
        cpu.memory.write(0x00a0, 0x08);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_zeroepagex() {
        let program:Vec<u8> = vec!(0xb5, 0xa0); // LDA $a0,X
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00a5, 0x08);
        cpu.register.set_x(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x08);
    }
    #[test]
    fn test_lda_absolute() {
        let program:Vec<u8> = vec!(0xad, 0x00, 0x1F);  // LDA $f000
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x1f00, 0x0F);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutex() {
        let program:Vec<u8> = vec!(0xbd, 0xa5, 0x00);  // LDA $00a5,X
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00aa, 0x0F);
        cpu.register.set_x(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_absolutey() {
        let program:Vec<u8> = vec!(0xb9, 0xa5, 0x00);  // LDA $00a5,Y
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00aa, 0x0F);
        cpu.register.set_y(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0x0F);
    }
    #[test]
    fn test_lda_indirectx() {
        let program:Vec<u8> = vec!(0xa1, 0xa0);  // LDA ($a0,X)
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00a1, 0xA3);
        cpu.memory.write(0x00a3, 0xA0);
        cpu.register.set_x(0x01);
        cpu.run_instructions(1);
        assert_eq!(cpu.register.get_a(), 0xA0);
    }
    #[test]
    fn test_lda_indirecty() {
        let program:Vec<u8> = vec!(0xb1, 0xa5);  // LDA ($a5),Y
        let mut cpu = create_test_context(&program);
        cpu.memory.write(0x00a4, 0xA0);
        cpu.memory.write(0x00a5, 0xA3);
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
        cpu.memory.write(0x0022, 146);
        cpu.run_instructions(1);
        assert_eq!(cpu.memory.peek(0x0022), 36);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
        cpu.run_instructions(1);
        assert_eq!(cpu.memory.peek(0x0022), 73);
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
        cpu.memory.write(0x0022, 147);
        cpu.run_instructions(1);
        assert_eq!(cpu.memory.peek(0x0022), 73);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
        cpu.run_instructions(1);
        assert_eq!(cpu.memory.peek(0x0022), 164);
        assert!(cpu.register.get_flag(StatusFlags::CARRY));
    }
    #[test]
    fn test_sta() {
        let program:Vec<u8> = vec!(0x8d, 0xa5, 0x00);  // STA $00a5
        let mut cpu = create_test_context(&program);
        cpu.register.set_a(0x05);
        cpu.run_instructions(1);
        assert_eq!(cpu.memory.peek(0xa5u8 as usize), 0x05);
    }
}
*/