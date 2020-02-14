use crate::cpu::bus::Bus;

pub enum StatusFlags {
    CARRY,
    ZERO,
    INTERRUPT,
    DECIMAL,
    BREAK,
    UNUSED,
    OVERFLOW,
    NEGATIVE,
}

pub struct Register {
    r_a: u8,
    r_x: u8,
    r_y: u8,
    r_sp: u8,
    r_pc: u16,
    r_sr: u8,
}

pub trait CpuRegister {
    fn get_a(&self) -> u8;
    fn get_x(&self) -> u8;
    fn get_y(&self) -> u8;
    fn get_sp(&self) -> u8;
    fn get_pc(&self) -> u16;
    fn get_sr(&self) -> u8;

    fn set_a(&mut self, v: u8) -> &mut Self;
    fn set_x(&mut self, v: u8) -> &mut Self;
    fn set_y(&mut self, v: u8) -> &mut Self;
    fn set_sp(&mut self, v: u8) -> &mut Self;
    fn set_pc(&mut self, v: u16) -> &mut Self;
    fn set_sr(&mut self, v: u8) -> &mut Self;

    fn set_flag(&mut self, flag: StatusFlags, v: bool) -> &mut Self;
    fn get_flag(&self, flag: StatusFlags) -> bool;

    fn incr_pc(&mut self) -> &mut Self;
    fn decr_pc(&mut self) -> &mut Self;

    fn incr_sp(&mut self) -> &mut Self;
    fn decr_sp(&mut self) -> &mut Self;

    fn push_stack<B: Bus>(&mut self, v: u8, bus: &mut B) -> &mut Self;
    fn pop_stack<B: Bus>(&mut self, bus: &mut B) -> u8;
}

impl Register {
    pub fn new() -> Register {
        Register {
            r_a: 0x00,
            r_x: 0x00,
            r_y: 0x00,
            r_sp: 0xff,
            r_pc: 0x8000,
            r_sr: 0b0010_0000,
        }
    }
}

impl CpuRegister for Register {
    fn get_a(&self) -> u8 {
        self.r_a
    }
    fn get_x(&self) -> u8 {
        self.r_x
    }
    fn get_y(&self) -> u8 {
        self.r_y
    }
    fn get_sp(&self) -> u8 {
        self.r_sp
    }
    fn get_pc(&self) -> u16 {
        self.r_pc
    }
    fn get_sr(&self) -> u8 {
        self.r_sr
    }
    fn set_a(&mut self, v: u8) -> &mut Self {
        self.r_a = v;
        self
    }
    fn set_x(&mut self, v: u8) -> &mut Self {
        self.r_x = v;
        self
    }
    fn set_y(&mut self, v: u8) -> &mut Self {
        self.r_y = v;
        self
    }
    fn set_sp(&mut self, v: u8) -> &mut Self {
        self.r_sp = v;
        self
    }
    fn set_pc(&mut self, v: u16) -> &mut Self {
        self.r_pc = v;
        self
    }
    fn set_sr(&mut self, v: u8) -> &mut Self {
        self.r_sr = v;
        self
    }
    fn set_flag(&mut self, flag: StatusFlags, v: bool) -> &mut Self {
        if v {
            match flag {
                StatusFlags::CARRY => self.r_sr |= 1 << 0,
                StatusFlags::ZERO => self.r_sr |= 1 << 1,
                StatusFlags::INTERRUPT => self.r_sr |= 1 << 2,
                StatusFlags::DECIMAL => self.r_sr |= 1 << 3,
                StatusFlags::BREAK => self.r_sr |= 1 << 4,
                StatusFlags::UNUSED => self.r_sr |= 1 << 5,
                StatusFlags::OVERFLOW => self.r_sr |= 1 << 6,
                StatusFlags::NEGATIVE => self.r_sr |= 1 << 7,
            }
        } else {
            match flag {
                StatusFlags::CARRY => self.r_sr &= 0b11111110,
                StatusFlags::ZERO => self.r_sr &= 0b11111101,
                StatusFlags::INTERRUPT => self.r_sr &= 0b11111011,
                StatusFlags::DECIMAL => self.r_sr &= 0b11110111,
                StatusFlags::BREAK => self.r_sr &= 0b11101111,
                StatusFlags::UNUSED => self.r_sr &= 0b11011111,
                StatusFlags::OVERFLOW => self.r_sr &= 0b10111111,
                StatusFlags::NEGATIVE => self.r_sr &= 0b01111111,
            }
        }
        self
    }
    fn get_flag(&self, flag: StatusFlags) -> bool {
        match flag {
            StatusFlags::CARRY => self.r_sr & 0b00000001 == 0b00000001,
            StatusFlags::ZERO => self.r_sr & 0b00000010 == 0b00000010,
            StatusFlags::INTERRUPT => self.r_sr & 0b00000100 == 0b00000100,
            StatusFlags::DECIMAL => self.r_sr & 0b00001000 == 0b00001000,
            StatusFlags::BREAK => self.r_sr & 0b00010000 == 0b00010000,
            StatusFlags::UNUSED => self.r_sr & 0b00100000 == 0b00100000,
            StatusFlags::OVERFLOW => self.r_sr & 0b01000000 == 0b01000000,
            StatusFlags::NEGATIVE => self.r_sr & 0b10000000 == 0b10000000,
        }
    }
    fn incr_pc(&mut self) -> &mut Self {
        self.r_pc += 1;
        self
    }
    fn decr_pc(&mut self) -> &mut Self {
        self.r_pc -= 1;
        self
    }
    fn incr_sp(&mut self) -> &mut Self {
        self.r_sp += 1;
        self
    }
    fn decr_sp(&mut self) -> &mut Self {
        self.r_sp -= 1;
        self
    }
    fn push_stack<B: Bus>(&mut self, v: u8, bus: &mut B) -> &mut Self {
        bus.write(self.r_sp as u16 | 0x0100, v);
        self.decr_sp();
        self
    }
    fn pop_stack<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.incr_sp();
        let val = bus.peek(0x0100 | self.r_sp as u16);
        val
    }
}