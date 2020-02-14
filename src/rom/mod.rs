use crate::memory::Memory;
use std::path::Path;
use std::str;

pub struct Cartbridge {
    program: Vec<u8>,
    character: Vec<u8>,
    mapper: u8,
    size: usize,
    pub offset: usize,
}

impl Cartbridge {
    pub fn new() -> Cartbridge {
        Cartbridge {
            program: Vec::new(),
            character: Vec::new(),
            mapper: 0,
            size: 0,
            offset: 0,
        }
    }
    pub fn read_file(&mut self, path: String) -> Vec<u8> {
        println!("ROM: Loading : {}", path);
        match std::fs::read(Path::new(&path)) {
            Result::Ok(rom) => {
                return rom;
            },
            Result::Err(err) => {
                println!("ROM: Cannot open .nes file: {}", path);
                panic!(err);
            }
        }
    }
    #[allow(dead_code)]
    pub fn load_from_vec(&mut self, program: &Vec<u8>) {
        for i in program.iter() {
            self.program.push(*i);
        }
        self.program.resize(0x8000, 0u8);
        self.size = 0x8000;
    }
    #[allow(dead_code)]
    pub fn get_program(&mut self) -> &mut Vec<u8> {
        &mut self.program
    }
    pub fn get_character(&mut self) -> &mut Vec<u8> {
        &mut self.character
    }
    pub fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self {
        println!("ROM: Loading buffer (size : {}) into Rom memory", data.len());
        let rom_name = str::from_utf8(&data[0..3]).unwrap();
        if rom_name != "NES" {
            panic!("ROM: Invalid ROM name header");
        }
        let next = str::from_utf8(&data[3..4]).unwrap();
        if next != "\x1a" {
            panic!("ROM: Invalid ROM header");
        }
        let prg_pages = data[4] as usize;
        println!("ROM: PRG_PAGES: {}", prg_pages);
        self.offset = prg_pages * 0x4000;
        let chr_pages = data[5] as usize;
        let _rom_control_one = data[6] & 0x01; // 0
        let mut character_rom_start = 0x0010 + prg_pages * 0x4000;
        let character_rom_end = character_rom_start + chr_pages * 0x2000;
        if character_rom_start + 0x0010 > data.len() {
            character_rom_start = data.len() - 0x0010;
        }
        self.program = data[0x0010..0x0010 + character_rom_start].to_vec();
        println!("ROM: PRG-ROM: {}", self.program.len());
        self.character = data[character_rom_start..character_rom_end].to_vec();
        println!("ROM: CHR-ROM: {}", self.character.len());
        self.size = self.program.len();
        self.mapper = 0;
        self
    }
}

impl Memory for Cartbridge {
    fn get_size(&self) -> usize {
        self.size
    }
    fn peek(&self, i: u16) -> u8 {
        self.program[i as usize]
    }
    fn write(&mut self, i: u16, value: u8) -> u8 {
        self.program[i as usize] = value;
        value
    }
    fn get_mem(&self) -> &[u8] {
        &self.program[0..self.size]
    }
}