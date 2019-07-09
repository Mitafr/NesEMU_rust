#[macro_use]
extern crate lazy_static;
mod cpu;
mod ppu;
mod memory;
mod rom;
mod driver;

use sdl2;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::fmt;

use cpu::Cpu;
use rom::Cartbridge;
use memory::Ram;
use rom::rom_mem::Rom;
use memory::Memory;
use cpu::cpu_register::*;
use ppu::Ppu;
use cpu::EmulationStatus;
use driver::gfx::Gfx;

struct Context {
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    cpu_register: cpu::cpu_register::Register,
    cpu_ram: memory::Ram,
    cpu_rom: Cartbridge,
}

impl Context {
    pub fn new() -> Context {
        let mut cpu = Cpu::new();
        let mut cpu_ram = memory::Ram::new();
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
            cpu_rom: cartbridge,
        }
    }
    pub fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let mut events: EventPump = sdl_context.event_pump().unwrap();
        let gfx: Gfx = Gfx::new(&sdl_context, "Mos6502");
        let mut cpu_bus = cpu::cpu_bus::CpuBus::new(&mut self.cpu_ram, &mut self.cpu_rom, &mut self.ppu);
        //cpu.reset();
        println!("{}", cpu_bus);
        //println!("{}", self.ppu);
        'main: loop {
            for event in events.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
                    _ => {}
                }
            }
            match self.cpu.run(&mut cpu_bus, &mut self.cpu_register) {
                s => {
                    if s == EmulationStatus::ERROR || s == EmulationStatus::BREAK {
                        break 'main;
                    }
                }
            }
        }
        //println!("{}", self.cpu);
        //println!("{}", self.ppu);
    }
}

fn main() -> Result<(), String> {
    let mut ctx = Context::new();
    println!("{}", ctx);
    ctx.run();
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
        writeln!(f, "========================")?;
        Ok(())
    }
}