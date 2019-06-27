use std::fs::File;
use std::fmt;
use std::io::prelude::*;

pub struct Cartbridge {
    path: String,
    program: Vec<u8>,
}

impl Cartbridge {
    pub fn new() -> Cartbridge {
        Cartbridge {
            path: String::new(),
            program: Vec::new(),
        }
    }
    pub fn load_from_file(&mut self, path: String) {
        self.path = path;
        println!("Loading : {}", self.path);
        
        let mut f = File::open(&self.path).expect(&format!("file not found: {}", self.path));
        let mut buffer = [0u8; 0xFFFF];
        if let Ok(bytes_read) = f.read(&mut buffer) {
            bytes_read
        } else {
            0
        };
        for (_i, byte) in buffer.bytes().enumerate() {
            let bit: u8 = byte.unwrap();
            self.program.push(bit);
        }
    }
    pub fn load_from_vec(&mut self, program: &Vec<u8>) {
        for byte in program.bytes() {
            let bit: u8 = byte.unwrap();
            self.program.push(bit);
        }
    }
    pub fn get_program(&mut self) -> &mut Vec<u8> {
        &mut self.program
    }
}