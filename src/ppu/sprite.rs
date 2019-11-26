use std::fmt;

use crate::ppu::register::Register;
use crate::ppu::register::PpuRegister;
use crate::ppu::mem::PpuMem;
use crate::ppu::palette::Palette;
use crate::renderer::Renderer;

pub struct SpriteMem {
    mem: [u8; 0x100],
}

impl SpriteMem {
    pub fn new() -> SpriteMem {
        SpriteMem {
            mem: [0; 0x100],
        }
    }

    /*pub fn peek(&mut self, i: usize) -> u8 {
        self.mem[i]
    }*/
    pub fn write_data(&mut self, i: usize, value: u8) -> u8 {
        println!("Writing in SPR-RAM at {:x?} -> {:x?} ({:08b})", i, value, value);
        self.mem[i] = value;
        value
    }
    pub fn get_mem(&self) -> &[u8] {
        &self.mem
    }
}

impl fmt::Display for SpriteMem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "|----------PPU SPRITEMEM--------------|")?;
        writeln!(f, "|--------------++---------------|")?;
        writeln!(f, "|\tadresse =>    value  |")?;
        writeln!(f, "|--------------++---------------|")?;
        for (i, b) in self.mem.iter().enumerate() {
            if i % 16 == 0 {    
                write!(f, "\n{:04x?}:", i)?;
            }
            if i % 4 == 0 {
                write!(f, "  ")?;
            }
            write!(f, "{:02x?} " , b)?;
        }
        Ok(())
    }
}

pub struct Sprite {
    pub coord_y: u8,
    pub coord_x: u8,
    #[allow(dead_code)]
    index: u8,
    pixels: Vec<Vec<u8>>,
    #[allow(dead_code)]
    flags: u8,
}

impl Sprite {
    pub fn new(value: u8) -> Sprite {
        Sprite {
            coord_x: value & 0b0000_1000,
            coord_y: value & 0b0000_0001,
            index: value & 0b0000_0010,
            pixels: (0..8).into_iter().map(|_| vec![0; 8]).collect(),
            flags: value & 0b0000_0100,
        }
    }
    pub fn build_sprite(&mut self, slice: &[u8; 16]) {
        for j in 0..8 {
            let tilelow = slice[j];
            let tilehi = slice[j + 8];
            for k in 0..8 {
                let vv = (((0b1000_0000 >> k) & tilelow) / (0x80 >> k)) << 1;
                let vvv = ((0b1000_0000 >> k) & tilehi) / (0x80 >> k);
                self.pixels[j as usize][k as usize] = vv + vvv;
            }
        }
    }
    #[allow(dead_code)]
    pub fn get_index(&self) -> u8 {
        self.index
    }
    pub fn get_pixels(&self) -> &Vec<Vec<u8>> {
        &self.pixels
    }
}

pub struct SpriteSet {
    sprites: Vec<Sprite>,
}

impl SpriteSet {
    pub fn new() -> SpriteSet {
        SpriteSet {
            sprites: vec!(),
        }
    }
    pub fn build(&mut self, spr_mem: &mut SpriteMem, register: &mut PpuRegister, vram: &mut PpuMem) {
            for i in spr_mem.get_mem().iter() {
                if *i == 0 {
                    continue;
                }
                let mut index = (*i & 0b0000_0010) as usize;
                let mut offset = 0;
                if register.get_sprite_table() == 1 {
                    index += 0x1000;
                    offset = 0x1000;
                }
                while index < offset + 16 {
                    let mut v = [0; 16];
                    for j in 0..16 {
                        v[j] = vram.peek(index as usize);
                        index += 1;
                    }
                    let mut sprite = Sprite::new(*i);
                    sprite.build_sprite(&v);
                    self.sprites.push(sprite);
                }
            }
    }
    pub fn draw(&mut self, renderer: &mut Renderer, palette: &mut Palette) {
        for i in self.sprites.iter_mut() {
            renderer.set_sprite(i, palette);
        }
    }
    pub fn len(&self) -> usize {
        self.sprites.len()
    }
}