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
            r_a: 0,
            r_x: 0,
            r_y: 0,
            r_sp: 0xff,
            r_pc: 0x8000,
            r_sr: 0x20,
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
        match flag {
            StatusFlags::CARRY => if v {self.r_sr |= 0x1} else {self.r_sr &= 0xfe},
            StatusFlags::ZERO => if v {self.r_sr |= 0x2} else {self.r_sr &= 0xfd},
            StatusFlags::INTERRUPT => if v {self.r_sr |= 0x4} else {self.r_sr &= 0xfb},
            StatusFlags::DECIMAL => if v {self.r_sr |= 0x8} else {self.r_sr &= 0xf7},
            StatusFlags::BREAK => if v {self.r_sr |= 0x10} else {self.r_sr &= 0xef},
            StatusFlags::UNUSED => if v {self.r_sr |= 0x20} else {self.r_sr &= 0xdf},
            StatusFlags::OVERFLOW => if v {self.r_sr |= 0x40} else {self.r_sr &= 0xbf},
            StatusFlags::NEGATIVE => if v {self.r_sr |= 0x80} else {self.r_sr &= 0x7F},
        }
        self
    }
    fn get_flag(&self, flag: StatusFlags) -> bool {
        match flag {
            StatusFlags::CARRY => self.r_sr & 0x1 == 0x1,
            StatusFlags::ZERO => self.r_sr & 0x2 == 0x2,
            StatusFlags::INTERRUPT => self.r_sr & 0x4 == 0x4,
            StatusFlags::DECIMAL => self.r_sr & 0x8 == 0x8,
            StatusFlags::BREAK => self.r_sr & 0x10 == 0x10,
            StatusFlags::UNUSED => self.r_sr & 0x20 == 0x20,
            StatusFlags::OVERFLOW => self.r_sr & 0x40 == 0x40,
            StatusFlags::NEGATIVE => self.r_sr & 0x80 == 0x80,
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