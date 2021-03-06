use std::collections::HashMap;

#[derive(Debug)]
pub struct Opcode {
    pub name: Instruction,
    pub mode: Addressing,
    pub cycle: u16,
}

lazy_static! {
    pub static ref OPCODES: HashMap<u8, Opcode> = {
        let mut map: HashMap<u8, Opcode> = HashMap::new();
        map.insert(0x69, Opcode{ name: Instruction::ADC, mode: Addressing::Immediate, cycle: 2});
        map.insert(0x65, Opcode{ name: Instruction::ADC, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x75, Opcode{ name: Instruction::ADC, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x6D, Opcode{ name: Instruction::ADC, mode: Addressing::Absolute, cycle: 4});
        map.insert(0x7D, Opcode{ name: Instruction::ADC, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x79, Opcode{ name: Instruction::ADC, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0x61, Opcode{ name: Instruction::ADC, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0x71, Opcode{ name: Instruction::ADC, mode: Addressing::IndirectIndexed, cycle: 5});

        map.insert(0x29, Opcode{ name: Instruction::AND, mode: Addressing::Immediate, cycle: 2});
        map.insert(0x25, Opcode{ name: Instruction::AND, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x35, Opcode{ name: Instruction::AND, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x2D, Opcode{ name: Instruction::AND, mode: Addressing::Absolute, cycle: 4});
        map.insert(0x3D, Opcode{ name: Instruction::AND, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x39, Opcode{ name: Instruction::AND, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0x21, Opcode{ name: Instruction::AND, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0x31, Opcode{ name: Instruction::AND, mode: Addressing::IndirectIndexed, cycle: 5});

        map.insert(0x0A, Opcode{ name: Instruction::ASL, mode: Addressing::Accumulator, cycle: 2});
        map.insert(0x06, Opcode{ name: Instruction::ASL, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x16, Opcode{ name: Instruction::ASL, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x0E, Opcode{ name: Instruction::ASL, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x1E, Opcode{ name: Instruction::ASL, mode: Addressing::AbsoluteX, cycle: 7});

        map.insert(0x90, Opcode{ name: Instruction::BCC, mode: Addressing::Relative, cycle: 2});
        map.insert(0xB0, Opcode{ name: Instruction::BCS, mode: Addressing::Relative, cycle: 2});
        map.insert(0xF0, Opcode{ name: Instruction::BEQ, mode: Addressing::Relative, cycle: 2});

        map.insert(0x24, Opcode{ name: Instruction::BIT, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x2C, Opcode{ name: Instruction::BIT, mode: Addressing::Absolute, cycle: 4});

        map.insert(0x30, Opcode{ name: Instruction::BMI, mode: Addressing::Relative, cycle: 2});

        map.insert(0xD0, Opcode{ name: Instruction::BNE, mode: Addressing::Relative, cycle: 2});

        map.insert(0x10, Opcode{ name: Instruction::BPL, mode: Addressing::Relative, cycle: 2});

        map.insert(0x00, Opcode{ name: Instruction::BRK, mode: Addressing::Implied, cycle: 7});

        map.insert(0x50, Opcode{ name: Instruction::BVC, mode: Addressing::Relative, cycle: 2});
        map.insert(0x70, Opcode{ name: Instruction::BVS, mode: Addressing::Relative, cycle: 2});

        map.insert(0x18, Opcode{ name: Instruction::CLC, mode: Addressing::Implied, cycle: 2});
        map.insert(0xD8, Opcode{ name: Instruction::CLD, mode: Addressing::Implied, cycle: 2});
        map.insert(0x58, Opcode{ name: Instruction::CLI, mode: Addressing::Implied, cycle: 2});
        map.insert(0xB8, Opcode{ name: Instruction::CLV, mode: Addressing::Implied, cycle: 2});

        map.insert(0xC9, Opcode{ name: Instruction::CMP, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xC5, Opcode{ name: Instruction::CMP, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xD5, Opcode{ name: Instruction::CMP, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0xCD, Opcode{ name: Instruction::CMP, mode: Addressing::Absolute, cycle: 4});
        map.insert(0xDD, Opcode{ name: Instruction::CMP, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0xD9, Opcode{ name: Instruction::CMP, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0xC1, Opcode{ name: Instruction::CMP, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0xD1, Opcode{ name: Instruction::CMP, mode: Addressing::IndirectIndexed, cycle: 5});

        map.insert(0xE0, Opcode{ name: Instruction::CPX, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xE4, Opcode{ name: Instruction::CPX, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xEC, Opcode{ name: Instruction::CPX, mode: Addressing::Absolute, cycle: 4});

        map.insert(0xC0, Opcode{ name: Instruction::CPY, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xC4, Opcode{ name: Instruction::CPY, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xCC, Opcode{ name: Instruction::CPY, mode: Addressing::Absolute, cycle: 4});

        map.insert(0xC6, Opcode{ name: Instruction::DEC, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0xD6, Opcode{ name: Instruction::DEC, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0xCE, Opcode{ name: Instruction::DEC, mode: Addressing::Absolute, cycle: 6});
        map.insert(0xDE, Opcode{ name: Instruction::DEC, mode: Addressing::AbsoluteX, cycle: 7});

        map.insert(0xCA, Opcode{ name: Instruction::DEX, mode: Addressing::Implied, cycle: 2});
        map.insert(0x88, Opcode{ name: Instruction::DEY, mode: Addressing::Implied, cycle: 2});

        map.insert(0xC7, Opcode{ name: Instruction::DCP, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0xD7, Opcode{ name: Instruction::DCP, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0xCF, Opcode{ name: Instruction::DCP, mode: Addressing::Absolute, cycle: 6});
        map.insert(0xDF, Opcode{ name: Instruction::DCP, mode: Addressing::AbsoluteX, cycle: 6});
        map.insert(0xDB, Opcode{ name: Instruction::DCP, mode: Addressing::AbsoluteY, cycle: 6});
        map.insert(0xC3, Opcode{ name: Instruction::DCP, mode: Addressing::IndexedIndirect, cycle: 8});
        map.insert(0xD3, Opcode{ name: Instruction::DCP, mode: Addressing::IndirectIndexed, cycle: 7});

        map.insert(0x49, Opcode{ name: Instruction::EOR, mode: Addressing::Immediate, cycle: 2});
        map.insert(0x45, Opcode{ name: Instruction::EOR, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x55, Opcode{ name: Instruction::EOR, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x4D, Opcode{ name: Instruction::EOR, mode: Addressing::Absolute, cycle: 4});
        map.insert(0x5D, Opcode{ name: Instruction::EOR, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x59, Opcode{ name: Instruction::EOR, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0x41, Opcode{ name: Instruction::EOR, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0x51, Opcode{ name: Instruction::EOR, mode: Addressing::IndirectIndexed, cycle: 5});

        map.insert(0xE6, Opcode{ name: Instruction::INC, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0xF6, Opcode{ name: Instruction::INC, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0xEE, Opcode{ name: Instruction::INC, mode: Addressing::Absolute, cycle: 6});
        map.insert(0xFE, Opcode{ name: Instruction::INC, mode: Addressing::AbsoluteX, cycle: 7});

        map.insert(0xE8, Opcode{ name: Instruction::INX, mode: Addressing::Implied, cycle: 2});
        map.insert(0xC8, Opcode{ name: Instruction::INY, mode: Addressing::Implied, cycle: 2});

        map.insert(0xE7, Opcode{ name: Instruction::ISB, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0xF7, Opcode{ name: Instruction::ISB, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0xE3, Opcode{ name: Instruction::ISB, mode: Addressing::IndexedIndirect, cycle: 8});
        map.insert(0xF3, Opcode{ name: Instruction::ISB, mode: Addressing::IndirectIndexed, cycle: 8});
        map.insert(0xEF, Opcode{ name: Instruction::ISB, mode: Addressing::Absolute, cycle: 6});
        map.insert(0xFF, Opcode{ name: Instruction::ISB, mode: Addressing::AbsoluteX, cycle: 7});
        map.insert(0xFB, Opcode{ name: Instruction::ISB, mode: Addressing::AbsoluteY, cycle: 7});

        map.insert(0x4C, Opcode{ name: Instruction::JMP, mode: Addressing::Absolute, cycle: 3});
        map.insert(0x6C, Opcode{ name: Instruction::JMP, mode: Addressing::IndirectAbsolute, cycle: 5});

        map.insert(0x20, Opcode{ name: Instruction::JSR, mode: Addressing::Absolute, cycle: 6});

        map.insert(0xA7, Opcode{ name: Instruction::LAX, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xB7, Opcode{ name: Instruction::LAX, mode: Addressing::ZeroPageY, cycle: 4});
        map.insert(0xA3, Opcode{ name: Instruction::LAX, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0xB3, Opcode{ name: Instruction::LAX, mode: Addressing::IndirectIndexed, cycle: 5});
        map.insert(0xAF, Opcode{ name: Instruction::LAX, mode: Addressing::Absolute, cycle: 4});
        map.insert(0xBF, Opcode{ name: Instruction::LAX, mode: Addressing::AbsoluteY, cycle: 4});

        map.insert(0xA9, Opcode{ name: Instruction::LDA, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xA5, Opcode{ name: Instruction::LDA, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xB5, Opcode{ name: Instruction::LDA, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0xAD, Opcode{ name: Instruction::LDA, mode: Addressing::Absolute, cycle: 4});
        map.insert(0xBD, Opcode{ name: Instruction::LDA, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0xB9, Opcode{ name: Instruction::LDA, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0xA1, Opcode{ name: Instruction::LDA, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0xB1, Opcode{ name: Instruction::LDA, mode: Addressing::IndirectIndexed, cycle: 5});

        map.insert(0xA2, Opcode{ name: Instruction::LDX, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xA6, Opcode{ name: Instruction::LDX, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xB6, Opcode{ name: Instruction::LDX, mode: Addressing::ZeroPageY, cycle: 4});
        map.insert(0xAE, Opcode{ name: Instruction::LDX, mode: Addressing::Absolute, cycle: 4});
        map.insert(0xBE, Opcode{ name: Instruction::LDX, mode: Addressing::AbsoluteY, cycle: 4});

        map.insert(0xA0, Opcode{ name: Instruction::LDY, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xA4, Opcode{ name: Instruction::LDY, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xB4, Opcode{ name: Instruction::LDY, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0xAC, Opcode{ name: Instruction::LDY, mode: Addressing::Absolute, cycle: 4});
        map.insert(0xBC, Opcode{ name: Instruction::LDY, mode: Addressing::AbsoluteX, cycle: 4});

        map.insert(0x4A, Opcode{ name: Instruction::LSR, mode: Addressing::Accumulator, cycle: 2});
        map.insert(0x46, Opcode{ name: Instruction::LSR, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x56, Opcode{ name: Instruction::LSR, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x4E, Opcode{ name: Instruction::LSR, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x5E, Opcode{ name: Instruction::LSR, mode: Addressing::AbsoluteX, cycle: 7});

        map.insert(0xEA, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        map.insert(0x1A, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        map.insert(0x3A, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        map.insert(0x5A, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        map.insert(0x7A, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        map.insert(0xDA, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        map.insert(0xFA, Opcode{ name: Instruction::NOP, mode: Addressing::Implied, cycle: 2});
        
        map.insert(0x04, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x44, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x64, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPage, cycle: 3});
        
        map.insert(0x0C, Opcode{ name: Instruction::NOP, mode: Addressing::Absolute, cycle: 4});
        
        map.insert(0x14, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x34, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x54, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x74, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0xD4, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0xF4, Opcode{ name: Instruction::NOP, mode: Addressing::ZeroPageX, cycle: 4});
        
        map.insert(0x1C, Opcode{ name: Instruction::NOP, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x3C, Opcode{ name: Instruction::NOP, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x5C, Opcode{ name: Instruction::NOP, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x7C, Opcode{ name: Instruction::NOP, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0xDC, Opcode{ name: Instruction::NOP, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0xFC, Opcode{ name: Instruction::NOP, mode: Addressing::AbsoluteX, cycle: 4});
        
        map.insert(0x80, Opcode{ name: Instruction::NOP, mode: Addressing::Immediate, cycle: 2});
        map.insert(0x82, Opcode{ name: Instruction::NOP, mode: Addressing::Immediate, cycle: 2});
        map.insert(0x89, Opcode{ name: Instruction::NOP, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xC2, Opcode{ name: Instruction::NOP, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xE2, Opcode{ name: Instruction::NOP, mode: Addressing::Immediate, cycle: 2});

        map.insert(0x09, Opcode{ name: Instruction::ORA, mode: Addressing::Immediate, cycle: 2});
        map.insert(0x05, Opcode{ name: Instruction::ORA, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x15, Opcode{ name: Instruction::ORA, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x0D, Opcode{ name: Instruction::ORA, mode: Addressing::Absolute, cycle: 4});
        map.insert(0x1D, Opcode{ name: Instruction::ORA, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0x19, Opcode{ name: Instruction::ORA, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0x01, Opcode{ name: Instruction::ORA, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0x11, Opcode{ name: Instruction::ORA, mode: Addressing::IndirectIndexed, cycle: 5});

        map.insert(0x48, Opcode{ name: Instruction::PHA, mode: Addressing::Implied, cycle: 3});
        map.insert(0x08, Opcode{ name: Instruction::PHP, mode: Addressing::Implied, cycle: 3});
        map.insert(0x68, Opcode{ name: Instruction::PLA, mode: Addressing::Implied, cycle: 4});
        map.insert(0x28, Opcode{ name: Instruction::PLP, mode: Addressing::Implied, cycle: 4});

        map.insert(0x27, Opcode{ name: Instruction::RLA, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x37, Opcode{ name: Instruction::RLA, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x23, Opcode{ name: Instruction::RLA, mode: Addressing::IndexedIndirect, cycle: 8});
        map.insert(0x33, Opcode{ name: Instruction::RLA, mode: Addressing::IndirectIndexed, cycle: 8});
        map.insert(0x2F, Opcode{ name: Instruction::RLA, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x3F, Opcode{ name: Instruction::RLA, mode: Addressing::AbsoluteX, cycle: 7});
        map.insert(0x3B, Opcode{ name: Instruction::RLA, mode: Addressing::AbsoluteY, cycle: 7});

        map.insert(0x2A, Opcode{ name: Instruction::ROL, mode: Addressing::Accumulator, cycle: 2});
        map.insert(0x26, Opcode{ name: Instruction::ROL, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x36, Opcode{ name: Instruction::ROL, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x2E, Opcode{ name: Instruction::ROL, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x3E, Opcode{ name: Instruction::ROL, mode: Addressing::AbsoluteX, cycle: 7});

        map.insert(0x6A, Opcode{ name: Instruction::ROR, mode: Addressing::Accumulator, cycle: 2});
        map.insert(0x66, Opcode{ name: Instruction::ROR, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x76, Opcode{ name: Instruction::ROR, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x6E, Opcode{ name: Instruction::ROR, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x7E, Opcode{ name: Instruction::ROR, mode: Addressing::AbsoluteX, cycle: 7});

        map.insert(0x67, Opcode{ name: Instruction::RRA, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x77, Opcode{ name: Instruction::RRA, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x63, Opcode{ name: Instruction::RRA, mode: Addressing::IndexedIndirect, cycle: 8});
        map.insert(0x73, Opcode{ name: Instruction::RRA, mode: Addressing::IndirectIndexed, cycle: 8});
        map.insert(0x6F, Opcode{ name: Instruction::RRA, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x7F, Opcode{ name: Instruction::RRA, mode: Addressing::AbsoluteX, cycle: 7});
        map.insert(0x7B, Opcode{ name: Instruction::RRA, mode: Addressing::AbsoluteY, cycle: 7});

        map.insert(0x40, Opcode{ name: Instruction::RTI, mode: Addressing::Implied, cycle: 6});
        map.insert(0x60, Opcode{ name: Instruction::RTS, mode: Addressing::Implied, cycle: 6});

        map.insert(0x87, Opcode{ name: Instruction::SAX, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x97, Opcode{ name: Instruction::SAX, mode: Addressing::ZeroPageY, cycle: 4});
        map.insert(0x83, Opcode{ name: Instruction::SAX, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0x8F, Opcode{ name: Instruction::SAX, mode: Addressing::Absolute, cycle: 4});
        
        map.insert(0xE9, Opcode{ name: Instruction::SBC, mode: Addressing::Immediate, cycle: 2});
        map.insert(0xE5, Opcode{ name: Instruction::SBC, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0xF5, Opcode{ name: Instruction::SBC, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0xED, Opcode{ name: Instruction::SBC, mode: Addressing::Absolute, cycle: 4});
        map.insert(0xFD, Opcode{ name: Instruction::SBC, mode: Addressing::AbsoluteX, cycle: 4});
        map.insert(0xF9, Opcode{ name: Instruction::SBC, mode: Addressing::AbsoluteY, cycle: 4});
        map.insert(0xE1, Opcode{ name: Instruction::SBC, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0xF1, Opcode{ name: Instruction::SBC, mode: Addressing::IndirectIndexed, cycle: 5});
        map.insert(0xEB, Opcode{ name: Instruction::SBC, mode: Addressing::Immediate, cycle: 2});

        map.insert(0x38, Opcode{ name: Instruction::SEC, mode: Addressing::Implied, cycle: 2});
        map.insert(0xF8, Opcode{ name: Instruction::SED, mode: Addressing::Implied, cycle: 2});
        map.insert(0x78, Opcode{ name: Instruction::SEI, mode: Addressing::Implied, cycle: 2});

        map.insert(0x07, Opcode{ name: Instruction::SLO, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x17, Opcode{ name: Instruction::SLO, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x0F, Opcode{ name: Instruction::SLO, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x1F, Opcode{ name: Instruction::SLO, mode: Addressing::AbsoluteX, cycle: 7});
        map.insert(0x1B, Opcode{ name: Instruction::SLO, mode: Addressing::AbsoluteY, cycle: 7});
        map.insert(0x03, Opcode{ name: Instruction::SLO, mode: Addressing::IndexedIndirect, cycle: 8});
        map.insert(0x13, Opcode{ name: Instruction::SLO, mode: Addressing::IndirectIndexed, cycle: 8});

        map.insert(0x47, Opcode{ name: Instruction::SRE, mode: Addressing::ZeroPage, cycle: 5});
        map.insert(0x57, Opcode{ name: Instruction::SRE, mode: Addressing::ZeroPageX, cycle: 6});
        map.insert(0x4F, Opcode{ name: Instruction::SRE, mode: Addressing::Absolute, cycle: 6});
        map.insert(0x5F, Opcode{ name: Instruction::SRE, mode: Addressing::AbsoluteX, cycle: 7});
        map.insert(0x5B, Opcode{ name: Instruction::SRE, mode: Addressing::AbsoluteY, cycle: 7});
        map.insert(0x43, Opcode{ name: Instruction::SRE, mode: Addressing::IndexedIndirect, cycle: 8});
        map.insert(0x53, Opcode{ name: Instruction::SRE, mode: Addressing::IndirectIndexed, cycle: 8});

        map.insert(0x85, Opcode{ name: Instruction::STA, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x95, Opcode{ name: Instruction::STA, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x8D, Opcode{ name: Instruction::STA, mode: Addressing::Absolute, cycle: 4});
        map.insert(0x9D, Opcode{ name: Instruction::STA, mode: Addressing::AbsoluteX, cycle: 5});
        map.insert(0x99, Opcode{ name: Instruction::STA, mode: Addressing::AbsoluteY, cycle: 5});
        map.insert(0x81, Opcode{ name: Instruction::STA, mode: Addressing::IndexedIndirect, cycle: 6});
        map.insert(0x91, Opcode{ name: Instruction::STA, mode: Addressing::IndirectIndexed, cycle: 6});

        map.insert(0x86, Opcode{ name: Instruction::STX, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x96, Opcode{ name: Instruction::STX, mode: Addressing::ZeroPageY, cycle: 4});
        map.insert(0x8E, Opcode{ name: Instruction::STX, mode: Addressing::Absolute, cycle: 4});

        map.insert(0x84, Opcode{ name: Instruction::STY, mode: Addressing::ZeroPage, cycle: 3});
        map.insert(0x94, Opcode{ name: Instruction::STY, mode: Addressing::ZeroPageX, cycle: 4});
        map.insert(0x8C, Opcode{ name: Instruction::STY, mode: Addressing::Absolute, cycle: 4});

        map.insert(0xAA, Opcode{ name: Instruction::TAX, mode: Addressing::Implied, cycle: 2});
        map.insert(0xA8, Opcode{ name: Instruction::TAY, mode: Addressing::Implied, cycle: 2});
        map.insert(0xBA, Opcode{ name: Instruction::TSX, mode: Addressing::Implied, cycle: 2});
        map.insert(0x8A, Opcode{ name: Instruction::TXA, mode: Addressing::Implied, cycle: 2});
        map.insert(0x9A, Opcode{ name: Instruction::TXS, mode: Addressing::Implied, cycle: 2});
        map.insert(0x98, Opcode{ name: Instruction::TYA, mode: Addressing::Implied, cycle: 2});
        map
    };
}
#[derive(PartialEq,Debug)]
pub enum Instruction {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    DCP,
    EOR,
    INC,
    INX,
    INY,
    ISB,
    JMP,
    JSR,
    LAX,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    RLA,
    ROL,
    ROR,
    RRA,
    RTI,
    RTS,
    SAX,
    SBC,
    SEC,
    SED,
    SEI,
    SLO,
    SRE,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

#[derive(PartialEq,Debug)]
pub enum Addressing {
    Immediate,
    ZeroPage,
    Relative,
    Implied,
    Absolute,
    Accumulator,
    ZeroPageX,
    ZeroPageY,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
    IndirectAbsolute,
}