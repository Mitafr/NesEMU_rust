mod cpu;
mod memory;

mod gfx;

use sdl2;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use cpu::Cpu;
use gfx::Gfx;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let mut events: EventPump = sdl_context.event_pump().unwrap();
    let gfx: Gfx = Gfx::new(&sdl_context, "Mos6502");
    let mut cpu: Cpu = Cpu::new(String::from("roms/snake.nes"));
    cpu.init()?;
    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
                _ => {}
            }
        }
        cpu.run()?;
    }
    println!("{}", cpu);
    Ok(())
}