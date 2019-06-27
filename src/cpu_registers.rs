
pub enum StatusFlags {
    CARRY,
    ZERO,
    INTERRUPT,
    DECIMAL,
    BREAK,
    OVERFLOW,
    NEGATIVE,
}

pub struct Registers {
    r_a: u8,
    r_x: u8,
    r_y: u8,
    stack: Vec<u16>,
    r_pc: u16,
    r_sp: u8,
}
pub trait CpuRegisters {
    fn get_a(&self) -> u8;
    fn get_x(&self) -> u8;
    fn get_y(&self) -> u8;
    fn get_stack(&self) -> Vec<u16>;
    fn get_pc(&self) -> u16;

    fn set_a(&mut self, v: u8) -> &mut Self;
    fn set_x(&mut self, v: u8) -> &mut Self;
    fn set_y(&mut self, v: u8) -> &mut Self;
    fn set_stack(&mut self, v: Vec<u16>) -> &mut Self;
    fn set_pc(&mut self, v: u16) -> &mut Self;

    fn set_flag(&mut self, flag: StatusFlags, v: bool) -> &mut Self;
    fn get_flag(&self, flag: StatusFlags) -> bool;

    fn incr_pc(&mut self) -> &mut Self;
    fn decr_pc(&mut self) -> &mut Self;

    fn push_stack(&mut self, v: u16) -> &mut Self;
    fn pop_stack(&mut self) -> &mut Self;
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            r_a: 0x00,
            r_x: 0x00,
            r_y: 0x00,
            stack: Vec::new(),
            r_pc: 0x600,
            r_sp: 0b00110000,
        }
    }
}

impl CpuRegisters for Registers {
    fn get_a(&self) -> u8 {
        self.r_a
    }
    fn get_x(&self) -> u8 {
        self.r_x
    }
    fn get_y(&self) -> u8 {
        self.r_y
    }
    fn get_stack(&self) -> Vec<u16> {
        self.stack.clone()
    }
    fn get_pc(&self) -> u16 {
        self.r_pc
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
    fn set_stack(&mut self, v: Vec<u16>) -> &mut Self {
        self.stack = v;
        self
    }
    fn set_pc(&mut self, v: u16) -> &mut Self {
        self.r_pc = v;
        self
    }

    fn set_flag(&mut self, flag: StatusFlags, v: bool) -> &mut Self {
        match flag {
            StatusFlags::CARRY => {
                if v {
                    self.r_sp |= 1 << 0;
                } else {
                    self.r_sp &= 0b11111110;
                }
            }
            StatusFlags::ZERO => {
                if v {
                    self.r_sp |= 1 << 1;
                } else {
                    self.r_sp &= 0b11111101;
                }
            }
            StatusFlags::INTERRUPT => {
                if v {
                    self.r_sp |= 1 << 2;
                } else {
                    self.r_sp &= 0b11111011;
                }
            }
            StatusFlags::DECIMAL => {
                if v {
                    self.r_sp |= 1 << 3;
                } else {
                    self.r_sp &= 0b11110111;
                }
            }
            StatusFlags::BREAK => {
                if v {
                    self.r_sp |= 1 << 4;
                } else {
                    self.r_sp &= 0b11101111;
                }
            }
            StatusFlags::OVERFLOW => {
                if v {
                    self.r_sp |= 1 << 6;
                } else {
                    self.r_sp &= 0b10111111;
                }
            }
            StatusFlags::NEGATIVE => {
                if v {
                    self.r_sp |= 1 << 7;
                } else {
                    self.r_sp &= 0b01111111;
                }
            }
        }
        self
    }
    fn get_flag(&self, flag: StatusFlags) -> bool {
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

    fn incr_pc(&mut self) -> &mut Self {
        self.r_pc += 1;
        self
    }
    fn decr_pc(&mut self) -> &mut Self {
        self.r_pc -= 1;
        self
    }

    fn push_stack(&mut self, v: u16) -> &mut Self {
        self.stack.push(v);
        self
    }
    fn pop_stack(&mut self) -> &mut Self {
        self.stack.pop();
        self
    }
}