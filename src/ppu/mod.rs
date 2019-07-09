pub mod ppu_register;
pub mod ppu_mem;

use crate::ppu::ppu_mem::PpuMem;
use crate::ppu::ppu_register::Register;
use crate::ppu::ppu_register::PpuRegister;

use std::fmt;

pub struct Ppu {
    register: Register,
    mem: PpuMem,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            register: Register::new(),
            mem: PpuMem::new(),
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.register.peek(i)
    }
    pub fn write(&mut self, i: usize, v: u8) -> u8 {
        self.register.write(i, v, &mut self.mem)
    }
}

impl fmt::Display for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.register)?;
        Ok(())
    }
}