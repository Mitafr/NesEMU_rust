#[macro_use]
extern crate lazy_static;
extern crate ini;
use sdl2;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use ini::Ini;

mod controller;
mod cpu;
mod debugger;
mod driver;
mod memory;
mod ppu;
mod renderer;
mod rom;

use std::fmt;
use std::fs;
use std::option::Option;
use std::path::Path;

use cpu::Cpu;
use rom::Cartbridge;
use cpu::memory::Ram;
#[allow(unused_imports)]
use cpu::register::CpuRegister;
#[allow(unused_imports)]
use cpu::register::Register;
#[allow(unused_imports)]
use cpu::bus::CpuBus;
#[allow(unused_imports)]
use cpu::bus::Bus;
use cpu::EmulationStatus;
use debugger::PpuDebugger;
use renderer::Renderer;
use ppu::PpuStatus;
use controller::Controller;

pub type Cycle = u64;

pub struct Context {
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    cpu_ram: Ram,
    rom: Cartbridge,
    debugger: Option<PpuDebugger>,
    renderer: Option<Renderer>,
    controller: Controller,
    events: EventPump,
    cpu_cycle: Cycle,
    ppu_cycle: Cycle,
    config_path: String,
    sdl_context: sdl2::Sdl,
}

impl Context {
    pub fn new() -> Context {
        let cpu = Cpu::new();
        let cpu_ram = Ram::new();
        let ppu = ppu::Ppu::new();
        let mut cartbridge = Cartbridge::new();
        let mut buffer = cartbridge.read_file(String::from("roms/nestest.nes"));
        cartbridge.load_program(&mut buffer);
        let sdl_context = sdl2::init().unwrap();
        let events: EventPump = sdl_context.event_pump().unwrap();
        let controller = Controller::new();
        Context {
            ppu: ppu,
            cpu: cpu,
            cpu_ram: cpu_ram,
            rom: cartbridge,
            debugger: None,
            controller: controller,
            events: events,
            renderer: None,
            cpu_cycle: 0,
            ppu_cycle: 0,
            config_path: String::from("config/config.ini"),
            sdl_context,
        }
    }
    pub fn run(&mut self) -> EmulationStatus{
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu, &mut self.controller);
        let cpu_cb: (Cycle, EmulationStatus) = self.cpu.run(&mut cpu_bus);
        let mut status = cpu_cb.1;
        for _ in 0..cpu_cb.0 * 3 {
            match self.ppu.run() {
                PpuStatus::RENDERING => {
                    match &mut self.renderer {
                        Some(renderer) => {
                            self.ppu.background.draw(renderer, &mut self.ppu.palette);
                            self.ppu.sprites.draw(renderer, &mut self.ppu.palette);
                            renderer.draw_window();
                        }
                        None => {}
                    }
                }
                PpuStatus::INTERRUPTNMI => self.cpu.trigger_nmi(),
                PpuStatus::PROCESSING => {}
            }
        }
        self.cpu_cycle += cpu_cb.0 as Cycle;
        self.ppu_cycle += cpu_cb.0 as Cycle * 3;
        for event in self.events.poll_iter() {
            self.controller.poll_events(&event);
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
                },
                Event::KeyDown { keycode: Some(Keycode::R), ..} => {
                    status = EmulationStatus::RESET;
                }
                _ => {}
            }
        }
        if status == EmulationStatus::PROCESSING && self.ppu.has_been_updated() {
            match &mut self.debugger {
                Some(debugger) => {
                    if debugger.is_open() {
                        debugger.draw_tileset(&self.ppu.tileset, &self.ppu.palette);
                        debugger.draw_palette(&self.ppu.palette);
                        debugger.draw();
                        self.ppu.clear_updated();
                    }
                }
                None => {}
            }
        }
        status
    }
    fn create_config_file(&self) {
        if !Path::new("config").exists() {
            fs::create_dir("config").unwrap();
        }
        if !Path::new(&self.config_path).exists() {
            let mut conf = Ini::new();
            conf.with_section(Some("Display".to_owned()))
                .set("scale", "1.1");
            conf.with_section(Some("Debugger".to_owned()))
                .set("scale", "2.0");
            conf.write_to_file(&self.config_path).unwrap();
        }
    }
    pub fn init(&mut self) {
        self.create_config_file();
        let conf = Ini::load_from_file(&self.config_path).unwrap();
        let renderer_scale = conf.section(Some("Display".to_owned())).unwrap().get("scale").unwrap().parse::<f32>().unwrap();
        self.renderer = Some(Renderer::new(&self.sdl_context, "NesEMU", renderer_scale));
        let debugger_scale = conf.section(Some("Debugger".to_owned())).unwrap().get("scale").unwrap().parse::<f32>().unwrap();
        self.debugger = Some(PpuDebugger::new(&self.sdl_context, debugger_scale));
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu, &mut self.controller);
        println!("CPU: Resetting");
        self.cpu_cycle += self.cpu.reset(&mut cpu_bus) as u64;
        println!("PPU: Initializing ...");
        self.ppu.init(&mut self.rom);
        println!("PPU: Initialized successfully");
        match &mut self.debugger {
            Some(debugger) => {
                debugger.init();
            }
            None => {}
        }
    }
    pub fn reset(&mut self) {
        self.cpu_ram = Ram::new();
        self.cpu_cycle = 0;
        self.ppu_cycle = 0;
        self.ppu.reset();
        let mut cpu_bus = CpuBus::new(&mut self.cpu_ram, &mut self.rom, &mut self.ppu, &mut self.controller);
        self.cpu.reset(&mut cpu_bus);
        self.ppu.reset();
        println!("PPU: Initializing ...");
        self.ppu.init(&mut self.rom);
        match &mut self.renderer {
            Some(renderer) => renderer.reset(),
            None => {}
        }
    }
}

fn main() -> Result<(), String> {
    let mut ctx = Context::new();
    ctx.init();
    'main: loop {
        'run: loop {
            match ctx.run() {
                s => {
                    if s == EmulationStatus::ERROR || s == EmulationStatus::BREAK {
                        break 'main;
                    }
                    if s == EmulationStatus::RESET {
                        ctx.reset();
                        break 'run;
                    }
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