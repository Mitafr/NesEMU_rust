#[macro_use]
extern crate lazy_static;

mod cpu;
mod ppu;
mod memory;
mod rom;
mod driver;
mod debugger;

use std::env;
use std::fmt;
use std::option::Option;

use cpu::Cpu;
use rom::Cartbridge;
use cpu::cpu_mem::Ram;
use cpu::cpu_register::CpuRegister;
use cpu::cpu_register::Register;
use cpu::cpu_bus::CpuBus;
use cpu::cpu_bus::Bus;
use ppu::PpuStatus;
use cpu::EmulationStatus;
use debugger::PpuDebugger;

pub struct Context {
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    cpu_register: Register,
    cpu_ram: Ram,
    rom: Cartbridge,
    debugger: Option<PpuDebugger>,
}

impl Context {
    pub fn new(debugging: bool) -> Context {
        let cpu = Cpu::new();
        let cpu_ram = Ram::new();
        let cpu_register = cpu::cpu_register::Register::new();
        let ppu = ppu::Ppu::new();
        let mut cartbridge = Cartbridge::new();
        let mut buffer = cartbridge.read_file(String::from("roms/hello.nes"));
        cartbridge.load_program(&mut buffer);
        let debugger = if debugging {
            Some(PpuDebugger::new())
        } else {
            None
        };
        Context {
            ppu: ppu,
            cpu: cpu,
            cpu_register: cpu_register,
            cpu_ram: cpu_ram,
            rom: cartbridge,
            debugger: debugger,
        }
    }
    pub fn run(&mut self) -> EmulationStatus{
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
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
        if status == EmulationStatus::PROCESSING && self.ppu.has_been_updated() {
            match &mut self.debugger {
                Some(debugger) => {
                    if debugger.is_open() {
                        debugger.draw_tileset(&mut self.ppu.tileset, &self.ppu.palette);
                        debugger.draw_palette(&self.ppu.palette);
                    }
                }
                None => {}
            }
        }
        status
    }
    pub fn init(&mut self) {
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
        self.cpu.reset(&mut cpu_bus, &mut self.cpu_register);
        self.ppu.init(&mut self.rom);
        match &mut self.debugger {
            Some(debugger) => {
                debugger.init();
            }
            None => {}
        }
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut ctx = Context::new(args.get(1) != None && args.get(1).unwrap() == "--debugger");
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