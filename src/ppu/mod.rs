pub mod ppu_mem;
pub mod ppu_register;
pub mod ppu_renderer;
pub mod ppu_background;

use crate::ppu::ppu_background::Background;
use crate::ppu::ppu_mem::PpuMem;
use crate::ppu::ppu_register::Register;
use crate::ppu::ppu_register::PpuRegister;
use crate::ppu::ppu_renderer::PpuRenderer;

use std::fmt;
use std::option::Option;

#[derive(PartialEq)]
pub enum PpuStatus {
    BREAK,
    ERROR,
    PROCESSING,
    WAITING,
    RENDERER_NOT_INITIALIZED,
}

const CYCLE_PER_LINE: u16 = 341;

pub struct Ppu {
    register: PpuRegister,
    mem: PpuMem,
    background: Background,
    renderer: Option<PpuRenderer>,
    cycle: u16,
    line: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            register: PpuRegister::new(),
            mem: PpuMem::new(),
            background: Background::new(),
            renderer: None,
            cycle: 0,
            line: 0,
        }
    }
    pub fn peek(&mut self, i: usize) -> u8 {
        self.register.peek(i)
    }
    pub fn write(&mut self, i: usize, v: u8) -> u8 {
        self.register.write(i, v, &mut self.mem)
    }
    pub fn init(&mut self) {
        self.renderer = Some(PpuRenderer::new("NesEMU"));
    }
    pub fn run(&mut self, cycle: u16) -> PpuStatus {
        let cycle = self.cycle + cycle;
        if self.cycle < CYCLE_PER_LINE {
            self.cycle = cycle;
            return PpuStatus::WAITING;
        }
        self.cycle = cycle - CYCLE_PER_LINE;
        if self.line == 0 {
            self.background.clear();
        }
        if self.line < 240 {
            self.background.build(&mut self.mem);
        }
        if self.line >= 262 {
            self.line = 0;
        }
        self.line += 1;
        match &mut self.renderer {
            Some(r) => {
                r.draw_window();
                if !r.is_open() || r.is_close_key_pressed() {
                    return PpuStatus::BREAK;
                }
                PpuStatus::PROCESSING
            }
            None => PpuStatus::RENDERER_NOT_INITIALIZED
        }
    }
}

impl fmt::Display for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.register)?;
        write!(f, "{}", self.mem)?;
        writeln!(f, "End ppu cycle : {}", self.cycle)?;
        write!(f, "Las line rendered : {}", self.line)?;
        Ok(())
    }
}