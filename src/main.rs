#[macro_use]
extern crate lazy_static;
use sdl2;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod cpu;
mod debugger;
mod driver;
mod memory;
mod ppu;
mod renderer;
mod rom;

use std::env;
use std::fmt;
use std::option::Option;

use cpu::Cpu;
use rom::Cartbridge;
use cpu::cpu_mem::Ram;
use cpu::cpu_register::CpuRegister;
use cpu::cpu_register::Register;
use cpu::cpu_bus::CpuBus;
#[allow(unused_imports)]
use cpu::cpu_bus::Bus;
use cpu::EmulationStatus;
use debugger::PpuDebugger;
use renderer::Renderer;
use ppu::PpuStatus;

pub struct Context {
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    cpu_register: Register,
    cpu_ram: Ram,
    rom: Cartbridge,
    debugger: Option<PpuDebugger>,
    renderer: Renderer,
    events: EventPump,
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
        let sdl_context = sdl2::init().unwrap();
        let events: EventPump = sdl_context.event_pump().unwrap();
        let debugger = Some(PpuDebugger::new(&sdl_context));
        let renderer = Renderer::new(&sdl_context, "NesEmu");
        Context {
            ppu: ppu,
            cpu: cpu,
            cpu_register: cpu_register,
            cpu_ram: cpu_ram,
            rom: cartbridge,
            debugger: debugger,
            events: events,
            renderer: renderer,
        }
    }
    pub fn run(&mut self) -> EmulationStatus{
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
        let cpu_cb: (u16, EmulationStatus) = self.cpu.run(&mut cpu_bus, &mut self.cpu_register);
        let mut status = cpu_cb.1;
        let s = self.ppu.run(cpu_cb.0 * 3);
        for event in self.events.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return EmulationStatus::BREAK;
                },
                Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                    match &mut self.debugger {
                        Some(debugger) => {
                            debugger.toggle_view();
                            self.ppu.force_update();
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }
        if s == PpuStatus::ERROR || s == PpuStatus::BREAK {
            status = EmulationStatus::BREAK;
        }
        if s == PpuStatus::RENDERERNOTINITIALIZED {
            panic!("PPu Renderer not properly initialized");
        }
        if s == PpuStatus::RENDERING {
            &mut self.ppu.background.draw(&mut self.renderer, &mut self.ppu.palette);
            self.ppu.sprites.draw(&mut self.renderer, &mut self.ppu.palette);
            self.renderer.draw_window();
        }
        if status == EmulationStatus::PROCESSING && self.ppu.has_been_updated() {
            match &mut self.debugger {
                Some(debugger) => {
                    if debugger.is_open() {
                        debugger.draw_tileset(&self.ppu.tileset, &self.ppu.palette);
                        debugger.draw_palette(&self.ppu.palette);
                        debugger.draw();
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