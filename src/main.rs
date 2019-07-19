#[macro_use]
extern crate lazy_static;

mod cpu;
mod ppu;
mod memory;
mod rom;
mod driver;
mod debugger;

use std::fmt;

use cpu::Cpu;
use rom::Cartbridge;
use cpu::cpu_mem::Ram;
use cpu::cpu_register::*;
use ppu::PpuStatus;
use cpu::EmulationStatus;
use debugger::PpuDebugger;
use debugger::DebuggerStatus;

pub struct Context {
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    cpu_register: cpu::cpu_register::Register,
    cpu_ram: Ram,
    rom: Cartbridge,
    debugger: PpuDebugger,
}

impl Context {
    pub fn new() -> Context {
        let cpu = Cpu::new();
        let cpu_ram = Ram::new();
        let cpu_register = cpu::cpu_register::Register::new();
        let ppu = ppu::Ppu::new();
        let mut cartbridge = Cartbridge::new();
        let mut buffer = cartbridge.read_file(String::from("roms/hello.nes"));
        cartbridge.load_program(&mut buffer);
        Context {
            ppu: ppu,
            cpu: cpu,
            cpu_register: cpu_register,
            cpu_ram: cpu_ram,
            rom: cartbridge,
            debugger: PpuDebugger::new(),
        }
    }
    pub fn run(&mut self) -> EmulationStatus{
        let mut cpu_bus = cpu::cpu_bus::CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
        let cpu_cb: (u16, EmulationStatus) = self.cpu.run(&mut cpu_bus, &mut self.cpu_register);
        let mut status = cpu_cb.1;
        match self.ppu.run(cpu_cb.0) {
            s => {
                if s == PpuStatus::ERROR || s == PpuStatus::BREAK {
                    status = EmulationStatus::BREAK;
                }
                if s == PpuStatus::RENDERER_NOT_INITIALIZED {
                    panic!("PPu Renderer not properly initialized");
                }
            }
        }
        if status == EmulationStatus::PROCESSING {
            self.debugger.draw_tileset(&mut self.ppu.tileset, &self.ppu.palette);
            self.debugger.draw_palette(&self.ppu.palette);
        }
        status
    }
    pub fn init(&mut self) {
        let mut cpu_bus = cpu::cpu_bus::CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
        self.cpu.reset(&mut cpu_bus, &mut self.cpu_register);
        self.ppu.init(&mut self.rom);
        self.debugger.init();
    }
}

fn main() -> Result<(), String> {
    let mut ctx = Context::new();
    ctx.init();
    'main: loop {
        match ctx.run() {
            s => {
                if s == EmulationStatus::ERROR || s == EmulationStatus::BREAK {
                    break 'main;
                }
            }
        }
    }
    println!("{}", ctx);
    Ok(())
}


impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "===========CPU===========")?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "Flags: {:08b}", self.cpu_register.get_sr())?;
        writeln!(f, "Accumulator: ${:x?}", self.cpu_register.get_a())?;
        writeln!(f, "X: ${:x?}", self.cpu_register.get_x())?;
        writeln!(f, "Y: ${:x?}", self.cpu_register.get_y())?;
        writeln!(f, "Stack: ${:x?}", self.cpu_register.get_sp())?;
        writeln!(f, "End PC: ${:x?}", self.cpu_register.get_pc())?;
        writeln!(f, "Opcode counter: {}", self.cpu.opcode_counter)?;
        writeln!(f, "{}", self.ppu)?;
        Ok(())
    }
}