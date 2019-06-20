mod cpu;
mod memory;

mod gfx;

use cpu::Cpu;
use gfx::Gfx;

fn main() -> Result<(), String> {
    let mut cpu: Cpu;
    cpu = Cpu::new(String::from("roms/test2.nes"));
    cpu.init()?;
    loop {
        cpu.run()?;
    }
    Ok(())
}