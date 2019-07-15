pub mod ppu_mem;
pub mod ppu_register;
pub mod ppu_renderer;

use crate::ppu::ppu_mem::PpuMem;
use crate::ppu::ppu_register::Register;
use crate::ppu::ppu_register::PpuRegister;
use crate::ppu::ppu_renderer::PpuRenderer;

use std::fmt;

#[derive(PartialEq)]
pub enum PpuStatus {
    BREAK,
    ERROR,
    PROCESSING,
    WAITING,
}

const CYCLE_PER_LINE: u16 = 341;

pub struct Ppu {
    register: PpuRegister,
    mem: PpuMem,
    renderer: PpuRenderer,
    cycle: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            register: PpuRegister::new(),
            mem: PpuMem::new(),
            renderer: PpuRenderer::new("NesEMU"),
            cycle: 0,
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.register.peek(i)
    }
    pub fn write(&mut self, i: usize, v: u8) -> u8 {
        self.register.write(i, v, &mut self.mem)
    }
    pub fn run(&mut self, cycle: u16) -> PpuStatus {
        let cycle = self.cycle + cycle;
        if self.cycle < CYCLE_PER_LINE {
            self.cycle = cycle;
            return PpuStatus::WAITING;
        }
        self.cycle = cycle - CYCLE_PER_LINE;
        self.renderer.draw_window();
        if !self.renderer.is_open() || self.renderer.is_close_key_pressed() {
            return PpuStatus::BREAK;
        }
        PpuStatus::PROCESSING
    }
}

impl fmt::Display for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.register)?;
        write!(f, "{}", self.mem)?;
        write!(f, "End ppu cycle : {}", self.cycle)?;
        Ok(())
    }
}