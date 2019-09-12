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
use ppu::register::Register as PpuRegister;

pub struct Context {
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    cpu_ram: Ram,
    rom: Cartbridge,
    debugger: Option<PpuDebugger>,
    renderer: Renderer,
    events: EventPump,
    cpu_cycle: usize,
    ppu_cycle: usize,
}

impl Context {
    pub fn new() -> Context {
        let cpu = Cpu::new();
        let cpu_ram = Ram::new();
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
            cpu_ram: cpu_ram,
            rom: cartbridge,
            debugger: debugger,
            events: events,
            renderer: renderer,
            cpu_cycle: 0,
            ppu_cycle: 0,
        }
    }
    pub fn run(&mut self) -> EmulationStatus{
        println!("TIMING: CPU: {:?}, PPU: {:?}", self.cpu_cycle, self.ppu_cycle);
        println!("VRAM Address : {:x?}", self.ppu.register.get_addr());
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
        let cpu_cb: (u16, EmulationStatus) = self.cpu.run(&mut cpu_bus);
        let mut status = cpu_cb.1;
        let s = self.ppu.run(cpu_cb.0 * 3);
        self.cpu_cycle += cpu_cb.0 as usize;
        self.ppu_cycle += cpu_cb.0 as usize * 3;
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
            self.ppu.background.draw(&mut self.renderer, &mut self.ppu.palette);
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
        self.cpu_cycle = 7;
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu);
        self.cpu.reset(&mut cpu_bus);
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
        writeln!(f, "{}", self.cpu)?;
        writeln!(f, "{}", self.ppu)?;
        Ok(())
    }
}